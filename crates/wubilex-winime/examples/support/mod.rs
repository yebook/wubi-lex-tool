#![allow(dead_code)]

use std::fmt;
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{CloseHandle, HANDLE, WIN32_ERROR};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    DryRun,
    Live,
}

pub fn parse_mode() -> Result<Mode, SpikeError> {
    parse_mode_from(std::env::args().skip(1))
}

fn parse_mode_from<I, S>(arguments: I) -> Result<Mode, SpikeError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut mode = Mode::DryRun;
    for argument in arguments {
        let argument = argument.as_ref();
        if argument == "--live" && mode == Mode::DryRun {
            mode = Mode::Live;
        } else {
            return Err(SpikeError::new(
                "arguments",
                format!("unsupported or duplicate argument: {argument}"),
            ));
        }
    }
    Ok(mode)
}

#[derive(Debug)]
pub struct SpikeError {
    stage: &'static str,
    detail: String,
}

impl SpikeError {
    pub fn new(stage: &'static str, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
        }
    }

    pub fn windows(stage: &'static str, error: windows::core::Error) -> Self {
        Self::new(
            stage,
            format!("HRESULT=0x{:08X}: {error}", error.code().0 as u32),
        )
    }

    pub fn win32(stage: &'static str, code: WIN32_ERROR) -> Self {
        let hresult = windows::core::HRESULT::from_win32(code.0);
        let system_error = windows::core::Error::from_hresult(hresult);
        Self::new(
            stage,
            format!(
                "Win32={} (0x{:08X}), HRESULT=0x{:08X}: {system_error}",
                code.0, code.0, hresult.0 as u32
            ),
        )
    }

    pub fn with_restore(self, restore: Self) -> Self {
        Self::new(
            self.stage,
            format!(
                "{}; RESTORATION FAILURE at {}: {}",
                self.detail, restore.stage, restore.detail
            ),
        )
    }
}

impl fmt::Display for SpikeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "stage={}: {}", self.stage, self.detail)
    }
}

impl std::error::Error for SpikeError {}

pub struct ComApartment;

impl ComApartment {
    pub fn initialize() -> Result<Self, SpikeError> {
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        result
            .ok()
            .map_err(|error| SpikeError::windows("CoInitializeEx", error))?;
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

pub struct OwnedHandle(HANDLE);

impl OwnedHandle {
    pub fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

    pub fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            let _ = unsafe { CloseHandle(self.0) };
        }
    }
}

pub fn is_elevated() -> Result<bool, SpikeError> {
    let mut token = HANDLE::default();
    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
            .map_err(|error| SpikeError::windows("OpenProcessToken", error))?;
    }
    let token = OwnedHandle::new(token);
    let mut elevation = TOKEN_ELEVATION::default();
    let mut returned = 0u32;
    unsafe {
        GetTokenInformation(
            token.raw(),
            TokenElevation,
            Some((&raw mut elevation).cast()),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
        .map_err(|error| SpikeError::windows("GetTokenInformation(TokenElevation)", error))?;
    }
    if returned < std::mem::size_of::<TOKEN_ELEVATION>() as u32 {
        return Err(SpikeError::new(
            "GetTokenInformation(TokenElevation)",
            format!("short response: {returned} bytes"),
        ));
    }
    Ok(elevation.TokenIsElevated != 0)
}

pub fn require_elevated() -> Result<(), SpikeError> {
    if is_elevated()? {
        Ok(())
    } else {
        Err(SpikeError::new(
            "live-preflight",
            "--live requires an elevated Windows process; no mutation was attempted",
        ))
    }
}

pub fn poll_until<T, F, P>(
    stage: &'static str,
    timeout: Duration,
    interval: Duration,
    mut sample: F,
    mut predicate: P,
) -> Result<T, SpikeError>
where
    F: FnMut() -> Result<T, SpikeError>,
    P: FnMut(&T) -> bool,
{
    let deadline = Instant::now() + timeout;
    loop {
        let value = sample()?;
        if predicate(&value) {
            return Ok(value);
        }
        if Instant::now() >= deadline {
            return Err(SpikeError::new(
                stage,
                "timed out waiting for verified state",
            ));
        }
        thread::sleep(interval);
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, parse_mode_from, poll_until};
    use std::time::Duration;

    #[test]
    fn mode_is_dry_run_without_arguments() {
        assert_eq!(
            parse_mode_from(Vec::<String>::new()).ok(),
            Some(Mode::DryRun)
        );
    }

    #[test]
    fn mode_accepts_exactly_one_live_switch() {
        assert_eq!(parse_mode_from(["--live"]).ok(), Some(Mode::Live));
        assert!(parse_mode_from(["--live", "--live"]).is_err());
        assert!(parse_mode_from(["--unknown"]).is_err());
    }

    #[test]
    fn poll_returns_matching_sample() {
        let mut next = 0;
        let result = poll_until(
            "test",
            Duration::from_millis(10),
            Duration::ZERO,
            || {
                next += 1;
                Ok(next)
            },
            |value| *value == 3,
        );
        assert_eq!(result.ok(), Some(3));
    }

    #[test]
    fn poll_reports_timeout_when_no_sample_matches() {
        let result = poll_until(
            "test timeout",
            Duration::ZERO,
            Duration::ZERO,
            || Ok(1),
            |value| *value == 2,
        );
        let error = result.expect_err("a zero-duration poll must time out");
        assert!(error.to_string().contains("stage=test timeout"));
    }
}

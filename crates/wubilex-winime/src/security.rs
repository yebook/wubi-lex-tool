//! Current-process Windows token inspection.

use thiserror::Error;

/// A staged native privilege-probe failure.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{stage} failed with Win32/HRESULT code {code}: {message}")]
pub struct NativeSecurityError {
    /// Native operation stage.
    pub stage: &'static str,
    /// HRESULT represented as an unsigned code.
    pub code: u32,
    /// Readable system detail.
    pub message: String,
}

impl NativeSecurityError {
    fn new(stage: &'static str, code: u32, message: impl Into<String>) -> Self {
        Self {
            stage,
            code,
            message: message.into(),
        }
    }
}

/// Adapter boundary for privilege probing and deterministic callers/tests.
pub trait ElevationProbe {
    /// Returns whether the current process token is elevated.
    fn is_elevated(&self) -> Result<bool, NativeSecurityError>;
}

/// Native current-process probe.
#[derive(Clone, Copy, Debug, Default)]
pub struct CurrentProcessElevationProbe;

/// Detects actual token elevation independently from the executable manifest.
pub fn current_process_is_elevated() -> Result<bool, NativeSecurityError> {
    CurrentProcessElevationProbe.is_elevated()
}

#[cfg(windows)]
impl ElevationProbe for CurrentProcessElevationProbe {
    fn is_elevated(&self) -> Result<bool, NativeSecurityError> {
        use windows::Win32::{
            Foundation::{CloseHandle, HANDLE},
            Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation},
            System::Threading::{GetCurrentProcess, OpenProcessToken},
        };

        struct OwnedToken(HANDLE);

        impl Drop for OwnedToken {
            fn drop(&mut self) {
                if !self.0.is_invalid() {
                    let _ = unsafe { CloseHandle(self.0) };
                }
            }
        }

        let mut token = HANDLE::default();
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
            .map_err(|error| native_error("OpenProcessToken", error))?;
        let token = OwnedToken(token);

        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        unsafe {
            GetTokenInformation(
                token.0,
                TokenElevation,
                Some((&raw mut elevation).cast()),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut returned,
            )
        }
        .map_err(|error| native_error("GetTokenInformation(TokenElevation)", error))?;

        let expected = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
        if returned < expected {
            return Err(NativeSecurityError::new(
                "GetTokenInformation(TokenElevation)",
                0,
                format!("native response contained {returned} bytes; expected {expected}"),
            ));
        }

        Ok(elevation.TokenIsElevated != 0)
    }
}

#[cfg(windows)]
fn native_error(stage: &'static str, error: windows::core::Error) -> NativeSecurityError {
    NativeSecurityError::new(stage, error.code().0 as u32, error.to_string())
}

#[cfg(not(windows))]
impl ElevationProbe for CurrentProcessElevationProbe {
    fn is_elevated(&self) -> Result<bool, NativeSecurityError> {
        Err(NativeSecurityError::new(
            "current_process_token",
            0,
            "Windows token inspection is unavailable on this platform",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{ElevationProbe, NativeSecurityError};

    struct FixedProbe(Result<bool, NativeSecurityError>);

    impl ElevationProbe for FixedProbe {
        fn is_elevated(&self) -> Result<bool, NativeSecurityError> {
            self.0.clone()
        }
    }

    #[test]
    fn adapter_preserves_elevated_and_non_elevated_outcomes() {
        assert!(FixedProbe(Ok(true)).is_elevated().expect("true outcome"));
        assert!(!FixedProbe(Ok(false)).is_elevated().expect("false outcome"));
    }

    #[test]
    fn adapter_preserves_native_stage_code_and_message() {
        let expected = NativeSecurityError::new("GetTokenInformation", 5, "access denied");
        let actual = FixedProbe(Err(expected.clone()))
            .is_elevated()
            .expect_err("native failure must remain visible");
        assert_eq!(actual, expected);
    }
}

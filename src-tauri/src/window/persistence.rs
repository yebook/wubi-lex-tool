//! Coalesced background persistence for normal window placement.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use crate::{config::WindowBounds, error::AppError};

const QUIET_PERIOD: Duration = Duration::from_millis(250);
const MAX_COALESCE_PERIOD: Duration = Duration::from_secs(2);
pub const EXIT_FLUSH_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, PartialEq)]
pub struct WindowPlacement {
    pub bounds: Option<WindowBounds>,
    pub maximized: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlushOutcome {
    Flushed,
    TimedOut,
    Disconnected,
}

enum WorkerMessage {
    Placement(WindowPlacement),
    FlushAndStop(Sender<()>),
}

struct PendingPlacement {
    placement: WindowPlacement,
    first_received_at: Instant,
    last_received_at: Instant,
}

#[derive(Debug)]
pub struct PlacementWorker {
    sender: Sender<WorkerMessage>,
    stopped: AtomicBool,
}

impl PlacementWorker {
    pub fn start(
        persist: impl Fn(WindowPlacement) -> Result<(), AppError> + Send + 'static,
        report_error: impl Fn(AppError) + Send + 'static,
    ) -> std::io::Result<Self> {
        Self::start_with_timing(persist, report_error, QUIET_PERIOD, MAX_COALESCE_PERIOD)
    }

    fn start_with_timing(
        persist: impl Fn(WindowPlacement) -> Result<(), AppError> + Send + 'static,
        report_error: impl Fn(AppError) + Send + 'static,
        quiet_period: Duration,
        max_coalesce_period: Duration,
    ) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .name("window-placement".to_owned())
            .spawn(move || {
                let mut pending: Option<PendingPlacement> = None;
                loop {
                    let message = if let Some(current) = pending.as_ref() {
                        let now = Instant::now();
                        let quiet_deadline = current.last_received_at + quiet_period;
                        let max_deadline = current.first_received_at + max_coalesce_period;
                        let deadline = quiet_deadline.min(max_deadline);
                        if now >= deadline {
                            persist_pending(&mut pending, &persist, &report_error);
                            continue;
                        }
                        receiver.recv_timeout(deadline.saturating_duration_since(now))
                    } else {
                        receiver.recv().map_err(|_| RecvTimeoutError::Disconnected)
                    };

                    match message {
                        Ok(WorkerMessage::Placement(placement)) => {
                            let now = Instant::now();
                            if let Some(current) = pending.as_mut() {
                                current.placement = placement;
                                current.last_received_at = now;
                            } else {
                                pending = Some(PendingPlacement {
                                    placement,
                                    first_received_at: now,
                                    last_received_at: now,
                                });
                            }
                        }
                        Ok(WorkerMessage::FlushAndStop(acknowledge)) => {
                            persist_pending(&mut pending, &persist, &report_error);
                            let _ = acknowledge.send(());
                            return;
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            persist_pending(&mut pending, &persist, &report_error);
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            persist_pending(&mut pending, &persist, &report_error);
                            return;
                        }
                    }
                }
            })?;
        Ok(Self {
            sender,
            stopped: AtomicBool::new(false),
        })
    }

    pub fn disabled() -> Self {
        let (sender, receiver) = mpsc::channel();
        drop(receiver);
        Self {
            sender,
            stopped: AtomicBool::new(true),
        }
    }

    pub fn schedule(&self, placement: WindowPlacement) -> bool {
        if self.stopped.load(Ordering::Acquire) {
            return false;
        }
        self.sender
            .send(WorkerMessage::Placement(placement))
            .is_ok()
    }

    pub fn flush_and_stop(&self, timeout: Duration) -> FlushOutcome {
        if self.stopped.swap(true, Ordering::AcqRel) {
            return FlushOutcome::Disconnected;
        }
        let (acknowledge, completion) = mpsc::channel();
        if self
            .sender
            .send(WorkerMessage::FlushAndStop(acknowledge))
            .is_err()
        {
            return FlushOutcome::Disconnected;
        }
        match completion.recv_timeout(timeout) {
            Ok(()) => FlushOutcome::Flushed,
            Err(RecvTimeoutError::Timeout) => FlushOutcome::TimedOut,
            Err(RecvTimeoutError::Disconnected) => FlushOutcome::Disconnected,
        }
    }
}

fn persist_pending(
    pending: &mut Option<PendingPlacement>,
    persist: &impl Fn(WindowPlacement) -> Result<(), AppError>,
    report_error: &impl Fn(AppError),
) {
    if let Some(pending) = pending.take()
        && let Err(error) = persist(pending.placement)
    {
        report_error(error);
    }
}

#[cfg(test)]
mod tests {
    use super::{FlushOutcome, PlacementWorker, WindowPlacement};
    use crate::{
        config::WindowBounds,
        error::{AppError, AppErrorCode, AppErrorKind},
    };
    use std::{
        sync::{Arc, Mutex, mpsc},
        thread,
        time::Duration,
    };

    fn placement(x: i32) -> WindowPlacement {
        WindowPlacement {
            bounds: Some(WindowBounds {
                x,
                y: 20,
                width: 1_024,
                height: 640,
                scale_factor: 1.0,
            }),
            maximized: false,
        }
    }

    #[test]
    fn quiet_period_persists_only_the_latest_placement() {
        let (saved, received) = mpsc::channel();
        let worker = PlacementWorker::start_with_timing(
            move |placement| saved.send(placement).map_err(test_error),
            |_| {},
            Duration::from_millis(15),
            Duration::from_millis(100),
        )
        .expect("worker thread");
        assert!(worker.schedule(placement(10)));
        assert!(worker.schedule(placement(20)));

        assert_eq!(
            received
                .recv_timeout(Duration::from_millis(200))
                .expect("coalesced placement"),
            placement(20)
        );
        assert!(received.try_recv().is_err());
        assert_eq!(
            worker.flush_and_stop(Duration::from_millis(100)),
            FlushOutcome::Flushed
        );
    }

    #[test]
    fn maximum_period_flushes_during_continuous_updates() {
        let (saved, received) = mpsc::channel();
        let worker = PlacementWorker::start_with_timing(
            move |placement| saved.send(placement).map_err(test_error),
            |_| {},
            Duration::from_millis(80),
            Duration::from_millis(35),
        )
        .expect("worker thread");
        for x in 0..4 {
            assert!(worker.schedule(placement(x)));
            thread::sleep(Duration::from_millis(10));
        }

        let persisted = received
            .recv_timeout(Duration::from_millis(150))
            .expect("maximum period placement");
        assert!(persisted.bounds.expect("bounds").x >= 2);
        assert_eq!(
            worker.flush_and_stop(Duration::from_millis(100)),
            FlushOutcome::Flushed
        );
    }

    #[test]
    fn flush_reports_errors_but_still_stops() {
        let errors = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&errors);
        let worker = PlacementWorker::start_with_timing(
            |_| Err(test_error("write failed")),
            move |error| observed.lock().expect("errors").push(error.code),
            Duration::from_secs(1),
            Duration::from_secs(2),
        )
        .expect("worker thread");
        assert!(worker.schedule(placement(1)));
        assert_eq!(
            worker.flush_and_stop(Duration::from_millis(100)),
            FlushOutcome::Flushed
        );
        assert_eq!(
            *errors.lock().expect("errors"),
            vec![AppErrorCode::ConfigWriteFailed]
        );
        assert!(!worker.schedule(placement(2)));
    }

    fn test_error(detail: impl std::fmt::Display) -> AppError {
        AppError::config(
            AppErrorCode::ConfigWriteFailed,
            AppErrorKind::Io,
            "配置写入失败。",
            Some(detail.to_string()),
            true,
        )
    }
}

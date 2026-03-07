use std::io;
use std::process::{Child, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum TimeoutOutcome {
    Completed(ExitStatus),
    TimedOut,
}

pub fn resolve_timeout_ms(
    mode: sc_hooks_core::dispatch::DispatchMode,
    timeout_override: Option<u64>,
    long_running: bool,
) -> Option<u64> {
    if long_running {
        return timeout_override;
    }

    match timeout_override {
        Some(ms) => Some(ms),
        None => match mode {
            sc_hooks_core::dispatch::DispatchMode::Sync => Some(5_000),
            sc_hooks_core::dispatch::DispatchMode::Async => Some(30_000),
        },
    }
}

pub fn wait_with_timeout(child: &mut Child, timeout_ms: Option<u64>) -> io::Result<TimeoutOutcome> {
    let Some(timeout_ms) = timeout_ms else {
        return child.wait().map(TimeoutOutcome::Completed);
    };

    let timeout = Duration::from_millis(timeout_ms);
    let start = Instant::now();

    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(TimeoutOutcome::Completed(status));
        }

        if start.elapsed() >= timeout {
            terminate_then_kill(child)?;
            return Ok(TimeoutOutcome::TimedOut);
        }

        thread::sleep(Duration::from_millis(25));
    }
}

fn terminate_then_kill(child: &mut Child) -> io::Result<()> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid;

        if let Ok(raw_pid) = i32::try_from(child.id()) {
            let _ = kill(Pid::from_raw(raw_pid), Signal::SIGTERM);
        }
    }

    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }

    let grace_start = Instant::now();
    let grace = Duration::from_secs(1);
    loop {
        if child.try_wait()?.is_some() {
            return Ok(());
        }

        if grace_start.elapsed() >= grace {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(());
        }

        thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_mode() {
        assert_eq!(
            resolve_timeout_ms(sc_hooks_core::dispatch::DispatchMode::Sync, None, false),
            Some(5_000)
        );
        assert_eq!(
            resolve_timeout_ms(sc_hooks_core::dispatch::DispatchMode::Async, None, false),
            Some(30_000)
        );
    }

    #[test]
    fn long_running_without_override_has_no_timeout() {
        assert_eq!(
            resolve_timeout_ms(sc_hooks_core::dispatch::DispatchMode::Sync, None, true),
            None
        );
    }

    #[test]
    fn override_wins() {
        assert_eq!(
            resolve_timeout_ms(
                sc_hooks_core::dispatch::DispatchMode::Sync,
                Some(1234),
                false
            ),
            Some(1234)
        );
    }
}

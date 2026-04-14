//! Shared process-spawn helpers used across the host and SDK.

use std::io;
use std::thread;
use std::time::Duration;

const EXECUTABLE_FILE_BUSY_RETRY_ATTEMPTS: usize = 3;
const EXECUTABLE_FILE_BUSY_RETRY_DELAY: Duration = Duration::from_millis(20);

/// Retries a process operation when the operating system reports
/// [`io::ErrorKind::ExecutableFileBusy`].
///
/// This centralizes the short bounded retry used when a freshly written script
/// is executed immediately after creation and the platform still reports the
/// executable as busy.
pub fn retry_executable_file_busy<T>(
    mut operation: impl FnMut() -> io::Result<T>,
) -> io::Result<T> {
    let mut last_err = None;
    for attempt in 0..EXECUTABLE_FILE_BUSY_RETRY_ATTEMPTS {
        match operation() {
            Ok(value) => return Ok(value),
            Err(err) if err.kind() == io::ErrorKind::ExecutableFileBusy => {
                if attempt + 1 == EXECUTABLE_FILE_BUSY_RETRY_ATTEMPTS {
                    return Err(err);
                }
                last_err = Some(err);
                thread::sleep(EXECUTABLE_FILE_BUSY_RETRY_DELAY);
            }
            Err(err) => return Err(err),
        }
    }

    // INVARIANT: the only fallthrough path is a retryable
    // `ExecutableFileBusy` error, which stores the latest error in `last_err`
    // before sleeping.
    Err(last_err.expect("executable-file-busy retry loop should capture the final error"))
}

#[cfg(test)]
mod tests {
    use super::retry_executable_file_busy;
    use std::io;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn retries_executable_file_busy_before_success() {
        let attempts = AtomicUsize::new(0);
        let result = retry_executable_file_busy(|| {
            if attempts.fetch_add(1, Ordering::SeqCst) < 2 {
                Err(io::Error::from(io::ErrorKind::ExecutableFileBusy))
            } else {
                Ok("ok")
            }
        })
        .expect("retry should eventually succeed");

        assert_eq!(result, "ok");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn returns_non_retryable_error_immediately() {
        let attempts = AtomicUsize::new(0);
        let err = retry_executable_file_busy(|| {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(io::Error::from(io::ErrorKind::PermissionDenied))
        })
        .expect_err("non-retryable error should be returned");

        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}

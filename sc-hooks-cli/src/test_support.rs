use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn cwd_lock() -> &'static Mutex<()> {
    CWD_LOCK.get_or_init(|| Mutex::new(()))
}

pub struct CurrentDirGuard {
    _lock: MutexGuard<'static, ()>,
    original: PathBuf,
}

pub fn scoped_current_dir(path: &Path) -> CurrentDirGuard {
    let lock = cwd_lock().lock().unwrap_or_else(|e| e.into_inner());
    let original = std::env::current_dir().expect("cwd should resolve");
    std::env::set_current_dir(path).expect("cwd should switch");
    CurrentDirGuard {
        _lock: lock,
        original,
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        if let Err(err) = std::env::set_current_dir(&self.original)
            && !std::thread::panicking()
        {
            panic!("cwd should restore: {err}");
        }
    }
}

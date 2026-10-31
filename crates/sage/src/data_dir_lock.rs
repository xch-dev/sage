use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions, TryLockError},
    io,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, Mutex, PoisonError, Weak},
};

use crate::{Error, Result};

/// Locks held by this process, keyed by data directory.
static HELD_LOCKS: LazyLock<Mutex<HashMap<PathBuf, Weak<File>>>> = LazyLock::new(Mutex::default);

/// An exclusive lock on the data directory, held for as long as Sage runs.
///
/// This prevents multiple processes (GUI instances, or the GUI alongside `sage rpc start`) from
/// using the same data directory. Each would otherwise keep its own copy of the keychain and
/// config in memory, and overwrite the other's changes when saving.
///
/// The OS releases the lock when the file is closed, including if the process crashes, so a stale
/// lock can't block startup. The lock file is never deleted, since deleting it while another
/// process is waiting to lock it would let two processes hold separate locks.
///
/// Instances within the same process share the lock. On Android, the app's setup runs again
/// whenever the activity is recreated, while the previous instance is still alive.
#[derive(Debug)]
pub(crate) enum DataDirLock {
    Held {
        _file: Arc<File>,
    },
    /// The filesystem doesn't support locking, so Sage runs without the lock.
    Unsupported(io::Error),
}

impl DataDirLock {
    pub(crate) fn acquire(path: &Path) -> Result<Self> {
        let key = fs::canonicalize(path)?;
        let mut held_locks = HELD_LOCKS.lock().unwrap_or_else(PoisonError::into_inner);

        held_locks.retain(|_, file| file.strong_count() > 0);

        if let Some(file) = held_locks.get(&key).and_then(Weak::upgrade) {
            return Ok(Self::Held { _file: file });
        }

        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(key.join("sage.lock"))?;

        match file.try_lock() {
            Ok(()) => {
                let file = Arc::new(file);
                held_locks.insert(key, Arc::downgrade(&file));
                Ok(Self::Held { _file: file })
            }
            Err(TryLockError::WouldBlock) => Err(Error::AlreadyRunning),
            // Refusing to start would be worse than running without the lock.
            Err(TryLockError::Error(error)) if error.kind() == io::ErrorKind::Unsupported => {
                Ok(Self::Unsupported(error))
            }
            Err(TryLockError::Error(error)) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn is_locked_by_another_process(path: &Path) -> Result<bool> {
        // Separate open files conflict even within a process, like separate processes do
        let file = File::open(path.join("sage.lock"))?;
        Ok(matches!(file.try_lock(), Err(TryLockError::WouldBlock)))
    }

    #[test]
    fn test_lock_is_exclusive_and_shared_within_process() -> Result<()> {
        let dir = TempDir::new()?;

        let first = DataDirLock::acquire(dir.path())?;
        let second = DataDirLock::acquire(dir.path())?;
        assert!(is_locked_by_another_process(dir.path())?);

        drop(first);
        assert!(is_locked_by_another_process(dir.path())?);

        drop(second);
        assert!(!is_locked_by_another_process(dir.path())?);

        Ok(())
    }
}

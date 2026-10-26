use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions, TryLockError},
    io,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError, Weak},
};

use crate::{Error, Result};

/// Locks currently held by this process, keyed by data directory.
static HELD_LOCKS: LazyLock<Mutex<HashMap<PathBuf, Weak<SharedLock>>>> =
    LazyLock::new(Mutex::default);

/// An exclusive lock on the data directory, held for as long as Sage runs.
///
/// This prevents multiple processes (GUI instances, or the GUI alongside `sage rpc start`) from
/// using the same data directory. Each would otherwise keep its own in-memory copy of the
/// keychain and config, and overwrite the other's changes when saving.
///
/// The lock is released by the OS when the file is closed, including if the process crashes, so
/// a stale lock can't block startup. The lock file itself is never deleted, since deleting it
/// while another process is waiting to lock it would let two processes hold separate locks.
///
/// Within a single process, a new instance takes the lock over from the previous one instead.
/// On Android, the app's setup runs again whenever the activity is recreated, while the previous
/// instance is still alive in background tasks. The previous instance stops being writable.
/// Writes are serialized within the process, so a takeover waits for any write in progress.
#[derive(Debug)]
pub(crate) struct DataDirLock {
    shared: Arc<SharedLock>,
    generation: u64,
}

#[derive(Debug)]
struct SharedLock {
    file: FileLock,
    /// The generation of the instance that currently owns the data directory.
    current_generation: Mutex<u64>,
}

#[derive(Debug)]
pub(crate) enum FileLock {
    Held {
        _file: File,
    },
    /// The filesystem doesn't support locking, so Sage runs without the lock.
    Unsupported(io::Error),
}

impl DataDirLock {
    pub(crate) fn acquire(path: &Path) -> Result<Self> {
        let key = fs::canonicalize(path)?;
        let mut held_locks = lock(&HELD_LOCKS);

        held_locks.retain(|_, shared| shared.strong_count() > 0);

        if let Some(shared) = held_locks.get(&key).and_then(Weak::upgrade) {
            let mut current_generation = lock(&shared.current_generation);
            *current_generation += 1;
            let generation = *current_generation;
            drop(current_generation);

            return Ok(Self { shared, generation });
        }

        let shared = Arc::new(SharedLock {
            file: FileLock::acquire(&key)?,
            current_generation: Mutex::new(0),
        });

        held_locks.insert(key, Arc::downgrade(&shared));

        Ok(Self {
            shared,
            generation: 0,
        })
    }

    /// Whether this is still the instance that owns the data directory.
    #[cfg(test)]
    fn is_current(&self) -> bool {
        *lock(&self.shared.current_generation) == self.generation
    }

    /// Returns a guard to hold while writing to the data directory, or `None` if another instance
    /// in this process has taken it over.
    pub(crate) fn write_guard(&self) -> Option<MutexGuard<'_, u64>> {
        let guard = lock(&self.shared.current_generation);
        (*guard == self.generation).then_some(guard)
    }

    pub(crate) fn file_lock(&self) -> &FileLock {
        &self.shared.file
    }
}

impl FileLock {
    fn acquire(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path.join("sage.lock"))?;

        match file.try_lock() {
            Ok(()) => Ok(Self::Held { _file: file }),
            Err(TryLockError::WouldBlock) => Err(Error::AlreadyRunning),
            // Some filesystems (such as certain network mounts) don't support locking.
            // Refusing to start there would be worse than running without the lock.
            Err(TryLockError::Error(error)) if is_lock_unsupported(&error) => {
                Ok(Self::Unsupported(error))
            }
            Err(TryLockError::Error(error)) => Err(error.into()),
        }
    }
}

/// Nothing that could panic runs while these mutexes are held, so poisoning can be ignored.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Whether a locking error means the filesystem can't lock files, rather than a real failure.
/// Several of these codes aren't mapped to [`io::ErrorKind::Unsupported`] by the standard library.
fn is_lock_unsupported(error: &io::Error) -> bool {
    if error.kind() == io::ErrorKind::Unsupported {
        return true;
    }

    let Some(code) = error.raw_os_error() else {
        return false;
    };

    #[cfg(unix)]
    {
        // NFS without a lock daemon reports `ENOLCK`, and macOS reports `ENOTSUP`.
        [libc::ENOLCK, libc::ENOTSUP, libc::EOPNOTSUPP, libc::ENOSYS].contains(&code)
    }

    #[cfg(windows)]
    {
        // `ERROR_INVALID_FUNCTION` and `ERROR_NOT_SUPPORTED`, returned by some network redirectors.
        [1, 50].contains(&code)
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = code;
        false
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_file_lock_is_exclusive() -> Result<()> {
        let dir = TempDir::new()?;

        // Separate open files conflict even within a process, like separate processes do
        let lock = FileLock::acquire(dir.path())?;
        assert!(matches!(
            FileLock::acquire(dir.path()),
            Err(Error::AlreadyRunning)
        ));

        drop(lock);
        FileLock::acquire(dir.path())?;

        Ok(())
    }

    #[test]
    fn test_lock_is_taken_over_within_process() -> Result<()> {
        let dir = TempDir::new()?;

        let first = DataDirLock::acquire(dir.path())?;
        assert!(first.is_current());

        let second = DataDirLock::acquire(dir.path())?;
        assert!(second.is_current());
        assert!(!first.is_current());
        assert!(first.write_guard().is_none());
        assert!(second.write_guard().is_some());

        // The file lock stays held until every instance is gone
        drop(second);
        assert!(matches!(
            FileLock::acquire(dir.path()),
            Err(Error::AlreadyRunning)
        ));

        drop(first);
        FileLock::acquire(dir.path())?;

        Ok(())
    }
}

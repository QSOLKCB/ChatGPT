use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use thiserror::Error;

const AUTHORITY_LOCK_NAME: &str = "qsol-chatgpt-authority.lock";

pub struct AuthorityInstanceGuard {
    _file: File,
    path: PathBuf,
}

impl AuthorityInstanceGuard {
    pub fn acquire() -> Result<Self, InstanceError> {
        let runtime_dir = validated_runtime_directory()?;
        Self::acquire_in(&runtime_dir)
    }

    pub(crate) fn acquire_in(runtime_dir: &Path) -> Result<Self, InstanceError> {
        let path = runtime_dir.join(AUTHORITY_LOCK_NAME);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true).mode(0o600);
        let mut file = match options.open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                return Err(InstanceError::AlreadyRunning);
            }
            Err(_) => return Err(InstanceError::LockCreateFailed),
        };

        if writeln!(file, "pid={}", std::process::id()).is_err() {
            let _ = fs::remove_file(&path);
            return Err(InstanceError::LockWriteFailed);
        }
        if file.sync_all().is_err() {
            let _ = fs::remove_file(&path);
            return Err(InstanceError::LockWriteFailed);
        }

        Ok(Self { _file: file, path })
    }
}

impl Drop for AuthorityInstanceGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InstanceError {
    #[error("cannot determine the current Linux user id")]
    UserIdUnavailable,
    #[error("XDG_RUNTIME_DIR is not the expected /run/user/<uid> directory")]
    UnexpectedRuntimeDirectory,
    #[error("runtime directory ownership or permissions are unsafe")]
    UnsafeRuntimeDirectory,
    #[error("another authority-bearing QSOL ChatGPT instance is already running, or a stale fail-closed lock remains")]
    AlreadyRunning,
    #[error("failed to create the authority-instance lock")]
    LockCreateFailed,
    #[error("failed to initialize the authority-instance lock")]
    LockWriteFailed,
}

pub(crate) fn validated_runtime_directory() -> Result<PathBuf, InstanceError> {
    let uid = current_uid()?;
    let expected_runtime = PathBuf::from(format!("/run/user/{uid}"));
    let runtime_dir = match env::var_os("XDG_RUNTIME_DIR") {
        Some(value) => PathBuf::from(value),
        None => expected_runtime.clone(),
    };

    if runtime_dir != expected_runtime {
        return Err(InstanceError::UnexpectedRuntimeDirectory);
    }
    validate_runtime_directory(&runtime_dir, uid)?;
    Ok(runtime_dir)
}

fn current_uid() -> Result<u32, InstanceError> {
    let status = fs::read_to_string("/proc/self/status")
        .map_err(|_| InstanceError::UserIdUnavailable)?;
    let uid_line = status
        .lines()
        .find(|line| line.starts_with("Uid:"))
        .ok_or(InstanceError::UserIdUnavailable)?;
    uid_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or(InstanceError::UserIdUnavailable)
}

fn validate_runtime_directory(path: &Path, uid: u32) -> Result<(), InstanceError> {
    let metadata = fs::metadata(path).map_err(|_| InstanceError::UnsafeRuntimeDirectory)?;
    if !metadata.is_dir() || metadata.uid() != uid {
        return Err(InstanceError::UnsafeRuntimeDirectory);
    }
    let mode = metadata.permissions().mode();
    if mode & 0o077 != 0 {
        return Err(InstanceError::UnsafeRuntimeDirectory);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(label: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "qsol-chatgpt-{label}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn second_authority_instance_fails_closed() {
        let dir = test_dir("instance-lock");
        let _ = fs::remove_dir_all(&dir);
        if let Err(error) = fs::create_dir(&dir) {
            panic!("failed to create fixture directory: {error}");
        }

        let first = AuthorityInstanceGuard::acquire_in(&dir);
        if first.is_err() {
            let _ = fs::remove_dir_all(&dir);
            panic!("first authority lock should succeed");
        }
        let second = AuthorityInstanceGuard::acquire_in(&dir);
        assert!(matches!(second, Err(InstanceError::AlreadyRunning)));

        drop(first);
        let third = AuthorityInstanceGuard::acquire_in(&dir);
        assert!(third.is_ok());
        drop(third);
        let _ = fs::remove_dir_all(&dir);
    }
}

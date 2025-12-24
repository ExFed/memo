//! Cache management for memoized command execution
//!
//! This module handles all file I/O operations for the memo cache, including:
//! - Cache directory management
//! - File path generation
//! - Memo metadata storage and retrieval
//! - Output streaming
//! - Lock acquisition for concurrent safety
//!
//! # Storage Structure
//!
//! Each memoized command produces three files:
//! - `<digest>.json` - Metadata (command, exit code, timestamp, digest)
//! - `<digest>.out` - Raw stdout bytes
//! - `<digest>.err` - Raw stderr bytes

use crate::constants::{CACHE_DIR_PERMISSIONS, FILE_PERMISSIONS};
use crate::error::{MemoError, Result};
use crate::memo::Memo;
use std::fs::{self, File, OpenOptions};
use std::io::{self, copy};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

/// Check if memoization is disabled via environment variable
///
/// Returns `true` if `MEMO_DISABLE=1`, otherwise `false`.
///
/// # Examples
///
/// ```no_run
/// # use memo::cache::is_memo_disabled;
/// if is_memo_disabled() {
///     println!("Memoization is disabled");
/// }
/// ```
pub fn is_memo_disabled() -> bool {
    std::env::var("MEMO_DISABLE")
        .map(|val| val == "1")
        .unwrap_or(false)
}

/// Get the cache directory path
///
/// Respects `$XDG_CACHE_HOME` environment variable, falling back to `~/.cache`.
///
/// # Examples
///
/// ```no_run
/// # use memo::cache::get_cache_dir;
/// let cache_dir = get_cache_dir().expect("Failed to get cache directory");
/// println!("Cache directory: {:?}", cache_dir);
/// ```
pub fn get_cache_dir() -> Result<PathBuf> {
    let base = if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg)
    } else {
        dirs::home_dir()
            .ok_or(MemoError::HomeNotFound)?
            .join(".cache")
    };
    Ok(base.join("memo"))
}

/// Ensure the cache directory exists with appropriate permissions
///
/// Creates the directory if it doesn't exist, and sets secure permissions (0o700)
/// on Unix systems.
pub fn ensure_cache_dir(cache_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(cache_dir)?;

    #[cfg(unix)]
    {
        let perm = fs::Permissions::from_mode(CACHE_DIR_PERMISSIONS);
        let _ = fs::set_permissions(cache_dir, perm);
    }

    Ok(())
}

/// Check if a memo is complete (all three cache files exist)
///
/// Returns `true` if the `.json`, `.out`, and `.err` files all exist.
pub fn memo_complete(cache_dir: &Path, digest: &str) -> bool {
    let (json_path, out_path, err_path) = get_cache_paths(cache_dir, digest);
    json_path.exists() && out_path.exists() && err_path.exists()
}

pub fn purge_memo(cache_dir: &Path, digest: &str) {
    let (json_path, out_path, err_path) = get_cache_paths(cache_dir, digest);
    let _ = fs::remove_file(json_path);
    let _ = fs::remove_file(out_path);
    let _ = fs::remove_file(err_path);
}

pub fn get_cache_paths(cache_dir: &Path, digest: &str) -> (PathBuf, PathBuf, PathBuf) {
    let json_path = cache_dir.join(format!("{}.json", digest));
    let out_path = cache_dir.join(format!("{}.out", digest));
    let err_path = cache_dir.join(format!("{}.err", digest));
    (json_path, out_path, err_path)
}

/// Create a new file with secure permissions (owner read/write only)
///
/// This helper ensures consistent file creation across the codebase with
/// appropriate security permissions on Unix systems.
pub fn create_secure_file(path: &Path) -> io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);

    #[cfg(unix)]
    {
        opts.mode(FILE_PERMISSIONS);
    }

    opts.open(path)
}

pub struct CacheLock {
    path: PathBuf,
}

impl Drop for CacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn try_acquire_lock(cache_dir: &Path, digest: &str) -> io::Result<CacheLock> {
    let lock_path = cache_dir.join(format!("{}.lock", digest));
    let _file = create_secure_file(&lock_path)?;
    Ok(CacheLock { path: lock_path })
}

#[cfg(test)]
pub fn write_memo(
    cache_dir: &Path,
    digest: &str,
    memo: &Memo,
    stdout: &[u8],
    stderr: &[u8],
) -> io::Result<()> {
    let (json_path, out_path, err_path) = get_cache_paths(cache_dir, digest);

    let json = serde_json::to_string_pretty(memo)?;
    fs::write(json_path, json)?;
    fs::write(out_path, stdout)?;
    fs::write(err_path, stderr)?;

    Ok(())
}

#[cfg(test)]
pub fn read_memo(cache_dir: &Path, digest: &str) -> io::Result<(Memo, Vec<u8>, Vec<u8>)> {
    let (json_path, out_path, err_path) = get_cache_paths(cache_dir, digest);

    let json = fs::read_to_string(json_path)?;
    let memo: Memo = serde_json::from_str(&json)?;
    let stdout = fs::read(out_path)?;
    let stderr = fs::read(err_path)?;

    Ok((memo, stdout, stderr))
}

/// Stream cached stdout to the given writer
pub fn stream_stdout<W: io::Write>(
    cache_dir: &Path,
    digest: &str,
    mut writer: W,
) -> io::Result<()> {
    let (_, out_path, _) = get_cache_paths(cache_dir, digest);
    let mut file = File::open(out_path)?;
    copy(&mut file, &mut writer)?;
    Ok(())
}

/// Stream cached stderr to the given writer
pub fn stream_stderr<W: io::Write>(
    cache_dir: &Path,
    digest: &str,
    mut writer: W,
) -> io::Result<()> {
    let (_, _, err_path) = get_cache_paths(cache_dir, digest);
    let mut file = File::open(err_path)?;
    copy(&mut file, &mut writer)?;
    Ok(())
}

/// Read just the memo metadata without loading output files
pub fn read_memo_metadata(cache_dir: &Path, digest: &str) -> io::Result<Memo> {
    let (json_path, _, _) = get_cache_paths(cache_dir, digest);
    let json = fs::read_to_string(json_path)?;
    let memo: Memo = serde_json::from_str(&json)?;
    Ok(memo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_cache() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("memo");
        (temp_dir, cache_dir)
    }

    #[test]
    fn test_ensure_cache_dir_creates_directory() {
        let (_temp, cache_dir) = setup_test_cache();

        assert!(!cache_dir.exists());
        ensure_cache_dir(&cache_dir).unwrap();
        assert!(cache_dir.exists());
        assert!(cache_dir.is_dir());
    }

    #[test]
    fn test_ensure_cache_dir_idempotent() {
        let (_temp, cache_dir) = setup_test_cache();

        ensure_cache_dir(&cache_dir).unwrap();
        ensure_cache_dir(&cache_dir).unwrap(); // Should not error
        assert!(cache_dir.exists());
    }

    #[test]
    fn test_write_and_read_memo() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let digest = "abc123";
        let memo = Memo {
            cmd: vec!["echo".to_string(), "test".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest.to_string(),
        };
        let stdout = b"test output\n";
        let stderr = b"test error\n";

        write_memo(&cache_dir, digest, &memo, stdout, stderr).unwrap();

        let (read_memo, read_stdout, read_stderr) = read_memo(&cache_dir, digest).unwrap();

        assert_eq!(read_memo.cmd, memo.cmd);
        assert_eq!(read_memo.exit_code, memo.exit_code);
        assert_eq!(read_memo.digest, memo.digest);
        assert_eq!(read_stdout, stdout);
        assert_eq!(read_stderr, stderr);
    }

    #[test]
    fn test_write_memo_empty_output() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let digest = "empty123";
        let memo = Memo {
            cmd: vec!["true".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest.to_string(),
        };

        write_memo(&cache_dir, digest, &memo, b"", b"").unwrap();

        let (_, stdout, stderr) = read_memo(&cache_dir, digest).unwrap();
        assert_eq!(stdout, b"");
        assert_eq!(stderr, b"");
    }

    #[test]
    fn test_write_memo_binary_data() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let digest = "binary123";
        let memo = Memo {
            cmd: vec!["binary".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest.to_string(),
        };
        let binary_data = vec![0x00, 0x01, 0xFF, 0xFE, 0x7F];

        write_memo(&cache_dir, digest, &memo, &binary_data, &binary_data).unwrap();

        let (_, stdout, stderr) = read_memo(&cache_dir, digest).unwrap();
        assert_eq!(stdout, binary_data);
        assert_eq!(stderr, binary_data);
    }

    #[test]
    fn test_read_nonexistent_memo() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let result = read_memo(&cache_dir, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_memos() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let digest1 = "multi1";
        let digest2 = "multi2";

        let memo1 = Memo {
            cmd: vec!["echo".to_string(), "one".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest1.to_string(),
        };

        let memo2 = Memo {
            cmd: vec!["echo".to_string(), "two".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 1,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest2.to_string(),
        };

        write_memo(&cache_dir, digest1, &memo1, b"one\n", b"").unwrap();
        write_memo(&cache_dir, digest2, &memo2, b"two\n", b"err\n").unwrap();

        let (read1, out1, err1) = read_memo(&cache_dir, digest1).unwrap();
        let (read2, out2, err2) = read_memo(&cache_dir, digest2).unwrap();

        assert_eq!(read1.cmd, vec!["echo", "one"]);
        assert_eq!(read2.cmd, vec!["echo", "two"]);
        assert_eq!(out1, b"one\n");
        assert_eq!(out2, b"two\n");
        assert_eq!(err1, b"");
        assert_eq!(err2, b"err\n");
    }

    #[test]
    fn test_cache_files_have_correct_names() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let digest = "names123";
        let memo = Memo {
            cmd: vec!["test".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest.to_string(),
        };

        write_memo(&cache_dir, digest, &memo, b"out", b"err").unwrap();

        assert!(cache_dir.join(format!("{}.json", digest)).exists());
        assert!(cache_dir.join(format!("{}.out", digest)).exists());
        assert!(cache_dir.join(format!("{}.err", digest)).exists());
    }

    #[test]
    fn test_get_cache_dir_respects_xdg() {
        let temp = TempDir::new().unwrap();
        let xdg_path = temp.path().to_path_buf();

        std::env::set_var("XDG_CACHE_HOME", &xdg_path);
        let cache_dir = get_cache_dir().unwrap();
        std::env::remove_var("XDG_CACHE_HOME");

        assert_eq!(cache_dir, xdg_path.join("memo"));
    }

    #[test]
    fn test_large_output() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let digest = "large123";
        let memo = Memo {
            cmd: vec!["large".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest.to_string(),
        };

        // Create 1MB of output
        let large_output = vec![b'A'; 1024 * 1024];

        write_memo(&cache_dir, digest, &memo, &large_output, b"").unwrap();

        let (_, stdout, _) = read_memo(&cache_dir, digest).unwrap();
        assert_eq!(stdout.len(), 1024 * 1024);
        assert_eq!(stdout, large_output);
    }

    #[test]
    fn test_stream_stdout() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let digest = "stream123";
        let memo = Memo {
            cmd: vec!["test".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest.to_string(),
        };

        write_memo(&cache_dir, digest, &memo, b"output data", b"error data").unwrap();

        let mut output = Vec::new();
        stream_stdout(&cache_dir, digest, &mut output).unwrap();
        assert_eq!(output, b"output data");
    }

    #[test]
    fn test_stream_stderr() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let digest = "stream456";
        let memo = Memo {
            cmd: vec!["test".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 0,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest.to_string(),
        };

        write_memo(&cache_dir, digest, &memo, b"output data", b"error data").unwrap();

        let mut errors = Vec::new();
        stream_stderr(&cache_dir, digest, &mut errors).unwrap();
        assert_eq!(errors, b"error data");
    }

    #[test]
    fn test_read_memo_metadata() {
        let (_temp, cache_dir) = setup_test_cache();
        ensure_cache_dir(&cache_dir).unwrap();

        let digest = "meta123";
        let memo = Memo {
            cmd: vec!["echo".to_string(), "test".to_string()],
            cwd: "/test/dir".to_string(),
            exit_code: 42,
            timestamp: "2025-12-22T01:51:52.369Z".to_string(),
            digest: digest.to_string(),
        };

        write_memo(&cache_dir, digest, &memo, b"large output here", b"errors").unwrap();

        let read_meta = read_memo_metadata(&cache_dir, digest).unwrap();
        assert_eq!(read_meta.cmd, vec!["echo", "test"]);
        assert_eq!(read_meta.exit_code, 42);
        assert_eq!(read_meta.digest, digest);
    }

    #[test]
    fn test_get_cache_paths() {
        let path = PathBuf::from("/tmp/cache");
        let (json, out, err) = get_cache_paths(&path, "abc123");

        assert_eq!(json, PathBuf::from("/tmp/cache/abc123.json"));
        assert_eq!(out, PathBuf::from("/tmp/cache/abc123.out"));
        assert_eq!(err, PathBuf::from("/tmp/cache/abc123.err"));
    }
}

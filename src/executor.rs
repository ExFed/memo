//! Command execution with output streaming
//!
//! This module handles the execution of shell commands and streaming their output
//! directly to cache files. This avoids loading large outputs into memory.

use crate::constants::FILE_PERMISSIONS;
use crate::error::{MemoError, Result};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

/// Result of command execution
pub struct ExecutionResult {
    /// The exit code returned by the command
    pub exit_code: i32,
}

/// Builder for command execution with output streaming
///
/// Provides a fluent API for configuring and executing commands.
///
/// # Examples
///
/// ```no_run
/// # use memo::executor::CommandExecutor;
/// # use std::path::Path;
/// let result = CommandExecutor::new()
///     .args(&["echo", "hello"])
///     .stdout_path(Path::new("/tmp/out.txt"))
///     .stderr_path(Path::new("/tmp/err.txt"))
///     .execute()
///     .expect("Command failed");
/// assert_eq!(result.exit_code, 0);
/// ```
#[allow(dead_code)] // Public API, used in tests
pub struct CommandExecutor<'a> {
    args: Option<&'a [&'a str]>,
    stdout_path: Option<&'a Path>,
    stderr_path: Option<&'a Path>,
}

#[allow(dead_code)] // Public API, used in tests
impl<'a> CommandExecutor<'a> {
    /// Create a new command executor builder
    pub fn new() -> Self {
        Self {
            args: None,
            stdout_path: None,
            stderr_path: None,
        }
    }

    /// Set the command and arguments
    pub fn args(mut self, args: &'a [&'a str]) -> Self {
        self.args = Some(args);
        self
    }

    /// Set the path for stdout output
    pub fn stdout_path(mut self, path: &'a Path) -> Self {
        self.stdout_path = Some(path);
        self
    }

    /// Set the path for stderr output
    pub fn stderr_path(mut self, path: &'a Path) -> Self {
        self.stderr_path = Some(path);
        self
    }

    /// Execute the command with the configured parameters
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Args, stdout_path, or stderr_path are not set
    /// - Command execution fails
    pub fn execute(self) -> Result<ExecutionResult> {
        let args = self
            .args
            .ok_or_else(|| MemoError::InvalidCommand("No command provided".to_string()))?;
        let stdout_path = self
            .stdout_path
            .ok_or_else(|| MemoError::InvalidCommand("No stdout path provided".to_string()))?;
        let stderr_path = self
            .stderr_path
            .ok_or_else(|| MemoError::InvalidCommand("No stderr path provided".to_string()))?;

        execute_and_stream(args, stdout_path, stderr_path)
    }
}

impl<'a> Default for CommandExecutor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a display string from command arguments
///
/// Joins arguments with spaces for user-friendly display.
///
/// # Examples
///
/// ```
/// # use memo::executor::build_command_string;
/// let args = vec!["echo".to_string(), "hello".to_string()];
/// assert_eq!(build_command_string(&args), "echo hello");
/// ```
pub fn build_command_string(args: &[String]) -> String {
    args.join(" ")
}

/// Create a new file with secure permissions (owner read/write only)
fn create_secure_file(path: &Path) -> std::io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);

    #[cfg(unix)]
    {
        opts.mode(FILE_PERMISSIONS);
    }

    opts.open(path)
}

/// Execute a command and stream its output directly to files
///
/// This function creates the output files with secure permissions and streams
/// stdout and stderr directly from the command without buffering in memory.
///
/// # Arguments
///
/// * `args` - Command and its arguments (first element is the command)
/// * `stdout_path` - Path where stdout will be written
/// * `stderr_path` - Path where stderr will be written
///
/// # Returns
///
/// Returns an `ExecutionResult` containing the exit code.
///
/// # Errors
///
/// Returns an error if:
/// - No command is provided
/// - File creation fails
/// - Command execution fails
///
/// # Examples
///
/// ```no_run
/// # use memo::executor::execute_and_stream;
/// # use std::path::Path;
/// let result = execute_and_stream(
///     &["echo", "hello"],
///     Path::new("/tmp/out.txt"),
///     Path::new("/tmp/err.txt")
/// ).expect("Command failed");
/// assert_eq!(result.exit_code, 0);
/// ```
pub fn execute_and_stream(
    args: &[&str],
    stdout_path: &Path,
    stderr_path: &Path,
) -> Result<ExecutionResult> {
    if args.is_empty() {
        return Err(MemoError::InvalidCommand("No command provided".to_string()));
    }

    let stdout_file = create_secure_file(stdout_path)?;
    let stderr_file = create_secure_file(stderr_path)?;

    let status = Command::new(args[0])
        .args(&args[1..])
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .status()?;

    let exit_code = status.code().unwrap_or(-1);

    Ok(ExecutionResult { exit_code })
}

/// Execute a command and stream output directly to stdout/stderr
///
/// This function executes a command without any caching, streaming output
/// directly to the current process's stdout and stderr.
///
/// # Arguments
///
/// * `args` - Command and its arguments (first element is the command)
///
/// # Returns
///
/// Returns an `ExecutionResult` containing the exit code.
///
/// # Errors
///
/// Returns an error if:
/// - No command is provided
/// - Command execution fails
///
/// # Examples
///
/// ```no_run
/// # use memo::executor::execute_direct;
/// let result = execute_direct(&["echo", "hello"]).expect("Command failed");
/// assert_eq!(result.exit_code, 0);
/// ```
pub fn execute_bypass(args: &[&str]) -> Result<ExecutionResult> {
    if args.is_empty() {
        return Err(MemoError::InvalidCommand("No command provided".to_string()));
    }

    let status = Command::new(args[0])
        .args(&args[1..])
        .status()?;

    let exit_code = status.code().unwrap_or(-1);

    Ok(ExecutionResult { exit_code })
}

/// Execute command for testing (keeps output in memory)
#[cfg(test)]
fn execute_command(args: &[&str]) -> std::io::Result<TestExecutionResult> {
    if args.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No command provided",
        ));
    }

    let output = Command::new(args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let exit_code = output.status.code().unwrap_or(-1);

    Ok(TestExecutionResult {
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code,
    })
}

#[cfg(test)]
struct TestExecutionResult {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_builder_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let stdout_path = temp_dir.path().join("out");
        let stderr_path = temp_dir.path().join("err");

        let result = CommandExecutor::new()
            .args(&["echo", "builder test"])
            .stdout_path(&stdout_path)
            .stderr_path(&stderr_path)
            .execute()
            .unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(fs::read_to_string(&stdout_path).unwrap(), "builder test\n");
    }

    #[test]
    fn test_execute_simple_command() {
        let result = execute_command(&["echo", "hello"]).unwrap();
        assert_eq!(result.stdout, b"hello\n");
        assert_eq!(result.stderr, b"");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execute_with_stderr() {
        let result = execute_command(&["sh", "-c", "echo error >&2"]).unwrap();
        assert_eq!(result.stdout, b"");
        assert_eq!(result.stderr, b"error\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execute_with_exit_code() {
        let result = execute_command(&["sh", "-c", "exit 42"]).unwrap();
        assert_eq!(result.exit_code, 42);
    }

    #[test]
    fn test_execute_mixed_output() {
        let result = execute_command(&["sh", "-c", "echo out; echo err >&2"]).unwrap();
        assert_eq!(result.stdout, b"out\n");
        assert_eq!(result.stderr, b"err\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execute_multiple_args() {
        let result = execute_command(&["printf", "%s %s", "hello", "world"]).unwrap();
        assert_eq!(result.stdout, b"hello world");
    }

    #[test]
    fn test_execute_empty_output() {
        let result = execute_command(&["true"]).unwrap();
        assert_eq!(result.stdout, b"");
        assert_eq!(result.stderr, b"");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execute_failure() {
        let result = execute_command(&["false"]).unwrap();
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn test_execute_with_special_chars() {
        let result = execute_command(&["echo", "hello \"world\""]).unwrap();
        assert_eq!(result.stdout, b"hello \"world\"\n");
    }

    #[test]
    fn test_execute_invalid_command() {
        let result = execute_command(&["this-command-does-not-exist-xyz"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_with_env_vars() {
        // Commands should execute in current environment
        let result = execute_command(&["sh", "-c", "echo $HOME"]).unwrap();
        // Should have some output (the HOME value)
        assert!(!result.stdout.is_empty());
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execute_binary_output() {
        let result = execute_command(&["printf", "\\x00\\x01\\xFF"]).unwrap();
        assert_eq!(result.stdout, vec![0x00, 0x01, 0xFF]);
    }

    #[test]
    fn test_execute_and_stream() {
        let temp_dir = TempDir::new().unwrap();
        let stdout_path = temp_dir.path().join("out");
        let stderr_path = temp_dir.path().join("err");

        let result = execute_and_stream(
            &["sh", "-c", "echo hello; echo world >&2"],
            &stdout_path,
            &stderr_path,
        )
        .unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(fs::read(&stdout_path).unwrap(), b"hello\n");
        assert_eq!(fs::read(&stderr_path).unwrap(), b"world\n");
    }

    #[test]
    fn test_execute_and_stream_large_output() {
        let temp_dir = TempDir::new().unwrap();
        let stdout_path = temp_dir.path().join("out");
        let stderr_path = temp_dir.path().join("err");

        // Generate 1MB of output without holding it all in memory in the command
        let result = execute_and_stream(
            &[
                "sh",
                "-c",
                "dd if=/dev/zero bs=1024 count=1024 2>/dev/null | tr '\\0' 'A'",
            ],
            &stdout_path,
            &stderr_path,
        )
        .unwrap();

        assert_eq!(result.exit_code, 0);
        let output = fs::read(&stdout_path).unwrap();
        assert_eq!(output.len(), 1024 * 1024);
        assert!(output.iter().all(|&b| b == b'A'));
    }

    #[test]
    fn test_execute_and_stream_binary() {
        let temp_dir = TempDir::new().unwrap();
        let stdout_path = temp_dir.path().join("out");
        let stderr_path = temp_dir.path().join("err");

        let result =
            execute_and_stream(&["printf", "\\x00\\x01\\xFF"], &stdout_path, &stderr_path).unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(fs::read(&stdout_path).unwrap(), vec![0x00, 0x01, 0xFF]);
    }

    #[test]
    fn test_build_command_string() {
        let cmd =
            build_command_string(&["echo".to_string(), "hello".to_string(), "world".to_string()]);
        assert_eq!(cmd, "echo hello world");
    }

    #[test]
    fn test_build_command_string_single_arg() {
        let cmd = build_command_string(&["ls".to_string()]);
        assert_eq!(cmd, "ls");
    }

    #[test]
    fn test_build_command_string_empty() {
        let cmd = build_command_string(&[]);
        assert_eq!(cmd, "");
    }

    #[test]
    fn test_build_command_string_with_spaces() {
        let cmd = build_command_string(&["echo".to_string(), "hello world".to_string()]);
        assert_eq!(cmd, "echo hello world");
    }

    #[test]
    fn test_build_command_string_special_chars() {
        let cmd = build_command_string(&[
            "echo".to_string(),
            "\"quoted\"".to_string(),
            "$VAR".to_string(),
        ]);
        assert_eq!(cmd, "echo \"quoted\" $VAR");
    }
}

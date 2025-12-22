// Unit tests for the executor module

use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

pub struct ExecutionResult {
    pub exit_code: i32,
}

pub fn build_command_string(args: &[String]) -> String {
    args.join(" ")
}

/// Execute command and stream output directly to files
pub fn execute_and_stream(
    args: &[&str],
    stdout_path: &Path,
    stderr_path: &Path,
) -> io::Result<ExecutionResult> {
    if args.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No command provided",
        ));
    }

    let mut stdout_opts = OpenOptions::new();
    stdout_opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        stdout_opts.mode(0o600);
    }
    let stdout_file = stdout_opts.open(stdout_path)?;

    let mut stderr_opts = OpenOptions::new();
    stderr_opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        stderr_opts.mode(0o600);
    }
    let stderr_file = stderr_opts.open(stderr_path)?;

    let status = Command::new(args[0])
        .args(&args[1..])
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .status()?;

    let exit_code = status.code().unwrap_or(-1);

    Ok(ExecutionResult { exit_code })
}

/// Execute command for testing (keeps output in memory)
#[cfg(test)]
fn execute_command(args: &[&str]) -> io::Result<TestExecutionResult> {
    if args.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
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
            &["sh", "-c", "dd if=/dev/zero bs=1024 count=1024 2>/dev/null | tr '\\0' 'A'"],
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

        let result = execute_and_stream(
            &["printf", "\\x00\\x01\\xFF"],
            &stdout_path,
            &stderr_path,
        )
        .unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(fs::read(&stdout_path).unwrap(), vec![0x00, 0x01, 0xFF]);
    }

    #[test]
    fn test_build_command_string() {
        let cmd = build_command_string(&["echo".to_string(), "hello".to_string(), "world".to_string()]);
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
        let cmd = build_command_string(&["echo".to_string(), "\"quoted\"".to_string(), "$VAR".to_string()]);
        assert_eq!(cmd, "echo \"quoted\" $VAR");
    }
}

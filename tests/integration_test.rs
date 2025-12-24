use assert_cmd::Command;
use predicates::prelude::predicate;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test environment for integration tests
///
/// Provides a clean temporary cache directory and helper methods for
/// interacting with the memo binary.
struct TestEnv {
    cache_dir: TempDir,
}

impl TestEnv {
    /// Create a new test environment with a temporary cache directory
    fn new() -> Self {
        let cache_dir = TempDir::new().unwrap();
        Self { cache_dir }
    }

    /// Get the path to the cache directory
    fn cache_path(&self) -> PathBuf {
        self.cache_dir.path().to_path_buf()
    }

    /// Create a configured Command for the memo binary
    ///
    /// The command is pre-configured with the test cache directory.
    fn cmd(&self) -> Command {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("memo");
        cmd.env("XDG_CACHE_HOME", self.cache_dir.path());
        cmd
    }

    /// List all cache files in sorted order
    fn list_cache_files(&self) -> Vec<String> {
        let memo_dir = self.cache_path().join("memo");
        if !memo_dir.exists() {
            return vec![];
        }

        let mut files: Vec<String> = fs::read_dir(&memo_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        files.sort();
        files
    }

    /// Read a cache file by name
    fn read_cache_file(&self, filename: &str) -> Vec<u8> {
        let path = self.cache_path().join("memo").join(filename);
        fs::read(&path).unwrap()
    }

    /// Assert that the cache contains exactly the specified number of files
    fn assert_cache_file_count(&self, expected: usize) {
        let files = self.list_cache_files();
        assert_eq!(
            files.len(),
            expected,
            "Expected {} cache files, found {}",
            expected,
            files.len()
        );
    }

    /// Assert that cache structure is valid (has .json, .out, .err for each memo)
    fn assert_valid_cache_structure(&self) {
        let files = self.list_cache_files();
        let memo_count = files.len() / 3;

        assert_eq!(
            files.len() % 3,
            0,
            "Cache should have 3 files per memo (json, out, err)"
        );

        for i in 0..memo_count {
            let has_json = files.iter().any(|f| f.ends_with(".json"));
            let has_out = files.iter().any(|f| f.ends_with(".out"));
            let has_err = files.iter().any(|f| f.ends_with(".err"));

            assert!(has_json, "Missing .json file for memo {}", i);
            assert!(has_out, "Missing .out file for memo {}", i);
            assert!(has_err, "Missing .err file for memo {}", i);
        }
    }
}

// Test Case 1: Basic Memoization
#[test]
fn test_basic_memoization() {
    let env = TestEnv::new();

    // First run - execute
    let output1 = env
        .cmd()
        .arg("echo")
        .arg("Hello, World!")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(String::from_utf8_lossy(&output1), "Hello, World!\n");

    // Second run - replay
    let output2 = env
        .cmd()
        .arg("echo")
        .arg("Hello, World!")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(String::from_utf8_lossy(&output2), "Hello, World!\n");
    assert_eq!(output1, output2);
}

// Test Case 2: Verbose Mode
#[test]
fn test_verbose_mode() {
    let env = TestEnv::new();

    // First run - execute with verbose
    env.cmd()
        .arg("--verbose")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout("test\n")
        .stderr(predicate::str::contains("miss `echo test`"));

    // Second run - replay with verbose
    env.cmd()
        .arg("--verbose")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout("test\n")
        .stderr(predicate::str::contains("hit `echo test`"));
}

// Test Case 3: Different Commands
#[test]
fn test_different_commands() {
    let env = TestEnv::new();

    env.cmd()
        .arg("echo")
        .arg("foo")
        .assert()
        .success()
        .stdout("foo\n");

    env.cmd()
        .arg("echo")
        .arg("bar")
        .assert()
        .success()
        .stdout("bar\n");

    // Verify first command still replays correctly
    env.cmd()
        .arg("echo")
        .arg("foo")
        .assert()
        .success()
        .stdout("foo\n");
}

// Test Case 4: Exit Code Preservation
#[test]
fn test_exit_code_preservation() {
    let env = TestEnv::new();

    // First run - execute with non-zero exit
    env.cmd()
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .code(42);

    // Second run - replay exit code
    env.cmd()
        .arg("sh")
        .arg("-c")
        .arg("exit 42")
        .assert()
        .code(42);
}

// Test Case 5: Stderr Capture
#[test]
fn test_stderr_capture() {
    let env = TestEnv::new();

    // First run - execute
    env.cmd()
        .arg("sh")
        .arg("-c")
        .arg("echo out; echo err >&2")
        .assert()
        .success()
        .stdout("out\n")
        .stderr("err\n");

    // Second run - replay
    env.cmd()
        .arg("sh")
        .arg("-c")
        .arg("echo out; echo err >&2")
        .assert()
        .success()
        .stdout("out\n")
        .stderr("err\n");
}

// Test Case 6: Argument Separator
#[test]
fn test_argument_separator() {
    let env = TestEnv::new();

    // Test -- separator prevents --verbose from being interpreted as flag
    env.cmd()
        .arg("--")
        .arg("echo")
        .arg("--verbose")
        .assert()
        .success()
        .stdout("--verbose\n");

    // Test --verbose before -- works as flag
    env.cmd()
        .arg("--verbose")
        .arg("--")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout("test\n")
        .stderr(predicate::str::contains("miss `echo test`"));
}

// Test Case 7: Complex Commands
#[test]
fn test_complex_commands() {
    let env = TestEnv::new();

    // First run
    let output1 = env
        .cmd()
        .arg("ls")
        .arg("-la")
        .arg("/etc/hosts")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Second run - should be identical
    let output2 = env
        .cmd()
        .arg("ls")
        .arg("-la")
        .arg("/etc/hosts")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(output1, output2);
}

// Test Case 8: Help Display
#[test]
fn test_help_display() {
    let env = TestEnv::new();

    env.cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--verbose"))
        .stdout(predicate::str::contains("--help"));
}

// Test Case 9: No Command Error
#[test]
fn test_no_command_error() {
    let env = TestEnv::new();

    env.cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// Test Case 10: Cache Directory Creation
#[test]
fn test_cache_directory_creation() {
    let env = TestEnv::new();

    // Cache dir should not exist initially
    let memo_dir = env.cache_path().join("memo");
    assert!(!memo_dir.exists());

    // Run command
    env.cmd()
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout("test\n");

    // Cache dir should now exist with three files
    assert!(memo_dir.exists());
    env.assert_cache_file_count(3);
    env.assert_valid_cache_structure();
}

// Test Case 11: Whitespace Handling
#[test]
fn test_whitespace_handling() {
    let env = TestEnv::new();

    // First run
    env.cmd()
        .arg("echo")
        .arg("  spaces  ")
        .assert()
        .success()
        .stdout("  spaces  \n");

    // Second run - whitespace should be preserved
    env.cmd()
        .arg("echo")
        .arg("  spaces  ")
        .assert()
        .success()
        .stdout("  spaces  \n");
}

// Test Case 12: Empty Output
#[test]
fn test_empty_output() {
    let env = TestEnv::new();

    // First run
    env.cmd().arg("true").assert().success().stdout("");

    // Second run
    env.cmd().arg("true").assert().success().stdout("");
}

// Additional Test: Verify cache file structure
#[test]
fn test_cache_file_structure() {
    let env = TestEnv::new();

    env.cmd().arg("echo").arg("hello").assert().success();

    let files = env.list_cache_files();
    assert_eq!(files.len(), 3);

    // Find the digest (basename without extension)
    let digest = files[0]
        .trim_end_matches(".err")
        .trim_end_matches(".json")
        .trim_end_matches(".out");

    // Verify all three files exist with same digest
    let json_file = format!("{}.json", digest);
    let out_file = format!("{}.out", digest);
    let err_file = format!("{}.err", digest);

    assert!(files.contains(&json_file));
    assert!(files.contains(&out_file));
    assert!(files.contains(&err_file));

    // Verify .out contains the stdout
    let out_content = env.read_cache_file(&out_file);
    assert_eq!(out_content, b"hello\n");

    // Verify .err is empty (echo has no stderr)
    let err_content = env.read_cache_file(&err_file);
    assert_eq!(err_content, b"");

    // Verify .json has valid structure
    let json_content = env.read_cache_file(&json_file);
    let json: serde_json::Value = serde_json::from_slice(&json_content).unwrap();

    assert!(json["cmd"].is_array());
    assert_eq!(
        json["cmd"].as_array().unwrap(),
        &vec![
            serde_json::Value::String("echo".into()),
            serde_json::Value::String("hello".into()),
        ]
    );
    assert!(json["exit_code"].is_number());
    assert_eq!(json["exit_code"].as_i64().unwrap(), 0);
    assert!(json["timestamp"].is_string());
    assert!(json["digest"].is_string());
    assert_eq!(json["digest"].as_str().unwrap(), digest);
}

// Additional Test: Binary data handling
#[test]
fn test_binary_data() {
    let env = TestEnv::new();

    // Create a command that outputs binary data
    env.cmd()
        .arg("printf")
        .arg("\\x00\\x01\\x02\\xFF")
        .assert()
        .success();

    // Replay should work with binary data
    let output = env
        .cmd()
        .arg("printf")
        .arg("\\x00\\x01\\x02\\xFF")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(output, vec![0x00, 0x01, 0x02, 0xFF]);
}

// Additional Test: Command with multiple arguments
#[test]
fn test_multiple_arguments() {
    let env = TestEnv::new();

    env.cmd()
        .arg("printf")
        .arg("%s %s %s")
        .arg("one")
        .arg("two")
        .arg("three")
        .assert()
        .success()
        .stdout("one two three");

    // Verify replay
    env.cmd()
        .arg("printf")
        .arg("%s %s %s")
        .arg("one")
        .arg("two")
        .arg("three")
        .assert()
        .success()
        .stdout("one two three");
}

// Additional Test: Command with quotes and special characters
#[test]
fn test_special_characters() {
    let env = TestEnv::new();

    env.cmd()
        .arg("echo")
        .arg("hello \"world\" $USER")
        .assert()
        .success()
        .stdout("hello \"world\" $USER\n");
}

// Additional Test: Different commands create different cache entries
#[test]
fn test_different_cache_entries() {
    let env = TestEnv::new();

    env.cmd().arg("echo").arg("foo").assert().success();
    env.cmd().arg("echo").arg("bar").assert().success();
    env.cmd().arg("echo").arg("baz").assert().success();

    // Should have 9 files (3 commands × 3 files each)
    env.assert_cache_file_count(9);
    env.assert_valid_cache_structure();
}

// Additional Test: Verify verbose flag can use short form
#[test]
fn test_verbose_short_flag() {
    let env = TestEnv::new();

    env.cmd()
        .arg("-v")
        .arg("echo")
        .arg("test")
        .assert()
        .success()
        .stdout("test\n")
        .stderr(predicate::str::contains("miss"));
}

// Additional Test: Mixed stdout/stderr with exit code
#[test]
fn test_mixed_output_with_error() {
    let env = TestEnv::new();

    env.cmd()
        .arg("sh")
        .arg("-c")
        .arg("echo stdout; echo stderr >&2; exit 5")
        .assert()
        .code(5)
        .stdout("stdout\n")
        .stderr("stderr\n");

    // Verify replay preserves all three
    env.cmd()
        .arg("sh")
        .arg("-c")
        .arg("echo stdout; echo stderr >&2; exit 5")
        .assert()
        .code(5)
        .stdout("stdout\n")
        .stderr("stderr\n");
}

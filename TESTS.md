# Test Suite Summary for Memo Application

## Overview
Comprehensive test suite covering all functionality described in PLAN.md.

## Test Organization

### Integration Tests (`tests/integration_test.rs`)
End-to-end tests using the compiled binary via `assert_cmd`.

**Test Helper:**
- `TestEnv` - Manages isolated test environments with temporary cache directories

**12 Core Validation Tests (from PLAN.md):**

1. **test_basic_memoization** - Verifies command execution and replay produce identical output
2. **test_verbose_mode** - Checks verbose flag outputs correct messages to stderr
3. **test_different_commands** - Ensures different commands create separate cache entries
4. **test_exit_code_preservation** - Verifies exit codes are stored and replayed correctly
5. **test_stderr_capture** - Tests stdout/stderr separation in capture and replay
6. **test_argument_separator** - Validates `--` separator correctly terminates option processing
7. **test_complex_commands** - Tests multi-argument commands (e.g., `ls -la /etc/hosts`)
8. **test_help_display** - Verifies `--help` shows proper usage information
9. **test_no_command_error** - Checks error handling when no command provided
10. **test_cache_directory_creation** - Validates automatic cache directory creation
11. **test_whitespace_handling** - Ensures whitespace is preserved in arguments
12. **test_empty_output** - Tests commands with no output (e.g., `true`)

**Additional Integration Tests:**

13. **test_cache_file_structure** - Verifies three files (.json, .out, .err) created per memo
14. **test_binary_data** - Tests handling of binary/non-UTF8 data
15. **test_multiple_arguments** - Validates complex argument passing
16. **test_special_characters** - Tests quotes and special characters in arguments
17. **test_different_cache_entries** - Verifies multiple memos coexist correctly
18. **test_verbose_short_flag** - Tests `-v` short form of verbose flag
19. **test_mixed_output_with_error** - Tests stdout + stderr + non-zero exit code together

### Unit Tests

#### Digest Module (`src/digest.rs`)
Tests SHA-256 hash computation:
- Same input produces same output
- Different inputs produce different outputs
- Output format (64 hex characters)
- Whitespace sensitivity
- Order sensitivity
- Empty string handling
- Special character handling
- Multiline input
- Known value verification (test vector)

#### Memo Module (`src/memo.rs`)
Tests JSON serialization/deserialization:
- Serialization to JSON
- Deserialization from JSON
- Roundtrip preservation
- Special characters in commands
- Negative exit codes
- Multiline commands

#### Cache Module (`src/cache.rs`)
Tests cache file operations:
- Directory creation
- Idempotent directory creation
- Write and read memo files
- Empty output handling
- Binary data storage
- Memo existence checking
- Reading non-existent memos (error case)
- Multiple memos coexistence
- Correct file naming (.json, .out, .err)
- XDG_CONFIG_HOME environment variable respect
- Large output (1MB test)

#### Executor Module (`src/executor.rs`)
Tests command execution:
- Simple command execution
- Stderr capture
- Exit code capture
- Mixed stdout/stderr output
- Multiple arguments
- Empty output
- Command failure (non-zero exit)
- Special characters
- Invalid command (error case)
- Environment variable access
- Binary output
- Command string building

## Test Coverage

### Functional Coverage
✓ All 12 validation test cases from PLAN.md
✓ Command-line argument parsing
✓ Verbose mode (long and short flags)
✓ Help text display
✓ Cache directory management
✓ File I/O operations
✓ SHA-256 digest computation
✓ JSON serialization
✓ Command execution
✓ Exit code handling
✓ Binary data handling

### Edge Cases Covered
✓ Empty output
✓ Empty input (digest)
✓ Large output (1MB)
✓ Binary/non-UTF8 data
✓ Special characters and quotes
✓ Whitespace preservation
✓ Multiple arguments
✓ Non-zero exit codes
✓ Negative exit codes
✓ Mixed stdout/stderr
✓ Command not found errors
✓ Invalid/missing arguments

### File Structure Tests
✓ Three files per memo (digest.json, digest.out, digest.err)
✓ Correct file naming
✓ File coexistence for multiple memos
✓ XDG_CONFIG_HOME environment variable
✓ Fallback to ~/.config

## Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test

# Run specific test
cargo test test_basic_memoization

# Run with output
cargo test -- --nocapture

# Run with verbose output
cargo test -- --nocapture --test-threads=1
```

## Test Dependencies

From `Cargo.toml`:
```toml
[dev-dependencies]
tempfile = "3.8"      # Temporary directories for isolated tests
assert_cmd = "2.0"    # Testing CLI applications
predicates = "3.0"    # Assertions for command output
```

## Test Data Isolation

All tests use:
- Temporary directories (`tempfile::TempDir`)
- Isolated XDG_CONFIG_HOME environments
- Automatic cleanup after test completion
- No interference between tests

## Expected Test Results

All tests should pass before implementation. After implementation:
- 9 unit tests in digest module
- 6 unit tests in memo module  
- 13 unit tests in cache module
- 15 unit tests in executor module
- 19 integration tests

**Total: 62 tests**

## Verification Checklist

Before implementation approval:
- [ ] All test files compile without errors
- [ ] Test structure matches PLAN.md requirements
- [ ] All 12 validation test cases present
- [ ] Edge cases covered
- [ ] Binary data handling tested
- [ ] File structure verified (.json, .out, .err)
- [ ] XDG_CONFIG_HOME tested
- [ ] Error cases covered
- [ ] Verbose mode tested

---

**Status**: Ready for Review
**Next Step**: Implement modules to make tests pass

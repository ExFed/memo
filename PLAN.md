# Memo Application - Implementation Plan

## Overview

A Rust command-line tool that memoizes shell command execution results, storing
stdout, stderr, and exit codes for repeated executions without re-running the
command.

## Core Functionality

### Command-Line Interface

```bash
memo [OPTIONS] [--] <COMMAND> [ARGS...]
```

**Arguments:**

- `<COMMAND> [ARGS...]` - The shell command to execute/memoize (required)
- `--` - Optional argument separator to terminate option processing

**Options:**

- `--verbose, -v` - Print memoization information to console
- `--help, -h` - Display usage information

**Examples:**

```bash
memo echo hello                    # Execute and memoize
memo echo hello                    # Replay from cache
memo --verbose -- echo hello       # With verbose output
memo -- ls -la /tmp               # Using -- separator
memo --verbose git status         # Complex command
```

### Memoization Strategy

#### Cache Key Generation

- **Key**: SHA-256 digest of the complete command line
  - Input: Full command string including all arguments
  - Format: Hex-encoded digest (64 characters)
  - Example: `echo hello` → `sha256("echo hello")` →
    `2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824`

#### Storage Location

- **Base Directory**: `$XDG_CONFIG_HOME/memo/`
  - Falls back to `$HOME/.config/memo/` if `XDG_CONFIG_HOME` is not set
  - Creates directory structure if it doesn't exist
- **File Structure**:

  ```text
  $XDG_CONFIG_HOME/memo/
  ├── <digest1>.json    # Metadata
  ├── <digest1>.out     # stdout
  ├── <digest1>.err     # stderr
  ├── <digest2>.json
  ├── <digest2>.out
  └── <digest2>.err
  ```

#### Memo File Format

**Metadata File (`<digest>.json`):**

```json
{
  "command": "echo hello",
  "exit_code": 0,
  "timestamp": "2025-12-22T01:45:05.389Z",
  "digest": "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
}
```

**Fields:**

- `command` (string): Original command line executed
- `exit_code` (i32): Process exit code
- `timestamp` (string): ISO 8601 timestamp of execution
- `digest` (string): SHA-256 hex digest (for verification)

**Output Files:**

- `<digest>.out` - Raw stdout bytes (preserves binary data, encoding, newlines)
- `<digest>.err` - Raw stderr bytes (preserves binary data, encoding, newlines)

### Execution Flow

#### 1. Parse Command Line

```text
Input: memo --verbose -- echo hello world
↓
Parse options: --verbose = true
Parse command: "echo hello world"
```

#### 2. Generate Cache Key

```text
Command: "echo hello world"
↓
SHA-256: compute_digest("echo hello world")
↓
Digest: "abc123..." (64 hex chars)
```

#### 3. Check Cache

```text
Path: $XDG_CONFIG_HOME/memo/abc123....json
↓
Exists? → YES: Load and replay
        → NO: Execute and store
```

#### 4a. Cache Hit (Replay)

1. Read metadata file (<digest>.json)
2. Parse JSON
3. Verify digest matches
4. If --verbose: Print "Replaying memoized result for: <command>"
5. Read and write <digest>.out to process stdout
6. Read and write <digest>.err to process stderr
7. Exit with stored exit_code

#### 4b. Cache Miss (Execute)

1. If --verbose: Print "Executing and memoizing: <command>"
2. Execute command in shell
3. Capture stdout, stderr, exit_code
4. Create memo metadata object with timestamp
5. Write to cache files atomically:
   - Write <digest>.json (metadata)
   - Write <digest>.out (stdout)
   - Write <digest>.err (stderr)
6. Write stdout to process stdout
7. Write stderr to process stderr
8. Exit with command's exit_code

## Technical Design

### Dependencies (Cargo.toml)

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
hex = "0.4"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"  # For XDG directory handling
```

### Module Structure

```text
src/
├── main.rs           # CLI parsing, main execution flow
├── memo.rs           # Memo struct and serialization
├── cache.rs          # Cache operations (read/write)
├── executor.rs       # Command execution logic
└── digest.rs         # Hash computation
```

### Key Rust Structures

```rust
// Memo metadata structure
#[derive(Serialize, Deserialize, Debug)]
struct Memo {
    command: String,
    exit_code: i32,
    timestamp: String,
    digest: String,
}

// CLI arguments
#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[arg(last = true, required = true)]
    command: Vec<String>,
}
```

### Error Handling

- Invalid command: Exit with error message
- Cache directory creation failure: Exit with error
- File I/O errors: Exit with descriptive message
- JSON parse errors: Treat as cache miss, re-execute
- Command execution errors: Store and replay the error

## Validation Test Cases

### Test Case 1: Basic Memoization

```bash
# First run - execute
$ memo echo "Hello, World!"
Hello, World!
$ echo $?
0

# Second run - replay
$ memo echo "Hello, World!"
Hello, World!
$ echo $?
0
```

**Expected**: Identical output, same exit code, second run is instant.

### Test Case 2: Verbose Mode

```bash
$ memo --verbose echo test
@memo: Executing and memoizing: echo test
test

$ memo --verbose echo test
@memo: Replaying memoized result for: echo test
test
```

**Expected**: Verbose messages to stderr, output to stdout.

### Test Case 3: Different Commands

```bash
$ memo echo foo
foo

$ memo echo bar
bar

$ memo echo foo
foo
```

**Expected**: Different commands create different memos, each replays correctly.

### Test Case 4: Exit Code Preservation

```bash
$ memo sh -c "exit 42"
$ echo $?
42

$ memo sh -c "exit 42"
$ echo $?
42
```

**Expected**: Non-zero exit codes are preserved and replayed.

### Test Case 5: Stderr Capture

```bash
$ memo sh -c "echo out; echo err >&2"
out
err

$ memo sh -c "echo out; echo err >&2"
out
err
```

**Expected**: Both stdout and stderr are captured and replayed.

### Test Case 6: Argument Separator

```bash
$ memo -- echo --verbose
--verbose

$ memo --verbose -- echo test
Executing and memoizing: echo test
test
```

**Expected**: `--` correctly separates options from command.

### Test Case 7: Complex Commands

```bash
$ memo ls -la /etc/hosts
[output of ls -la /etc/hosts]

$ memo ls -la /etc/hosts
[same output, instant]
```

**Expected**: Multi-argument commands work correctly.

### Test Case 8: Help Display

```bash
$ memo --help
[Usage information displayed]
```

**Expected**: Help text shows usage, options, examples.

### Test Case 9: No Command Error

```bash
$ memo
error: Missing required command
[Usage hint]
```

**Expected**: Clear error when no command provided.

### Test Case 10: Cache Directory Creation

```bash
$ rm -rf $XDG_CONFIG_HOME/memo
$ memo echo test
test
$ ls $XDG_CONFIG_HOME/memo
[digest].json  [digest].out  [digest].err
```

**Expected**: Cache directory created automatically with all three files.

### Test Case 11: Whitespace Handling

```bash
$ memo echo "  spaces  "
  spaces

$ memo echo "  spaces  "
  spaces
```

**Expected**: Whitespace preserved in output and memoization.

### Test Case 12: Empty Output

```bash
$ memo true
$ echo $?
0

$ memo true
$ echo $?
0
```

**Expected**: Commands with no output work correctly.

## Implementation Steps

1. **Project Setup**
   - Initialize Cargo project
   - Add dependencies
   - Create module structure

2. **Digest Module**
   - Implement SHA-256 hashing for command strings
   - Return hex-encoded digest

3. **Memo Module**
   - Define Memo struct with serde (metadata only)
   - Implement serialization/deserialization

4. **Cache Module**
   - Implement cache directory path resolution (XDG)
   - Directory creation with proper error handling
   - File read/write operations for:
     - Metadata JSON (<digest>.json)
     - Raw stdout bytes (<digest>.out)
     - Raw stderr bytes (<digest>.err)
   - Atomic write operations

5. **Executor Module**
   - Command execution with stdout/stderr capture
   - Exit code capture
   - Use `std::process::Command`

6. **Main Module**
   - CLI parsing with clap
   - Main execution flow
   - Verbose output handling
   - Error handling and user-friendly messages

7. **Testing**
   - Unit tests for each module
   - Integration tests for complete workflows
   - Manual validation against test cases

## Security Considerations

- **Command Injection**: Commands are executed via shell; users must trust their
  input
- **File System**: Cache files stored in user directory with standard
  permissions
- **Disk Usage**: No automatic cleanup; users manage cache manually
- **Hash Collisions**: SHA-256 provides sufficient collision resistance

## Future Enhancements (Out of Scope)

- Cache expiration/TTL
- Cache size limits
- Cache clearing command
- Parallel execution safety (file locking)
- Compression for large outputs
- Statistics (cache hit rate)
- Environment variable consideration in key
- Working directory consideration in key

## Success Criteria

✓ Application compiles without warnings
✓ All 12 validation test cases pass
✓ Help text is clear and accurate
✓ Verbose mode provides useful information
✓ Cache files use separate .json, .out, .err files
✓ Binary/non-UTF8 data handled correctly in output files
✓ Exit codes match original command execution
✓ Stdout and stderr are correctly separated
✓ XDG_CONFIG_HOME is respected
✓ Directory creation is automatic and safe
✓ Error messages are user-friendly

---

**Document Version**: 1.1
**Date**: 2025-12-22
**Status**: Ready for Implementation

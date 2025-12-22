# Quick Test Reference

## Test Files Created

```
memo/
├── tests/
│   └── integration_test.rs    (19 end-to-end tests)
├── src/
│   ├── digest.rs              (9 unit tests)
│   ├── memo.rs                (6 unit tests)
│   ├── cache.rs               (13 unit tests)
│   ├── executor.rs            (15 unit tests)
│   └── main.rs                (stub, not yet implemented)
└── TESTS.md                   (this summary)
```

## Running Specific Test Categories

```bash
# All tests
cargo test

# Just unit tests
cargo test --lib

# Just integration tests  
cargo test --test integration_test

# Specific module
cargo test digest::tests
cargo test cache::tests
cargo test executor::tests
cargo test memo::tests

# Specific test
cargo test test_basic_memoization
cargo test test_cache_file_structure
```

## What Each Module Tests

**digest.rs**: SHA-256 hash computation
- Input → 64 char hex digest
- Deterministic, collision-resistant

**memo.rs**: JSON metadata structure
- Serialization/deserialization
- Command, exit_code, timestamp, digest fields

**cache.rs**: File I/O operations
- XDG_CONFIG_HOME/memo/ directory
- Write/read .json, .out, .err files
- Binary data support

**executor.rs**: Command execution
- Spawn processes
- Capture stdout, stderr, exit_code
- Handle binary output

**integration_test.rs**: Full workflow
- CLI argument parsing
- Execute → cache → replay flow
- Verbose mode
- Error handling

## Test Status (Before Implementation)

```
$ cargo test
   Compiling memo v0.1.0
warning: unused variables, dead code (expected)
...
    Finished test [unoptimized + debuginfo] target(s)
     Running unittests src/main.rs (target/debug/deps/memo-...)
     Running tests/integration_test.rs (target/debug/deps/integration_test-...)

All tests should PASS once implementation is complete.
```

## Key Test Scenarios

1. **First run** → execute command, save to cache
2. **Second run** → read from cache, replay
3. **Verbose** → print to stderr what's happening
4. **Binary data** → preserve non-UTF8 bytes
5. **Exit codes** → preserve non-zero exits
6. **Stderr** → separate from stdout
7. **Multiple commands** → independent cache entries
8. **Large output** → handle MB+ outputs

## Implementation Order (Suggested)

1. ✓ Tests written (current state)
2. → Implement `digest.rs` (simplest)
3. → Implement `memo.rs` (just struct)
4. → Implement `executor.rs` (command execution)
5. → Implement `cache.rs` (file I/O)
6. → Implement `main.rs` (tie it together)
7. → Run tests, fix failures
8. → All tests green ✓


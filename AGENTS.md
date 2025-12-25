# Agents Guide

## Build & Test

- **Build**: `cargo build`
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt`
- **Test**: `cargo test`
- **Run Single Test**: `cargo test -- test_name_substring`
- **Run Integration**: `cargo test --test integration_test`
- **Run**: `cargo run -- [args]`

## Code Style & Conventions

- **Rust Edition**: 2021.
- **Formatting**: Strict `cargo fmt`.
- **Imports**: Group std, external (3rd party), and crate (internal) modules separately.
- **Errors**: Use `thiserror` for library errors and `Result<T, Box<dyn std::error::Error>>` or `anyhow` for app logic if appropriate. Currently using `error::Result`.
- **Async**: Project is currently **synchronous** (using standard `std::process`, `std::fs`).
- **Safety**: Avoid `unsafe` unless absolutely required.
- **Conventions**: Follow standard Rust idioms (snake_case for functions/vars, CamelCase for types).
- **Paths**: Use `std::path::PathBuf` for file paths.
- **Logging**: Use `eprintln!` for CLI status messages (prefixed with `:: memo ::`) to avoid polluting stdout.

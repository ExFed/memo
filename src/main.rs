//! # Memo - Command Memoization Tool
//!
//! Memo is a command-line tool that memoizes (caches) shell command execution results.
//! When you run a command through memo, it stores the stdout, stderr, and exit code.
//! Subsequent executions of the same command will replay the cached results instantly
//! without re-running the command.
//!
//! ## How It Works
//!
//! - **Cache Key**: SHA-256 hash of the command arguments and current working directory
//! - **Storage**: Three separate files per memoized command:
//!   - `<digest>.json` - Metadata (command, exit code, timestamp)
//!   - `<digest>.out` - Captured stdout
//!   - `<digest>.err` - Captured stderr
//! - **Location**: `$XDG_CACHE_HOME/memo/` (defaults to `~/.cache/memo/`)
//!
//! ## Usage Examples
//!
//! ```bash
//! # First run executes the command
//! memo echo "Hello, World!"
//!
//! # Second run replays from cache (instant)
//! memo echo "Hello, World!"
//!
//! # Verbose mode shows cache hits/misses
//! memo -v ls -la /etc
//!
//! # Commands with different arguments create separate cache entries
//! memo echo "foo"
//! memo echo "bar"
//! ```
//!
//! ## Features
//!
//! - Preserves exact stdout, stderr, and exit codes
//! - Handles binary data correctly
//! - Streaming architecture for memory efficiency
//! - Lock-based concurrency control
//! - Secure file permissions on Unix systems

mod cache;
mod constants;
mod digest;
mod error;
mod executor;
mod memo;

use cache::{
    create_secure_file, ensure_cache_dir, get_cache_dir, get_cache_paths, is_memo_disabled,
    memo_complete, purge_memo, read_memo_metadata, stream_stderr, stream_stdout, try_acquire_lock,
};
use chrono::Utc;
use clap::Parser;
use constants::{LOCK_WAIT_INTERVAL_MS, LOCK_WAIT_TIMEOUT_SECS};
use digest::compute_digest_for_args;
use error::{MemoError, Result};
use executor::{build_command_string, execute_and_stream, execute_bypass};
use memo::Memo;
use std::io;
use std::process;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "memo")]
#[command(about = "Memoize shell command execution", long_about = None)]
struct Cli {
    /// Print memoization information
    #[arg(short, long)]
    verbose: bool,

    /// Command to execute/memoize
    #[arg(trailing_var_arg = true, required = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Cli::parse();

    // Check if memoization is disabled
    if is_memo_disabled() {
        if args.verbose {
            eprintln!(":: memo :: disabled");
        }

        // Convert Vec<String> to Vec<&str>
        let cmd_args: Vec<&str> = args.command.iter().map(|s| s.as_str()).collect();

        // Execute directly without caching
        let result = execute_bypass(&cmd_args)?;
        process::exit(result.exit_code);
    }

    // Get cache directory
    let cache_dir = get_cache_dir()?;
    ensure_cache_dir(&cache_dir)?;

    // Get current working directory
    let cwd = std::env::current_dir()?.to_string_lossy().to_string();

    // Build command string for display and compute digest from argv.
    let command_string = build_command_string(&args.command);
    let digest = compute_digest_for_args(&args.command, &cwd)?;

    // Check if memo exists
    if memo_complete(&cache_dir, &digest) {
        // Cache hit - replay
        if args.verbose {
            eprintln!(":: memo :: hit `{command_string}` => {digest}");
        }

        // Read metadata
        let memo = read_memo_metadata(&cache_dir, &digest)?;

        // Stream output to stdout/stderr
        stream_stdout(&cache_dir, &digest, io::stdout())?;
        stream_stderr(&cache_dir, &digest, io::stderr())?;

        // Exit with stored exit code
        process::exit(memo.exit_code);
    } else {
        // Cache miss - execute and memoize
        if args.verbose {
            eprintln!(":: memo :: miss `{command_string}` => {digest}");
        }

        // Best-effort wait if another process is currently memoizing the same digest.
        let wait_deadline = Instant::now() + Duration::from_secs(LOCK_WAIT_TIMEOUT_SECS);
        loop {
            match try_acquire_lock(&cache_dir, &digest) {
                Ok(lock) => {
                    // If we acquired the lock but the memo became complete meanwhile, just replay.
                    if memo_complete(&cache_dir, &digest) {
                        if args.verbose {
                            eprintln!(":: memo :: hit `{command_string}` => {digest}");
                        }
                        let memo = read_memo_metadata(&cache_dir, &digest)?;
                        stream_stdout(&cache_dir, &digest, io::stdout())?;
                        stream_stderr(&cache_dir, &digest, io::stderr())?;
                        drop(lock);
                        process::exit(memo.exit_code);
                    }

                    // If a prior run left a partial cache behind, clear it.
                    purge_memo(&cache_dir, &digest);

                    // Get cache file paths
                    let (json_path, out_path, err_path) = get_cache_paths(&cache_dir, &digest);

                    // Convert Vec<String> to Vec<&str>
                    let cmd_args: Vec<&str> = args.command.iter().map(|s| s.as_str()).collect();

                    // Execute command and stream to files
                    let result = execute_and_stream(&cmd_args, &out_path, &err_path)?;

                    // Create memo metadata
                    let memo = Memo {
                        cmd: args.command.clone(),
                        cwd: cwd.clone(),
                        exit_code: result.exit_code,
                        timestamp: Utc::now().to_rfc3339(),
                        digest: digest.clone(),
                    };

                    // Write metadata to JSON (only if it doesn't already exist)
                    let json = serde_json::to_string_pretty(&memo)?;

                    {
                        use std::io::Write;
                        let mut f = create_secure_file(&json_path)?;
                        f.write_all(json.as_bytes())?;
                    }

                    // Stream output to stdout/stderr
                    stream_stdout(&cache_dir, &digest, io::stdout())?;
                    stream_stderr(&cache_dir, &digest, io::stderr())?;

                    // Exit with command's exit code
                    drop(lock);
                    process::exit(result.exit_code);
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    if memo_complete(&cache_dir, &digest) {
                        if args.verbose {
                            eprintln!(":: memo :: hit `{command_string}` => {digest}");
                        }
                        let memo = read_memo_metadata(&cache_dir, &digest)?;
                        stream_stdout(&cache_dir, &digest, io::stdout())?;
                        stream_stderr(&cache_dir, &digest, io::stderr())?;
                        process::exit(memo.exit_code);
                    }

                    if Instant::now() >= wait_deadline {
                        return Err(MemoError::LockTimeout);
                    }

                    std::thread::sleep(Duration::from_millis(LOCK_WAIT_INTERVAL_MS));
                }
                Err(e) => return Err(MemoError::Io(e)),
            }
        }
    }
}

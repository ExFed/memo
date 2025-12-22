mod cache;
mod digest;
mod executor;
mod memo;

use cache::{
    ensure_cache_dir, get_cache_dir, get_cache_paths, memo_complete, purge_memo, read_memo_metadata,
    stream_stderr, stream_stdout, try_acquire_lock,
};
use chrono::Utc;
use clap::Parser;
use digest::compute_digest_for_args;
use executor::{build_command_string, execute_and_stream};
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

fn run() -> io::Result<()> {
    let args = Cli::parse();

    // Get cache directory
    let cache_dir = get_cache_dir()?;
    ensure_cache_dir(&cache_dir)?;

    // Build command string for display and compute digest from argv.
    let command_string = build_command_string(&args.command);
    let digest = compute_digest_for_args(&args.command)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Check if memo exists
    if memo_complete(&cache_dir, &digest) {
        // Cache hit - replay
        if args.verbose {
            eprintln!(":: memo :: Replaying memoized result for: {}", command_string);
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
            eprintln!(":: memo :: Executing and memoizing: {}", command_string);
        }

        // Best-effort wait if another process is currently memoizing the same digest.
        let wait_deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match try_acquire_lock(&cache_dir, &digest) {
                Ok(lock) => {
                    // If we acquired the lock but the memo became complete meanwhile, just replay.
                    if memo_complete(&cache_dir, &digest) {
                        if args.verbose {
                            eprintln!(
                                ":: memo :: Replaying memoized result for: {}",
                                command_string
                            );
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
                        exit_code: result.exit_code,
                        timestamp: Utc::now().to_rfc3339(),
                        digest: digest.clone(),
                    };

                    // Write metadata to JSON (only if it doesn't already exist)
                    let json = serde_json::to_string_pretty(&memo)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                    let mut opts = std::fs::OpenOptions::new();
                    opts.write(true).create_new(true);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        opts.mode(0o600);
                    }
                    {
                        use std::io::Write;
                        let mut f = opts.open(json_path)?;
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
                            eprintln!(
                                ":: memo :: Replaying memoized result for: {}",
                                command_string
                            );
                        }
                        let memo = read_memo_metadata(&cache_dir, &digest)?;
                        stream_stdout(&cache_dir, &digest, io::stdout())?;
                        stream_stderr(&cache_dir, &digest, io::stderr())?;
                        process::exit(memo.exit_code);
                    }

                    if Instant::now() >= wait_deadline {
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "Timed out waiting for memoization lock",
                        ));
                    }

                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

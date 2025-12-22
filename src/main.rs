mod cache;
mod digest;
mod executor;
mod memo;

use cache::{ensure_cache_dir, get_cache_dir, get_cache_paths, memo_exists, read_memo_metadata, stream_stderr, stream_stdout};
use chrono::Utc;
use clap::Parser;
use digest::compute_digest;
use executor::{build_command_string, execute_and_stream};
use memo::Memo;
use std::io;
use std::process;

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

    // Build command string and compute digest
    let command_string = build_command_string(&args.command);
    let digest = compute_digest(&command_string);

    // Check if memo exists
    if memo_exists(&cache_dir, &digest) {
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

        // Get cache file paths
        let (json_path, out_path, err_path) = get_cache_paths(&cache_dir, &digest);

        // Convert Vec<String> to Vec<&str>
        let cmd_args: Vec<&str> = args.command.iter().map(|s| s.as_str()).collect();

        // Execute command and stream to files
        let result = execute_and_stream(&cmd_args, &out_path, &err_path)?;

        // Create memo metadata
        let memo = Memo {
            command: command_string,
            exit_code: result.exit_code,
            timestamp: Utc::now().to_rfc3339(),
            digest: digest.clone(),
        };

        // Write metadata to JSON
        let json = serde_json::to_string_pretty(&memo)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        std::fs::write(json_path, json)?;

        // Stream output to stdout/stderr
        stream_stdout(&cache_dir, &digest, io::stdout())?;
        stream_stderr(&cache_dir, &digest, io::stderr())?;

        // Exit with command's exit code
        process::exit(result.exit_code);
    }
}

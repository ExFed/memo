mod cache;
mod digest;
mod executor;
mod memo;

use clap::Parser;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "memo")]
#[command(about = "Memoize shell command execution", long_about = None)]
struct Cli {
    /// Print memoization information
    #[arg(short, long)]
    verbose: bool,

    /// Command to execute/memoize
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

fn main() {
    // Parse CLI arguments
    let args = Cli::parse();

    // TODO: Implement main logic
    eprintln!("Not yet implemented");
    process::exit(1);
}

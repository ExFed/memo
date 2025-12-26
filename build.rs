use std::process::Command;

fn main() {
    // Attempt to get version from git describe, filtering for version tags (v*.*.*)
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty", "--match", "v*.*.*"])
        .output();

    let git_version = match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => {
            // Fallback to Cargo.toml version if git is unavailable or fails
            env!("CARGO_PKG_VERSION").to_string()
        }
    };

    println!("cargo:rustc-env=GIT_VERSION={}", git_version);
    
    // Rerun build script if git state changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");
}

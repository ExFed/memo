use std::process::Command;

use vergen_gitcl::{
    BuildBuilder, CargoBuilder, Emitter, GitclBuilder, RustcBuilder, SysinfoBuilder,
};

fn main() {
    emit_version().unwrap();
}

fn emit_version() -> anyhow::Result<()> {
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let gitcl = GitclBuilder::default()
        .all()
        .describe(true, true, Some("v*.*.*"))
        .commit_timestamp(false) // workaround: see below
        .commit_date(false) // workaround: see below
        .build()?;
    let rustc = RustcBuilder::all_rustc()?;
    let sysinfo = SysinfoBuilder::all_sysinfo()?;

    Emitter::new()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gitcl)?
        .add_instructions(&rustc)?
        .add_instructions(&sysinfo)?
        .emit()?;

    // workaround: when SOURCE_DATE_EPOCH is set vergen does not emit the
    // correct commit date or timestamp even though it has access to the git
    // repository. Manually invoke git to get the values and emit them here.
    //
    // see: <https://github.com/rustyhorde/vergen/issues/452>
    emit_rustc_env_git_log_1("VERGEN_GIT_COMMIT_TIMESTAMP", "%cI")?;
    emit_rustc_env_git_log_1("VERGEN_GIT_COMMIT_DATE", "%cs")?;

    Ok(())
}

fn emit_rustc_env_git_log_1(k: &str, format: &str) -> anyhow::Result<()> {
    let o = Command::new("git")
        .args(["log", "-1", &format!("--format={}", format)])
        .output()?;

    if !o.status.success() {
        return Err(anyhow::anyhow!(
            "git log failed: {}",
            String::from_utf8_lossy(&o.stderr)
        ));
    }

    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if s.is_empty() {
        return Err(anyhow::anyhow!("git log returned empty output"));
    }

    println!("cargo:rustc-env={}={}", k, s);

    Ok(())
}

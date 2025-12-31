use vergen_gitcl::{
    BuildBuilder, CargoBuilder, Emitter, GitclBuilder, RustcBuilder, SysinfoBuilder,
};

fn main() {
    generate_version().unwrap();
}

fn generate_version() -> anyhow::Result<()> {
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let gitcl = GitclBuilder::default()
        .all()
        .describe(true, true, Some("v*.*.*"))
        .build()?;
    let rustc = RustcBuilder::all_rustc()?;
    let sysinfo = SysinfoBuilder::all_sysinfo()?;

    Emitter::new()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gitcl)?
        .add_instructions(&rustc)?
        .add_instructions(&sysinfo)?
        .emit()
}

use vergen_git2::{BuildBuilder, CargoBuilder, Emitter, Git2Builder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = BuildBuilder::default().build_timestamp(true).build()?;
    let cargo = CargoBuilder::default().build()?;
    let git2 = Git2Builder::default().describe(true, true, None).build()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&git2)?
        .emit()?;
    Ok(())
}

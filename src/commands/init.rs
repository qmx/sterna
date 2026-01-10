use std::fs;

use git2::Repository;

use crate::error::Error;

pub fn run() -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;

    let index_dir = repo_path.join("sterna/index");
    fs::create_dir_all(&index_dir)?;

    fs::write(index_dir.join("issues"), "")?;
    fs::write(index_dir.join("edges"), "")?;

    println!("Initialized Sterna in {}", repo_path.display());
    Ok(())
}

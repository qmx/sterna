use std::fs;
use std::io::{self, Write};

use git2::Repository;

use crate::commands::export;
use crate::error::Error;

pub fn run(yes: bool) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;

    let sterna_dir = repo_path.join("sterna");

    if !sterna_dir.exists() {
        return Err(Error::NotInitialized);
    }

    // Export current data as backup to stdout
    eprintln!("Exporting current data as backup...");
    export::run(None)?;
    eprintln!();

    // Prompt for confirmation unless --yes
    if !yes {
        eprint!("This will remove all Sterna data. Continue? [y/N] ");
        io::stderr().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            eprintln!("Aborted.");
            return Ok(());
        }
    }

    // Delete sterna/ directory
    fs::remove_dir_all(&sterna_dir)?;
    eprintln!("Removed sterna/ directory");

    // Delete refs/sterna/* if any exist
    let git_dir = repo.path();
    let refs_sterna = git_dir.join("refs").join("sterna");
    if refs_sterna.exists() {
        fs::remove_dir_all(&refs_sterna)?;
        eprintln!("Removed refs/sterna/");
    }

    eprintln!("Purge complete. Orphaned blobs will be cleaned by git gc.");

    Ok(())
}

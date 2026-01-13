use std::io::{self, Write};

use git2::Repository;

use crate::commands::export;
use crate::error::Error;
use crate::snapshot;

pub fn run(yes: bool) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    if !snapshot::is_initialized(&repo) {
        return Err(Error::NotInitialized);
    }

    eprintln!("Exporting current data as backup...");
    export::run(None)?;
    eprintln!();

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

    snapshot::delete_snapshot(&repo)?;
    eprintln!("Removed refs/sterna/snapshot");

    eprintln!("Purge complete. Orphaned blobs will be cleaned by git gc.");

    Ok(())
}

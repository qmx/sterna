use git2::Repository;

use crate::error::Error;
use crate::snapshot;

pub fn run() -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    snapshot::init(&repo)?;

    println!("Initialized Sterna");
    Ok(())
}

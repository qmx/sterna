use git2::Repository;

use crate::error::Error;

pub fn get_editor() -> Result<String, Error> {
    let repo = Repository::discover(".")?;
    let config = repo.config()?;
    config
        .get_string("user.email")
        .map_err(|_| Error::NoIdentity("Set git config user.email".into()))
}

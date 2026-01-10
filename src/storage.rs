use git2::Repository;

use crate::error::Error;
use crate::types::Issue;

pub fn get_editor() -> Result<String, Error> {
    let repo = Repository::discover(".")?;
    let config = repo.config()?;
    config
        .get_string("user.email")
        .map_err(|_| Error::NoIdentity("Set git config user.email".into()))
}

pub fn write_blob(repo: &Repository, content: &[u8]) -> Result<git2::Oid, Error> {
    Ok(repo.blob(content)?)
}

pub fn read_blob(repo: &Repository, oid: git2::Oid) -> Result<Vec<u8>, Error> {
    let blob = repo.find_blob(oid)?;
    Ok(blob.content().to_vec())
}

pub fn serialize_issue(issue: &Issue) -> Result<Vec<u8>, Error> {
    Ok(serde_json::to_vec(issue)?)
}

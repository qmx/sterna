use git2::Repository;

use crate::error::Error;
use crate::snapshot;

pub fn run(remote: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let remote_name = remote.unwrap_or_else(|| "origin".to_string());

    if !snapshot::is_initialized(&repo) {
        return Err(Error::NotInitialized);
    }

    // Get current snapshot commit for reporting
    let reference = repo.find_reference("refs/sterna/snapshot")?;
    let commit = reference.peel_to_commit()?;
    let commit_id = commit.id().to_string();

    // Count issues and edges for reporting
    let issues = snapshot::load_issues(&repo)?;
    let edges = snapshot::load_edges(&repo)?;

    // Push to remote
    let mut git_remote = repo.find_remote(&remote_name)?;
    git_remote.push(&["refs/sterna/snapshot:refs/sterna/snapshot"], None)?;

    eprintln!(
        "Pushed snapshot ({}) with {} issues, {} edges to {}",
        &commit_id[..7],
        issues.len(),
        edges.len(),
        remote_name
    );

    Ok(())
}

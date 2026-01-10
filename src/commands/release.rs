use git2::Repository;

use crate::error::Error;
use crate::index::IssueIndex;
use crate::storage;
use crate::types::{Issue, Status};

pub fn run(id_prefix: String, reason: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let mut index = IssueIndex::load(repo_path)?;

    let (id, oid) = index.find_unique(&id_prefix)?;
    let data = storage::read_blob(&repo, oid)?;
    let mut issue = Issue::from_json(&data)?;

    if !issue.claimed {
        return Err(Error::NotClaimed(id));
    }

    issue.claimed = false;
    issue.status = Status::Open;
    issue.claim_context = None;
    issue.claimed_at = None;
    issue.reason = reason;
    issue.lamport += 1;
    issue.updated_at = chrono::Utc::now().timestamp();
    issue.editor = storage::get_editor()?;

    let content = storage::serialize_issue(&issue)?;
    let new_oid = storage::write_blob(&repo, &content)?;
    index.entries.insert(id.clone(), new_oid);
    index.save(repo_path)?;

    println!("Released {}", id);
    Ok(())
}

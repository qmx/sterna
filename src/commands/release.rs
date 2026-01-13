use git2::Repository;

use crate::error::Error;
use crate::snapshot;
use crate::storage;
use crate::types::Status;

pub fn run(id_prefix: String, reason: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    let id = snapshot::find_issue_id(&repo, &id_prefix)?;
    let mut issue = snapshot::load_issue(&repo, &id)?;

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

    snapshot::save_issue(&repo, &issue, &format!("Release issue {}", id))?;

    println!("Released {}", id);
    Ok(())
}

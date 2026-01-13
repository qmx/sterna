use git2::Repository;

use crate::error::Error;
use crate::snapshot;
use crate::storage;
use crate::types::Status;

pub fn run(id_prefix: String, context: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    let id = snapshot::find_issue_id(&repo, &id_prefix)?;
    let mut issue = snapshot::load_issue(&repo, &id)?;

    if issue.claimed {
        return Err(Error::AlreadyClaimed(id));
    }
    if issue.status == Status::Closed {
        return Err(Error::IsClosed(id));
    }

    issue.claimed = true;
    issue.status = Status::InProgress;
    issue.claim_context = context;
    issue.claimed_at = Some(chrono::Utc::now().timestamp() as u64);
    issue.lamport += 1;
    issue.updated_at = chrono::Utc::now().timestamp();
    issue.editor = storage::get_editor()?;

    snapshot::save_issue(&repo, &issue, &format!("Claim issue {}", id))?;

    println!("Claimed {}", id);
    Ok(())
}

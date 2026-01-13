use git2::Repository;

use crate::error::Error;
use crate::snapshot;
use crate::storage;
use crate::types::{IssueType, Priority};

pub fn run(
    id_prefix: String,
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    issue_type: Option<String>,
    labels: Option<Vec<String>>,
) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    let id = snapshot::find_issue_id(&repo, &id_prefix)?;
    let mut issue = snapshot::load_issue(&repo, &id)?;

    if let Some(t) = title {
        issue.title = t;
    }
    if let Some(d) = description {
        issue.description = d;
    }
    if let Some(p) = priority {
        issue.priority = Priority::from_str(&p)?;
    }
    if let Some(t) = issue_type {
        issue.issue_type = IssueType::from_str(&t)?;
    }
    if let Some(l) = labels {
        issue.labels = l;
    }

    issue.lamport += 1;
    issue.updated_at = chrono::Utc::now().timestamp();
    issue.editor = storage::get_editor()?;

    snapshot::save_issue(&repo, &issue, &format!("Update issue {}", id))?;

    println!("Updated {}", id);
    Ok(())
}

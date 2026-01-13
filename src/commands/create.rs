use git2::Repository;

use crate::error::Error;
use crate::id;
use crate::snapshot;
use crate::storage;
use crate::types::{Issue, IssueType, Priority, SCHEMA_VERSION, Status};

pub fn run(
    title: String,
    description: Option<String>,
    priority: Option<String>,
    issue_type: Option<String>,
    labels: Vec<String>,
) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let editor = storage::get_editor()?;

    let existing_ids = snapshot::get_existing_ids(&repo)?;

    let id = id::generate_id(
        &title,
        description.as_deref().unwrap_or(""),
        &editor,
        &existing_ids,
    );
    let now = chrono::Utc::now().timestamp();

    let priority = match priority {
        Some(p) => Priority::from_str(&p)?,
        None => Priority::Medium,
    };

    let issue_type = match issue_type {
        Some(t) => IssueType::from_str(&t)?,
        None => IssueType::Task,
    };

    let issue = Issue {
        schema_version: SCHEMA_VERSION,
        id: id.clone(),
        title,
        description: description.unwrap_or_default(),
        status: Status::Open,
        priority,
        issue_type,
        labels,
        created_at: now,
        updated_at: now,
        lamport: 1,
        editor,
        claimed: false,
        claim_context: None,
        claimed_at: None,
        reason: None,
    };

    snapshot::save_issue(&repo, &issue, &format!("Create issue {}", id))?;

    println!("{}", id);
    Ok(())
}

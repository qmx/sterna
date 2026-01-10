use git2::Repository;

use crate::error::Error;
use crate::index::IssueIndex;
use crate::storage;
use crate::types::{Issue, IssueType, Priority};

pub fn run(
    id_prefix: String,
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    issue_type: Option<String>,
    labels: Option<Vec<String>>,
) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let mut index = IssueIndex::load(repo_path)?;

    let (id, oid) = index.find_unique(&id_prefix)?;
    let data = storage::read_blob(&repo, oid)?;
    let mut issue = Issue::from_json(&data)?;

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

    let content = storage::serialize_issue(&issue)?;
    let new_oid = storage::write_blob(&repo, &content)?;
    index.entries.insert(id.clone(), new_oid);
    index.save(repo_path)?;

    println!("Updated {}", id);
    Ok(())
}

use chrono::{TimeZone, Utc};
use git2::Repository;

use crate::error::Error;
use crate::index::IssueIndex;
use crate::storage;
use crate::types::Issue;

pub fn run(id_prefix: String, json: bool) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let index = IssueIndex::load(repo_path)?;

    let (_, oid) = index.find_unique(&id_prefix)?;
    let data = storage::read_blob(&repo, oid)?;
    let issue = Issue::from_json(&data)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&issue)?);
    } else {
        print_issue(&issue);
    }
    Ok(())
}

fn print_issue(issue: &Issue) {
    println!("ID:          {}", issue.id);
    println!("Title:       {}", issue.title);
    println!("Status:      {:?}", issue.status);
    println!("Priority:    {}", issue.priority.as_str());
    println!("Type:        {}", issue.issue_type.as_str());
    if !issue.labels.is_empty() {
        println!("Labels:      {}", issue.labels.join(", "));
    }
    println!("Created:     {}", format_timestamp(issue.created_at));
    println!("Updated:     {}", format_timestamp(issue.updated_at));
    println!("Editor:      {}", issue.editor);
    println!("Claimed:     {}", issue.claimed);
    if let Some(ref ctx) = issue.claim_context {
        println!("Context:     {}", ctx);
    }
    if let Some(ref reason) = issue.reason {
        println!("Reason:      {}", reason);
    }
    if !issue.description.is_empty() {
        println!("\n{}", issue.description);
    }
}

fn format_timestamp(ts: i64) -> String {
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| ts.to_string())
}

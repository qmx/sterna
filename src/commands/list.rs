use git2::Repository;

use crate::error::Error;
use crate::index::IssueIndex;
use crate::storage;
use crate::types::{Issue, IssueType, Status};

pub fn run(status: Option<String>, issue_type: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let index = IssueIndex::load(repo_path)?;

    let status_filter = status.map(|s| parse_status(&s)).transpose()?;
    let type_filter = issue_type.map(|t| IssueType::from_str(&t)).transpose()?;

    let mut issues: Vec<Issue> = Vec::new();
    for (_, oid) in &index.entries {
        let data = storage::read_blob(&repo, *oid)?;
        let issue = Issue::from_json(&data)?;

        if let Some(ref s) = status_filter {
            if issue.status != *s {
                continue;
            }
        }
        if let Some(ref t) = type_filter {
            if issue.issue_type != *t {
                continue;
            }
        }
        issues.push(issue);
    }

    issues.sort_by(|a, b| (a.priority, a.created_at).cmp(&(b.priority, b.created_at)));

    println!(
        "{:<12} {:<12} {:<8} {:<10} {}",
        "ID", "STATUS", "PRI", "TYPE", "TITLE"
    );
    println!("{}", "-".repeat(60));
    for issue in issues {
        println!(
            "{:<12} {:<12} {:<8} {:<10} {}",
            issue.id,
            status_str(issue.status),
            issue.priority.as_str(),
            issue.issue_type.as_str(),
            truncate(&issue.title, 40)
        );
    }
    Ok(())
}

fn parse_status(s: &str) -> Result<Status, Error> {
    match s.to_lowercase().as_str() {
        "open" => Ok(Status::Open),
        "in_progress" | "inprogress" | "in-progress" => Ok(Status::InProgress),
        "closed" => Ok(Status::Closed),
        _ => Err(Error::InvalidPriority(format!("Unknown status: {}", s))),
    }
}

fn status_str(s: Status) -> &'static str {
    match s {
        Status::Open => "open",
        Status::InProgress => "in_progress",
        Status::Closed => "closed",
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

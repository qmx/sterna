use git2::Repository;

use crate::error::Error;
use crate::snapshot;
use crate::types::{Issue, IssueType, Status};

pub fn run(status: Option<String>, issue_type: Option<String>, json: bool) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    let status_filter = status.map(|s| parse_status(&s)).transpose()?;
    let type_filter = issue_type.map(|t| IssueType::from_str(&t)).transpose()?;

    let all_issues = snapshot::load_issues(&repo)?;

    let mut issues: Vec<Issue> = all_issues
        .into_values()
        .filter(|issue| {
            if let Some(ref s) = status_filter {
                if issue.status != *s {
                    return false;
                }
            }
            if let Some(ref t) = type_filter {
                if issue.issue_type != *t {
                    return false;
                }
            }
            true
        })
        .collect();

    issues.sort_by(|a, b| (a.priority, a.created_at).cmp(&(b.priority, b.created_at)));

    if json {
        println!("{}", serde_json::to_string_pretty(&issues)?);
    } else {
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

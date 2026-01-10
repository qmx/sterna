use git2::Repository;

use crate::error::Error;
use crate::index::IssueIndex;
use crate::storage;
use crate::types::{Issue, Status};

pub fn run() -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let index = IssueIndex::load(repo_path)?;

    let mut issues: Vec<Issue> = Vec::new();
    for (_, oid) in &index.entries {
        let data = storage::read_blob(&repo, *oid)?;
        let issue = Issue::from_json(&data)?;

        // Ready = open AND not claimed
        if issue.status == Status::Open && !issue.claimed {
            issues.push(issue);
        }
    }

    issues.sort_by_key(|i| i.priority);

    println!("{:<12} {:<8} {:<10} {}", "ID", "PRI", "TYPE", "TITLE");
    println!("{}", "-".repeat(50));
    for issue in issues {
        println!(
            "{:<12} {:<8} {:<10} {}",
            issue.id,
            issue.priority.as_str(),
            issue.issue_type.as_str(),
            truncate(&issue.title, 40)
        );
    }
    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

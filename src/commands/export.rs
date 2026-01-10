use std::fs;

use git2::Repository;
use serde::Serialize;

use crate::error::Error;
use crate::index::{EdgeIndex, IssueIndex};
use crate::storage;
use crate::types::{Edge, Issue};

#[derive(Serialize)]
struct Export {
    version: u32,
    exported_at: i64,
    issues: Vec<Issue>,
    edges: Vec<Edge>,
}

pub fn run(output: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;

    let issue_index = IssueIndex::load(repo_path)?;
    let edge_index = EdgeIndex::load(repo_path)?;

    // Load all issues
    let mut issues: Vec<Issue> = Vec::new();
    for (_, oid) in &issue_index.entries {
        let data = storage::read_blob(&repo, *oid)?;
        let issue = Issue::from_json(&data)?;
        issues.push(issue);
    }

    // Load all edges
    let mut edges: Vec<Edge> = Vec::new();
    for entry in &edge_index.entries {
        let data = storage::read_blob(&repo, entry.oid)?;
        let edge = Edge::from_json(&data)?;
        edges.push(edge);
    }

    let export = Export {
        version: 1,
        exported_at: chrono::Utc::now().timestamp(),
        issues,
        edges,
    };

    let json = serde_json::to_string_pretty(&export)?;

    match output {
        Some(path) => {
            fs::write(&path, &json)?;
            eprintln!("Exported to {}", path);
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

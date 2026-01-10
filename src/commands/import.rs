use std::fs;

use git2::Repository;
use serde::Deserialize;

use crate::error::Error;
use crate::index::{EdgeEntry, EdgeIndex, IssueIndex};
use crate::storage;
use crate::types::{Edge, Issue};

#[derive(Deserialize)]
struct Import {
    #[allow(dead_code)]
    version: u32,
    #[allow(dead_code)]
    exported_at: i64,
    issues: Vec<Issue>,
    edges: Vec<Edge>,
}

pub fn run(file: String) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;

    // Load existing data
    let mut issue_index = IssueIndex::load(repo_path)?;
    let mut edge_index = EdgeIndex::load(repo_path)?;

    // Parse import file
    let content = fs::read_to_string(&file)?;
    let import: Import = serde_json::from_str(&content)?;

    let mut issues_added = 0;
    let mut issues_updated = 0;
    let mut edges_added = 0;

    // Merge issues (LWW by Lamport)
    for imported_issue in import.issues {
        if let Some(&existing_oid) = issue_index.entries.get(&imported_issue.id) {
            // Issue exists - check Lamport clock
            let existing_data = storage::read_blob(&repo, existing_oid)?;
            let existing_issue = Issue::from_json(&existing_data)?;

            if imported_issue.lamport > existing_issue.lamport {
                // Imported is newer - replace
                let content = storage::serialize_issue(&imported_issue)?;
                let oid = storage::write_blob(&repo, &content)?;
                issue_index.entries.insert(imported_issue.id.clone(), oid);
                issues_updated += 1;
            }
        } else {
            // New issue - insert
            let content = storage::serialize_issue(&imported_issue)?;
            let oid = storage::write_blob(&repo, &content)?;
            issue_index.entries.insert(imported_issue.id.clone(), oid);
            issues_added += 1;
        }
    }

    // Merge edges (union - skip duplicates)
    for imported_edge in import.edges {
        let exists = edge_index.exists(
            &imported_edge.source,
            &imported_edge.target,
            imported_edge.edge_type,
        );

        if !exists {
            let content = storage::serialize_edge(&imported_edge)?;
            let oid = storage::write_blob(&repo, &content)?;
            edge_index.entries.push(EdgeEntry {
                source: imported_edge.source,
                target: imported_edge.target,
                edge_type: imported_edge.edge_type,
                oid,
            });
            edges_added += 1;
        }
    }

    // Save indices
    issue_index.save(repo_path)?;
    edge_index.save(repo_path)?;

    println!(
        "Imported: {} issues added, {} issues updated, {} edges added",
        issues_added, issues_updated, edges_added
    );

    Ok(())
}

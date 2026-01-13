use std::fs;

use git2::Repository;
use serde::Deserialize;

use crate::error::Error;
use crate::snapshot;
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

    let existing_issues = snapshot::load_issues(&repo)?;
    let existing_edges = snapshot::load_edges(&repo)?;

    let content = fs::read_to_string(&file)?;
    let import: Import = serde_json::from_str(&content)?;

    let mut issues_added = 0;
    let mut issues_updated = 0;
    let mut edges_added = 0;

    // Merge issues (LWW by Lamport)
    for imported_issue in import.issues {
        if let Some(existing_issue) = existing_issues.get(&imported_issue.id) {
            if imported_issue.lamport > existing_issue.lamport {
                snapshot::save_issue(
                    &repo,
                    &imported_issue,
                    &format!("Import: update issue {}", imported_issue.id),
                )?;
                issues_updated += 1;
            }
        } else {
            snapshot::save_issue(
                &repo,
                &imported_issue,
                &format!("Import: add issue {}", imported_issue.id),
            )?;
            issues_added += 1;
        }
    }

    // Merge edges (union - skip duplicates)
    for imported_edge in import.edges {
        let exists = existing_edges.iter().any(|e| {
            e.source == imported_edge.source
                && e.target == imported_edge.target
                && e.edge_type == imported_edge.edge_type
        });

        if !exists {
            snapshot::save_edge(
                &repo,
                &imported_edge,
                &format!(
                    "Import: add edge {} {} {}",
                    imported_edge.source,
                    imported_edge.edge_type.as_str(),
                    imported_edge.target
                ),
            )?;
            edges_added += 1;
        }
    }

    println!(
        "Imported: {} issues added, {} issues updated, {} edges added",
        issues_added, issues_updated, edges_added
    );

    Ok(())
}

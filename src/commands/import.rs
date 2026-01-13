use std::fs;

use git2::Repository;
use serde::Deserialize;

use crate::dag;
use crate::error::Error;
use crate::snapshot;
use crate::types::{Edge, EdgeType, Issue};

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

    let mut issues_to_save: Vec<Issue> = Vec::new();
    let mut edges_to_add: Vec<Edge> = Vec::new();
    let mut edges_skipped = 0;

    // Collect issues (LWW by Lamport)
    for imported_issue in import.issues {
        let dominated = if let Some(existing) = existing_issues.get(&imported_issue.id) {
            imported_issue.lamport > existing.lamport
                || (imported_issue.lamport == existing.lamport
                    && imported_issue.updated_at > existing.updated_at)
        } else {
            true
        };

        if dominated {
            issues_to_save.push(imported_issue);
        }
    }

    // Collect edges (union - skip duplicates and cycles)
    let mut current_edges = existing_edges.clone();

    for imported_edge in import.edges {
        let exists = current_edges.iter().any(|e| {
            e.source == imported_edge.source
                && e.target == imported_edge.target
                && e.edge_type == imported_edge.edge_type
        });

        if exists {
            continue;
        }

        // Check for cycles (skip for RelatesTo and Duplicates)
        if !matches!(imported_edge.edge_type, EdgeType::RelatesTo | EdgeType::Duplicates) {
            if dag::would_create_cycle(
                &current_edges,
                &imported_edge.source,
                &imported_edge.target,
                imported_edge.edge_type,
            ) {
                eprintln!(
                    "Skipping edge {} -> {} ({}): would create cycle",
                    imported_edge.source,
                    imported_edge.target,
                    imported_edge.edge_type.as_str()
                );
                edges_skipped += 1;
                continue;
            }
        }

        current_edges.push(imported_edge.clone());
        edges_to_add.push(imported_edge);
    }

    // Calculate counts
    let issues_added = issues_to_save
        .iter()
        .filter(|i| !existing_issues.contains_key(&i.id))
        .count();
    let issues_updated = issues_to_save.len() - issues_added;
    let edges_added = edges_to_add.len();

    // Single batch commit
    if !issues_to_save.is_empty() || !edges_to_add.is_empty() {
        snapshot::merge_snapshot(
            &repo,
            &issues_to_save,
            &edges_to_add,
            &format!(
                "Import: {} issues, {} edges",
                issues_to_save.len(),
                edges_to_add.len()
            ),
        )?;
    }

    println!(
        "Imported: {} issues added, {} issues updated, {} edges added{}",
        issues_added,
        issues_updated,
        edges_added,
        if edges_skipped > 0 {
            format!(" ({} edges skipped due to cycles)", edges_skipped)
        } else {
            String::new()
        }
    );

    Ok(())
}

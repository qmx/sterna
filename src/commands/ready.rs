use std::collections::HashMap;

use git2::Repository;

use crate::error::Error;
use crate::snapshot;
use crate::types::{Edge, EdgeType, Issue, Status};

pub fn run(json: bool) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    let all_issues = snapshot::load_issues(&repo)?;
    let edges = snapshot::load_edges(&repo)?;

    let mut ready_issues: Vec<Issue> = Vec::new();
    for issue in all_issues.values() {
        // Ready = open AND not claimed AND not blocked
        if issue.status == Status::Open && !issue.claimed {
            if !is_blocked(&issue.id, &edges, &all_issues) {
                ready_issues.push(issue.clone());
            }
        }
    }

    ready_issues.sort_by_key(|i| i.priority);

    if json {
        println!("{}", serde_json::to_string_pretty(&ready_issues)?);
    } else {
        println!("{:<12} {:<8} {:<10} {}", "ID", "PRI", "TYPE", "TITLE");
        println!("{}", "-".repeat(50));
        for issue in ready_issues {
            println!(
                "{:<12} {:<8} {:<10} {}",
                issue.id,
                issue.priority.as_str(),
                issue.issue_type.as_str(),
                truncate(&issue.title, 40)
            );
        }
    }
    Ok(())
}

/// Check if an issue is blocked by unclosed dependencies.
/// An issue is blocked if:
/// - It has a DependsOn edge to an unclosed issue
/// - Another issue has a Blocks edge pointing to it and that issue is unclosed
fn is_blocked(issue_id: &str, edges: &[Edge], issues: &HashMap<String, Issue>) -> bool {
    for edge in edges {
        match edge.edge_type {
            EdgeType::DependsOn => {
                // If this issue depends on another, check if target is closed
                if edge.source == issue_id {
                    if let Some(target) = issues.get(&edge.target) {
                        if target.status != Status::Closed {
                            return true;
                        }
                    }
                }
            }
            EdgeType::Blocks => {
                // If another issue blocks this one, check if source is closed
                if edge.target == issue_id {
                    if let Some(source) = issues.get(&edge.source) {
                        if source.status != Status::Closed {
                            return true;
                        }
                    }
                }
            }
            EdgeType::ParentChild => {
                // Child is blocked if parent is not closed
                if edge.source == issue_id {
                    if let Some(parent) = issues.get(&edge.target) {
                        if parent.status != Status::Closed {
                            return true;
                        }
                    }
                }
            }
            // RelatesTo and Duplicates don't block
            _ => {}
        }
    }
    false
}

fn truncate(s: &str, max_len: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", chars[..max_len - 3].iter().collect::<String>())
    }
}

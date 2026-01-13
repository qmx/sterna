use git2::Repository;

use crate::error::Error;
use crate::snapshot;
use crate::types::{Edge, Issue};

pub fn run(remote: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let remote_name = remote.unwrap_or_else(|| "origin".to_string());

    // Fetch remote snapshot ref
    let mut git_remote = repo.find_remote(&remote_name)?;
    git_remote.fetch(
        &["refs/sterna/snapshot:refs/sterna/remote"],
        None,
        None,
    )?;

    // Load remote snapshot
    let remote_ref = repo.find_reference("refs/sterna/remote")?;
    let remote_commit = remote_ref.peel_to_commit()?;
    let remote_tree = remote_commit.tree()?;

    // Load local data
    let local_issues = snapshot::load_issues(&repo)?;
    let local_edges = snapshot::load_edges(&repo)?;

    let mut issues_added = 0;
    let mut issues_updated = 0;
    let mut edges_added = 0;

    // Load remote issues subtree
    let issues_tree_entry = remote_tree
        .get_name("issues")
        .ok_or(Error::InvalidSnapshot)?;
    let issues_tree = issues_tree_entry.to_object(&repo)?.peel_to_tree()?;

    for entry in issues_tree.iter() {
        let blob = repo.find_blob(entry.id())?;
        let remote_issue = Issue::from_json(blob.content())?;

        if let Some(existing_issue) = local_issues.get(&remote_issue.id) {
            // Issue exists - check Lamport clock
            if remote_issue.lamport > existing_issue.lamport {
                // Remote is newer - replace
                snapshot::save_issue(
                    &repo,
                    &remote_issue,
                    &format!("Pull: update issue {}", remote_issue.id),
                )?;
                issues_updated += 1;
            }
        } else {
            // New issue - insert
            snapshot::save_issue(
                &repo,
                &remote_issue,
                &format!("Pull: add issue {}", remote_issue.id),
            )?;
            issues_added += 1;
        }
    }

    // Merge edges from remote (union)
    let edges_tree_entry = remote_tree
        .get_name("edges")
        .ok_or(Error::InvalidSnapshot)?;
    let edges_tree = edges_tree_entry.to_object(&repo)?.peel_to_tree()?;

    for entry in edges_tree.iter() {
        let blob = repo.find_blob(entry.id())?;
        let remote_edge = Edge::from_json(blob.content())?;

        let exists = local_edges.iter().any(|e| {
            e.source == remote_edge.source
                && e.target == remote_edge.target
                && e.edge_type == remote_edge.edge_type
        });

        if !exists {
            snapshot::save_edge(
                &repo,
                &remote_edge,
                &format!(
                    "Pull: add edge {} {} {}",
                    remote_edge.source,
                    remote_edge.edge_type.as_str(),
                    remote_edge.target
                ),
            )?;
            edges_added += 1;
        }
    }

    // Clean up remote ref
    repo.find_reference("refs/sterna/remote")?.delete()?;

    eprintln!(
        "Pulled from {}: {} issues added, {} issues updated, {} edges added",
        remote_name, issues_added, issues_updated, edges_added
    );

    Ok(())
}

use git2::Repository;

use crate::error::Error;
use crate::index::{EdgeEntry, EdgeIndex, IssueIndex};
use crate::storage;
use crate::types::{Edge, Issue, Snapshot};

pub fn run(remote: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
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

    // Parse remote snapshot.json
    let snapshot_entry = remote_tree
        .get_name("snapshot.json")
        .ok_or(Error::InvalidSnapshot)?;
    let snapshot_blob = snapshot_entry.to_object(&repo)?.peel_to_blob()?;
    let _remote_snapshot: Snapshot = serde_json::from_slice(snapshot_blob.content())?;

    // Load local indices
    let mut issue_index = IssueIndex::load(repo_path)?;
    let mut edge_index = EdgeIndex::load(repo_path)?;

    let mut issues_added = 0;
    let mut issues_updated = 0;
    let mut edges_added = 0;

    // Merge issues from remote
    let issues_tree_entry = remote_tree.get_name("issues").ok_or(Error::InvalidSnapshot)?;
    let issues_tree = issues_tree_entry.to_object(&repo)?.peel_to_tree()?;

    for entry in issues_tree.iter() {
        let issue_oid = entry.id();
        let blob = repo.find_blob(issue_oid)?;
        let remote_issue = Issue::from_json(blob.content())?;

        if let Some(&existing_oid) = issue_index.entries.get(&remote_issue.id) {
            // Issue exists - check Lamport clock
            let existing_data = storage::read_blob(&repo, existing_oid)?;
            let existing_issue = Issue::from_json(&existing_data)?;

            if remote_issue.lamport > existing_issue.lamport {
                // Remote is newer - replace
                issue_index.entries.insert(remote_issue.id.clone(), issue_oid);
                issues_updated += 1;
            }
        } else {
            // New issue - insert
            issue_index.entries.insert(remote_issue.id.clone(), issue_oid);
            issues_added += 1;
        }
    }

    // Merge edges from remote (union)
    let edges_tree_entry = remote_tree.get_name("edges").ok_or(Error::InvalidSnapshot)?;
    let edges_tree = edges_tree_entry.to_object(&repo)?.peel_to_tree()?;

    for entry in edges_tree.iter() {
        let edge_oid = entry.id();
        let blob = repo.find_blob(edge_oid)?;
        let remote_edge = Edge::from_json(blob.content())?;

        let exists = edge_index.exists(
            &remote_edge.source,
            &remote_edge.target,
            remote_edge.edge_type,
        );

        if !exists {
            edge_index.entries.push(EdgeEntry {
                source: remote_edge.source,
                target: remote_edge.target,
                edge_type: remote_edge.edge_type,
                oid: edge_oid,
            });
            edges_added += 1;
        }
    }

    // Save updated indices
    issue_index.save(repo_path)?;
    edge_index.save(repo_path)?;

    eprintln!(
        "Pulled from {}: {} issues added, {} issues updated, {} edges added",
        remote_name, issues_added, issues_updated, edges_added
    );

    Ok(())
}

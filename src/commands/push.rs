use git2::Repository;

use crate::error::Error;
use crate::index::{EdgeIndex, IssueIndex};
use crate::storage;
use crate::types::{Snapshot, SCHEMA_VERSION};

pub fn run(remote: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let remote_name = remote.unwrap_or_else(|| "origin".to_string());

    let issue_index = IssueIndex::load(repo_path)?;
    let edge_index = EdgeIndex::load(repo_path)?;

    // Collect all issue and edge hashes
    let issue_hashes: Vec<String> = issue_index
        .entries
        .values()
        .map(|oid| oid.to_string())
        .collect();
    let edge_hashes: Vec<String> = edge_index
        .entries
        .iter()
        .map(|e| e.oid.to_string())
        .collect();

    // Calculate max lamport from issues
    let mut max_lamport = 0u64;
    for (_, oid) in &issue_index.entries {
        let data = storage::read_blob(&repo, *oid)?;
        let issue = crate::types::Issue::from_json(&data)?;
        if issue.lamport > max_lamport {
            max_lamport = issue.lamport;
        }
    }

    // Determine snapshot version (increment if previous exists)
    let version = get_current_version(&repo).unwrap_or(0) + 1;

    // Create snapshot
    let snapshot = Snapshot {
        schema_version: SCHEMA_VERSION,
        version,
        created_at: chrono::Utc::now().timestamp(),
        lamport: max_lamport,
        issue_hashes: issue_hashes.clone(),
        edge_hashes: edge_hashes.clone(),
    };

    // Serialize snapshot
    let snapshot_json = serde_json::to_vec_pretty(&snapshot)?;
    let snapshot_oid = repo.blob(&snapshot_json)?;

    // Build issues tree
    let mut issues_tree_builder = repo.treebuilder(None)?;
    for (id, oid) in &issue_index.entries {
        issues_tree_builder.insert(id, *oid, 0o100644)?;
    }
    let issues_tree_oid = issues_tree_builder.write()?;

    // Build edges tree
    let mut edges_tree_builder = repo.treebuilder(None)?;
    for (idx, entry) in edge_index.entries.iter().enumerate() {
        // Use index as filename for edges
        edges_tree_builder.insert(&format!("{:08}", idx), entry.oid, 0o100644)?;
    }
    let edges_tree_oid = edges_tree_builder.write()?;

    // Build root tree
    let mut root_tree_builder = repo.treebuilder(None)?;
    root_tree_builder.insert("snapshot.json", snapshot_oid, 0o100644)?;
    root_tree_builder.insert("issues", issues_tree_oid, 0o040000)?;
    root_tree_builder.insert("edges", edges_tree_oid, 0o040000)?;
    let root_tree_oid = root_tree_builder.write()?;

    // Create commit
    let tree = repo.find_tree(root_tree_oid)?;
    let sig = repo.signature()?;

    // Get parent commit if exists
    let parent: Option<git2::Commit> = repo
        .find_reference("refs/sterna/snapshot")
        .ok()
        .and_then(|r| r.peel_to_commit().ok());

    let parents: Vec<&git2::Commit> = parent.as_ref().map(|p| vec![p]).unwrap_or_default();

    let commit_oid = repo.commit(
        Some("refs/sterna/snapshot"),
        &sig,
        &sig,
        &format!("Sterna snapshot v{}", version),
        &tree,
        &parents,
    )?;

    eprintln!(
        "Created snapshot v{} ({}) with {} issues, {} edges",
        version,
        &commit_oid.to_string()[..7],
        issue_hashes.len(),
        edge_hashes.len()
    );

    // Push to remote
    let mut git_remote = repo.find_remote(&remote_name)?;
    git_remote.push(&["refs/sterna/snapshot:refs/sterna/snapshot"], None)?;

    eprintln!("Pushed to {}", remote_name);

    Ok(())
}

fn get_current_version(repo: &Repository) -> Option<u64> {
    let reference = repo.find_reference("refs/sterna/snapshot").ok()?;
    let commit = reference.peel_to_commit().ok()?;
    let tree = commit.tree().ok()?;
    let entry = tree.get_name("snapshot.json")?;
    let obj = entry.to_object(repo).ok()?;
    let blob = obj.as_blob()?;
    let snapshot: Snapshot = serde_json::from_slice(blob.content()).ok()?;
    Some(snapshot.version)
}

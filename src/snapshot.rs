use std::collections::HashMap;
use std::fs::{File, OpenOptions};

use fs2::FileExt;
use git2::{Commit, Repository, Tree};

use crate::error::Error;
use crate::types::{Edge, EdgeType, Issue};

const SNAPSHOT_REF: &str = "refs/sterna/snapshot";

/// Advisory lock for snapshot operations
pub struct SnapshotLock {
    _file: File,
}

impl SnapshotLock {
    pub fn acquire(repo: &Repository) -> Result<Self, Error> {
        let lock_path = repo.path().join("sterna.lock");
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)
            .map_err(|e| Error::LockFailed(e.to_string()))?;
        file.lock_exclusive()
            .map_err(|e| Error::LockFailed(e.to_string()))?;
        Ok(Self { _file: file })
    }
}

/// Check if Sterna is initialized in this repo
pub fn is_initialized(repo: &Repository) -> bool {
    repo.find_reference(SNAPSHOT_REF).is_ok()
}

/// Get the current snapshot commit, if any
fn get_snapshot_commit(repo: &Repository) -> Result<Commit, Error> {
    let reference = repo.find_reference(SNAPSHOT_REF)?;
    let commit = reference.peel_to_commit()?;
    Ok(commit)
}

/// Get the current snapshot tree
fn get_snapshot_tree(repo: &Repository) -> Result<Tree, Error> {
    let commit = get_snapshot_commit(repo)?;
    Ok(commit.tree()?)
}

/// Get a subtree by name from a parent tree
fn get_subtree<'a>(repo: &'a Repository, tree: &Tree, name: &str) -> Result<Tree<'a>, Error> {
    let entry = tree
        .get_name(name)
        .ok_or_else(|| Error::CorruptedSnapshot(format!("missing {} subtree", name)))?;
    let obj = entry.to_object(repo)?;
    obj.peel_to_tree()
        .map_err(|e| Error::CorruptedSnapshot(format!("{} is not a tree: {}", name, e)))
}

/// Initialize Sterna - creates empty snapshot with issues/ and edges/ subtrees
pub fn init(repo: &Repository) -> Result<(), Error> {
    if is_initialized(repo) {
        return Err(Error::AlreadyInitialized);
    }

    let issues_builder = repo.treebuilder(None)?;
    let issues_oid = issues_builder.write()?;

    let edges_builder = repo.treebuilder(None)?;
    let edges_oid = edges_builder.write()?;

    let mut root_builder = repo.treebuilder(None)?;
    root_builder.insert("issues", issues_oid, 0o040000)?;
    root_builder.insert("edges", edges_oid, 0o040000)?;
    let tree_oid = root_builder.write()?;

    let tree = repo.find_tree(tree_oid)?;
    let sig = repo.signature()?;

    repo.commit(
        Some(SNAPSHOT_REF),
        &sig,
        &sig,
        "Initialize Sterna",
        &tree,
        &[], // No parents - initial commit
    )?;

    Ok(())
}

/// Load all issues from the snapshot
pub fn load_issues(repo: &Repository) -> Result<HashMap<String, Issue>, Error> {
    if !is_initialized(repo) {
        return Err(Error::NotInitialized);
    }

    let tree = get_snapshot_tree(repo)?;
    let issues_tree = get_subtree(repo, &tree, "issues")?;

    let mut issues = HashMap::new();
    for entry in issues_tree.iter() {
        let id = entry.name().unwrap_or("").to_string();
        if id.is_empty() {
            continue;
        }
        let obj = entry.to_object(repo)?;
        let blob = obj.peel_to_blob()?;
        let issue = Issue::from_json(blob.content())?;
        issues.insert(id, issue);
    }
    Ok(issues)
}

/// Load a single issue by ID (or prefix)
pub fn load_issue(repo: &Repository, id_prefix: &str) -> Result<Issue, Error> {
    let issues = load_issues(repo)?;

    let matches: Vec<_> = issues
        .iter()
        .filter(|(id, _)| id.starts_with(id_prefix))
        .collect();

    match matches.len() {
        0 => Err(Error::NotFound(id_prefix.to_string())),
        1 => Ok(matches[0].1.clone()),
        _ => Err(Error::AmbiguousId(
            id_prefix.to_string(),
            matches.iter().map(|(id, _)| (*id).clone()).collect(),
        )),
    }
}

/// Find unique issue ID from prefix
pub fn find_issue_id(repo: &Repository, id_prefix: &str) -> Result<String, Error> {
    let issues = load_issues(repo)?;

    let matches: Vec<_> = issues
        .keys()
        .filter(|id| id.starts_with(id_prefix))
        .collect();

    match matches.len() {
        0 => Err(Error::NotFound(id_prefix.to_string())),
        1 => Ok(matches[0].clone()),
        _ => Err(Error::AmbiguousId(
            id_prefix.to_string(),
            matches.iter().map(|id| (*id).clone()).collect(),
        )),
    }
}

/// Load all edges from the snapshot
pub fn load_edges(repo: &Repository) -> Result<Vec<Edge>, Error> {
    if !is_initialized(repo) {
        return Err(Error::NotInitialized);
    }

    let tree = get_snapshot_tree(repo)?;
    let edges_tree = get_subtree(repo, &tree, "edges")?;

    let mut edges = Vec::new();
    for entry in edges_tree.iter() {
        let obj = entry.to_object(repo)?;
        let blob = obj.peel_to_blob()?;
        let edge = Edge::from_json(blob.content())?;
        edges.push(edge);
    }
    Ok(edges)
}

/// Check if an edge already exists
pub fn edge_exists(repo: &Repository, source: &str, target: &str, edge_type: EdgeType) -> Result<bool, Error> {
    let edges = load_edges(repo)?;
    Ok(edges
        .iter()
        .any(|e| e.source == source && e.target == target && e.edge_type == edge_type))
}

/// Save an issue (create or update)
pub fn save_issue(repo: &Repository, issue: &Issue, message: &str) -> Result<(), Error> {
    if !is_initialized(repo) {
        return Err(Error::NotInitialized);
    }
    let _lock = SnapshotLock::acquire(repo)?;

    let current_commit = get_snapshot_commit(repo)?;
    let current_tree = current_commit.tree()?;
    let issues_tree = get_subtree(repo, &current_tree, "issues")?;
    let edges_tree = get_subtree(repo, &current_tree, "edges")?;

    let blob_content = serde_json::to_vec(issue)?;
    let blob_oid = repo.blob(&blob_content)?;

    let mut issues_builder = repo.treebuilder(Some(&issues_tree))?;
    issues_builder.insert(&issue.id, blob_oid, 0o100644)?;
    let new_issues_oid = issues_builder.write()?;

    // Build new root tree (keeping edges unchanged)
    let mut root_builder = repo.treebuilder(None)?;
    root_builder.insert("issues", new_issues_oid, 0o040000)?;
    root_builder.insert("edges", edges_tree.id(), 0o040000)?;
    let new_tree_oid = root_builder.write()?;
    let new_tree = repo.find_tree(new_tree_oid)?;
    let sig = repo.signature()?;

    repo.commit(
        Some(SNAPSHOT_REF),
        &sig,
        &sig,
        message,
        &new_tree,
        &[&current_commit],
    )?;

    Ok(())
}

/// Save an edge
pub fn save_edge(repo: &Repository, edge: &Edge, message: &str) -> Result<(), Error> {
    if !is_initialized(repo) {
        return Err(Error::NotInitialized);
    }
    let _lock = SnapshotLock::acquire(repo)?;

    let current_commit = get_snapshot_commit(repo)?;
    let current_tree = current_commit.tree()?;
    let issues_tree = get_subtree(repo, &current_tree, "issues")?;
    let edges_tree = get_subtree(repo, &current_tree, "edges")?;

    let blob_content = serde_json::to_vec(edge)?;
    let blob_oid = repo.blob(&blob_content)?;

    // Edge filename: source_target_type
    let edge_name = format!("{}_{}_{}", edge.source, edge.target, edge.edge_type.as_str());

    let mut edges_builder = repo.treebuilder(Some(&edges_tree))?;
    edges_builder.insert(&edge_name, blob_oid, 0o100644)?;
    let new_edges_oid = edges_builder.write()?;

    // Build new root tree (keeping issues unchanged)
    let mut root_builder = repo.treebuilder(None)?;
    root_builder.insert("issues", issues_tree.id(), 0o040000)?;
    root_builder.insert("edges", new_edges_oid, 0o040000)?;
    let new_tree_oid = root_builder.write()?;
    let new_tree = repo.find_tree(new_tree_oid)?;
    let sig = repo.signature()?;

    repo.commit(
        Some(SNAPSHOT_REF),
        &sig,
        &sig,
        message,
        &new_tree,
        &[&current_commit],
    )?;

    Ok(())
}

/// Delete the snapshot ref (for purge)
pub fn delete_snapshot(repo: &Repository) -> Result<(), Error> {
    if let Ok(mut reference) = repo.find_reference(SNAPSHOT_REF) {
        reference.delete()?;
    }
    Ok(())
}

/// Get all existing issue IDs (for collision checking during create)
pub fn get_existing_ids(repo: &Repository) -> Result<std::collections::HashSet<String>, Error> {
    if !is_initialized(repo) {
        return Ok(std::collections::HashSet::new());
    }
    let issues = load_issues(repo)?;
    Ok(issues.keys().cloned().collect())
}

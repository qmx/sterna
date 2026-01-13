use git2::Repository;

use crate::error::Error;
use crate::snapshot;
use crate::types::{Edge, Issue};

pub fn run(remote: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;
    let remote_name = remote.unwrap_or_else(|| "origin".to_string());

    let mut git_remote = repo.find_remote(&remote_name)?;
    git_remote.fetch(&["refs/sterna/snapshot:refs/sterna/remote"], None, None)?;

    let remote_ref = repo.find_reference("refs/sterna/remote")?;
    let remote_commit = remote_ref.peel_to_commit()?;
    let remote_tree = remote_commit.tree()?;

    let local_issues = snapshot::load_issues(&repo)?;
    let local_edges = snapshot::load_edges(&repo)?;

    let mut issues_to_save: Vec<Issue> = Vec::new();
    let mut edges_to_add: Vec<Edge> = Vec::new();

    let issues_tree_entry = remote_tree
        .get_name("issues")
        .ok_or(Error::InvalidSnapshot)?;
    let issues_tree = issues_tree_entry.to_object(&repo)?.peel_to_tree()?;

    for entry in issues_tree.iter() {
        let blob = repo.find_blob(entry.id())?;
        let remote_issue = Issue::from_json(blob.content())?;

        let dominated = if let Some(existing) = local_issues.get(&remote_issue.id) {
            remote_issue.lamport > existing.lamport
                || (remote_issue.lamport == existing.lamport
                    && remote_issue.updated_at > existing.updated_at)
        } else {
            true
        };

        if dominated {
            issues_to_save.push(remote_issue);
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
            edges_to_add.push(remote_edge);
        }
    }

    // Single batch commit
    let issues_added = issues_to_save
        .iter()
        .filter(|i| !local_issues.contains_key(&i.id))
        .count();
    let issues_updated = issues_to_save.len() - issues_added;
    let edges_added = edges_to_add.len();

    if !issues_to_save.is_empty() || !edges_to_add.is_empty() {
        snapshot::merge_snapshot(
            &repo,
            &issues_to_save,
            &edges_to_add,
            &format!(
                "Pull from {}: {} issues, {} edges",
                remote_name,
                issues_to_save.len(),
                edges_to_add.len()
            ),
        )?;
    }

    repo.find_reference("refs/sterna/remote")?.delete()?;

    eprintln!(
        "Pulled from {}: {} issues added, {} issues updated, {} edges added",
        remote_name, issues_added, issues_updated, edges_added
    );

    Ok(())
}

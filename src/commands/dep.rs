use git2::Repository;

use crate::dag;
use crate::error::Error;
use crate::snapshot;
use crate::types::{Edge, EdgeType, SCHEMA_VERSION};

pub fn add(
    source: String,
    needs: Option<String>,
    blocks: Option<String>,
    relates_to: Option<String>,
    parent: Option<String>,
    duplicates: Option<String>,
) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    let source_id = snapshot::find_issue_id(&repo, &source)?;

    let (target_prefix, edge_type) = if let Some(t) = needs {
        (t, EdgeType::DependsOn)
    } else if let Some(t) = blocks {
        (t, EdgeType::Blocks)
    } else if let Some(t) = relates_to {
        (t, EdgeType::RelatesTo)
    } else if let Some(t) = parent {
        (t, EdgeType::ParentChild)
    } else if let Some(t) = duplicates {
        (t, EdgeType::Duplicates)
    } else {
        return Err(Error::NoEdgeTarget);
    };

    let target_id = snapshot::find_issue_id(&repo, &target_prefix)?;

    if source_id == target_id {
        return Err(Error::SelfReference(source_id));
    }

    if snapshot::edge_exists(&repo, &source_id, &target_id, edge_type)? {
        return Err(Error::DuplicateEdge(source_id, target_id));
    }

    let edges = snapshot::load_edges(&repo)?;
    if dag::would_create_cycle(&edges, &source_id, &target_id, edge_type) {
        return Err(Error::WouldCreateCycle(source_id, target_id));
    }

    let edge = Edge {
        schema_version: SCHEMA_VERSION,
        source: source_id.clone(),
        target: target_id.clone(),
        edge_type,
        created_at: chrono::Utc::now().timestamp(),
    };

    snapshot::save_edge(
        &repo,
        &edge,
        &format!("{} {} {}", source_id, edge_type.as_str(), target_id),
    )?;

    println!("{} {} {}", source_id, edge_type.as_str(), target_id);
    Ok(())
}

pub fn remove(
    source: String,
    needs: Option<String>,
    blocks: Option<String>,
    relates_to: Option<String>,
    parent: Option<String>,
    duplicates: Option<String>,
) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    let (target_prefix, edge_type) = if let Some(t) = needs {
        (t, EdgeType::DependsOn)
    } else if let Some(t) = blocks {
        (t, EdgeType::Blocks)
    } else if let Some(t) = relates_to {
        (t, EdgeType::RelatesTo)
    } else if let Some(t) = parent {
        (t, EdgeType::ParentChild)
    } else if let Some(t) = duplicates {
        (t, EdgeType::Duplicates)
    } else {
        return Err(Error::NoEdgeTarget);
    };

    let source_id = snapshot::find_issue_id(&repo, &source)?;
    let target_id = snapshot::find_issue_id(&repo, &target_prefix)?;

    let deleted = snapshot::delete_edge(
        &repo,
        &source_id,
        &target_id,
        edge_type,
        &format!("Remove edge: {} {} {}", source_id, edge_type.as_str(), target_id),
    )?;

    if deleted {
        println!("Removed: {} {} {}", source_id, edge_type.as_str(), target_id);
    } else {
        println!("Edge not found: {} {} {}", source_id, edge_type.as_str(), target_id);
    }
    Ok(())
}

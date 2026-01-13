use git2::Repository;

use crate::dag;
use crate::error::Error;
use crate::snapshot;
use crate::types::{Edge, EdgeType, SCHEMA_VERSION};

pub fn run(
    source: String,
    needs: Option<String>,
    blocks: Option<String>,
    relates_to: Option<String>,
    parent: Option<String>,
    duplicates: Option<String>,
) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    // Resolve source issue
    let source_id = snapshot::find_issue_id(&repo, &source)?;

    // Determine edge type and target
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

    // Resolve target issue
    let target_id = snapshot::find_issue_id(&repo, &target_prefix)?;

    // Check for self-reference
    if source_id == target_id {
        return Err(Error::SelfReference(source_id));
    }

    // Check for duplicate edge
    if snapshot::edge_exists(&repo, &source_id, &target_id, edge_type)? {
        return Err(Error::DuplicateEdge(source_id, target_id));
    }

    // Check for cycles
    let edges = snapshot::load_edges(&repo)?;
    if dag::would_create_cycle(&edges, &source_id, &target_id, edge_type) {
        return Err(Error::WouldCreateCycle(source_id, target_id));
    }

    // Create edge
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

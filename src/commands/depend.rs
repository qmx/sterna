use git2::Repository;

use crate::error::Error;
use crate::index::{EdgeIndex, IssueIndex};
use crate::storage;
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
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;

    let issue_index = IssueIndex::load(repo_path)?;
    let mut edge_index = EdgeIndex::load(repo_path)?;

    // Resolve source issue
    let (source_id, _) = issue_index.find_unique(&source)?;

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
    let (target_id, _) = issue_index.find_unique(&target_prefix)?;

    // Check for self-reference
    if source_id == target_id {
        return Err(Error::SelfReference(source_id));
    }

    // Check for duplicate edge
    if edge_index.exists(&source_id, &target_id, edge_type) {
        return Err(Error::DuplicateEdge(source_id, target_id));
    }

    // Create edge
    let edge = Edge {
        schema_version: SCHEMA_VERSION,
        source: source_id.clone(),
        target: target_id.clone(),
        edge_type,
        created_at: chrono::Utc::now().timestamp(),
    };

    let content = storage::serialize_edge(&edge)?;
    let oid = storage::write_blob(&repo, &content)?;

    edge_index.entries.push(crate::index::EdgeEntry {
        source: source_id.clone(),
        target: target_id.clone(),
        edge_type,
        oid,
    });
    edge_index.save(repo_path)?;

    println!(
        "{} {} {}",
        source_id,
        edge_type.as_str(),
        target_id
    );
    Ok(())
}

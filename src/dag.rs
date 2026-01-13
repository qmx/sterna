use std::collections::{HashMap, HashSet};

use crate::types::{Edge, EdgeType};

/// Check if adding a new edge would create a cycle in the dependency graph.
/// Only checks for DependsOn, Blocks, and ParentChild edges (directed relationships).
/// RelatesTo and Duplicates are symmetric and don't create cycles.
pub fn would_create_cycle(edges: &[Edge], source: &str, target: &str, edge_type: EdgeType) -> bool {
    // Only check cycle-forming edge types
    if matches!(edge_type, EdgeType::RelatesTo | EdgeType::Duplicates) {
        return false;
    }

    // Build adjacency list from existing edges
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for edge in edges {
        // Only consider cycle-forming edges
        if matches!(
            edge.edge_type,
            EdgeType::DependsOn | EdgeType::Blocks | EdgeType::ParentChild
        ) {
            adj.entry(edge.source.as_str())
                .or_default()
                .push(edge.target.as_str());
        }
    }

    // Add the proposed edge
    adj.entry(source).or_default().push(target);

    // DFS from source to see if we can reach source again (cycle)
    let mut visited = HashSet::new();
    let mut stack = vec![source];

    while let Some(node) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }

        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                if neighbor == source {
                    // Found a path back to source = cycle
                    return true;
                }
                if !visited.contains(neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Edge, SCHEMA_VERSION};

    fn make_edge(source: &str, target: &str, edge_type: EdgeType) -> Edge {
        Edge {
            schema_version: SCHEMA_VERSION,
            source: source.to_string(),
            target: target.to_string(),
            edge_type,
            created_at: 0,
        }
    }

    #[test]
    fn test_no_cycle_empty() {
        let edges: Vec<Edge> = vec![];
        assert!(!would_create_cycle(&edges, "a", "b", EdgeType::DependsOn));
    }

    #[test]
    fn test_direct_cycle() {
        let edges = vec![make_edge("a", "b", EdgeType::DependsOn)];
        // b -> a would create a->b->a cycle
        assert!(would_create_cycle(&edges, "b", "a", EdgeType::DependsOn));
    }

    #[test]
    fn test_indirect_cycle() {
        let edges = vec![
            make_edge("a", "b", EdgeType::DependsOn),
            make_edge("b", "c", EdgeType::DependsOn),
        ];
        // c -> a would create a->b->c->a cycle
        assert!(would_create_cycle(&edges, "c", "a", EdgeType::DependsOn));
    }

    #[test]
    fn test_relates_to_no_cycle() {
        let edges = vec![make_edge("a", "b", EdgeType::RelatesTo)];
        // RelatesTo edges don't form cycles
        assert!(!would_create_cycle(&edges, "b", "a", EdgeType::RelatesTo));
    }
}

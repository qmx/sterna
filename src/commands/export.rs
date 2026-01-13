use std::fs;

use git2::Repository;
use serde::Serialize;

use crate::error::Error;
use crate::snapshot;
use crate::types::{Edge, Issue};

#[derive(Serialize)]
struct Export {
    version: u32,
    exported_at: i64,
    issues: Vec<Issue>,
    edges: Vec<Edge>,
}

pub fn run(output: Option<String>) -> Result<(), Error> {
    let repo = Repository::discover(".")?;

    let issues: Vec<Issue> = snapshot::load_issues(&repo)?.into_values().collect();
    let edges = snapshot::load_edges(&repo)?;

    let export = Export {
        version: 1,
        exported_at: chrono::Utc::now().timestamp(),
        issues,
        edges,
    };

    let json = serde_json::to_string_pretty(&export)?;

    match output {
        Some(path) => {
            fs::write(&path, &json)?;
            eprintln!("Exported to {}", path);
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

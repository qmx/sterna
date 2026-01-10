use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::Error;
use crate::types::EdgeType;

pub struct IssueIndex {
    pub entries: HashMap<String, git2::Oid>,
}

impl IssueIndex {
    pub fn load(repo_path: &Path) -> Result<Self, Error> {
        let path = repo_path.join("sterna/index/issues");
        let mut entries = HashMap::new();

        if !repo_path.join("sterna").exists() {
            return Err(Error::NotInitialized);
        }

        if path.exists() {
            for line in fs::read_to_string(&path)?.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    entries.insert(parts[0].to_string(), git2::Oid::from_str(parts[1])?);
                }
            }
        }
        Ok(Self { entries })
    }

    pub fn save(&self, repo_path: &Path) -> Result<(), Error> {
        let path = repo_path.join("sterna/index/issues");
        let tmp = path.with_extension("tmp");

        let mut lines: Vec<_> = self
            .entries
            .iter()
            .map(|(id, oid)| format!("{} {}", id, oid))
            .collect();
        lines.sort();

        let content = lines.join("\n");
        fs::write(&tmp, &content)?;
        fs::rename(tmp, path)?;
        Ok(())
    }

    pub fn find_unique(&self, prefix: &str) -> Result<(String, git2::Oid), Error> {
        let matches: Vec<_> = self
            .entries
            .iter()
            .filter(|(id, _)| id.starts_with(prefix))
            .collect();

        match matches.len() {
            0 => Err(Error::NotFound(prefix.to_string())),
            1 => {
                let (id, oid) = matches[0];
                Ok((id.clone(), *oid))
            }
            _ => Err(Error::AmbiguousId(
                prefix.to_string(),
                matches.iter().map(|(id, _)| (*id).clone()).collect(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeEntry {
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub oid: git2::Oid,
}

pub struct EdgeIndex {
    pub entries: Vec<EdgeEntry>,
}

impl EdgeIndex {
    pub fn load(repo_path: &Path) -> Result<Self, Error> {
        let path = repo_path.join("sterna/index/edges");
        let mut entries = Vec::new();

        if !repo_path.join("sterna").exists() {
            return Err(Error::NotInitialized);
        }

        if path.exists() {
            for line in fs::read_to_string(&path)?.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 4 {
                    entries.push(EdgeEntry {
                        source: parts[0].to_string(),
                        target: parts[1].to_string(),
                        edge_type: EdgeType::from_str(parts[2])?,
                        oid: git2::Oid::from_str(parts[3])?,
                    });
                }
            }
        }
        Ok(Self { entries })
    }

    pub fn save(&self, repo_path: &Path) -> Result<(), Error> {
        let path = repo_path.join("sterna/index/edges");
        let tmp = path.with_extension("tmp");

        let mut lines: Vec<_> = self
            .entries
            .iter()
            .map(|e| format!("{} {} {} {}", e.source, e.target, e.edge_type.as_str(), e.oid))
            .collect();
        lines.sort();

        let content = lines.join("\n");
        fs::write(&tmp, &content)?;
        fs::rename(tmp, path)?;
        Ok(())
    }

    pub fn find_by_source(&self, source: &str) -> Vec<&EdgeEntry> {
        self.entries.iter().filter(|e| e.source == source).collect()
    }

    pub fn find_by_target(&self, target: &str) -> Vec<&EdgeEntry> {
        self.entries.iter().filter(|e| e.target == target).collect()
    }

    pub fn exists(&self, source: &str, target: &str, edge_type: EdgeType) -> bool {
        self.entries
            .iter()
            .any(|e| e.source == source && e.target == target && e.edge_type == edge_type)
    }
}

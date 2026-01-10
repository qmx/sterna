use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::Error;

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

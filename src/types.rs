use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::error::Error;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Issue {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub priority: Priority,
    pub issue_type: IssueType,
    pub labels: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub lamport: u64,
    pub editor: String,
    pub claimed: bool,
    pub claim_context: Option<String>,
    pub claimed_at: Option<u64>,
    pub reason: Option<String>,
}

impl Issue {
    pub fn from_json(data: &[u8]) -> Result<Self, Error> {
        let value: serde_json::Value = serde_json::from_slice(data)?;
        let version = value["schema_version"].as_u64().unwrap_or(0) as u32;
        if version != SCHEMA_VERSION {
            return Err(Error::SchemaMismatch {
                expected: SCHEMA_VERSION,
                found: version,
            });
        }
        Ok(serde_json::from_value(value)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,
    InProgress,
    Closed,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Medium = 2,
    Low = 3,
    Backlog = 4,
}

impl Priority {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "critical" | "0" => Ok(Priority::Critical),
            "high" | "1" => Ok(Priority::High),
            "medium" | "2" => Ok(Priority::Medium),
            "low" | "3" => Ok(Priority::Low),
            "backlog" | "4" => Ok(Priority::Backlog),
            _ => Err(Error::InvalidPriority(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Critical => "critical",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
            Priority::Backlog => "backlog",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    Epic,
    Task,
    Bug,
    Feature,
    Chore,
}

impl IssueType {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "epic" => Ok(IssueType::Epic),
            "task" => Ok(IssueType::Task),
            "bug" => Ok(IssueType::Bug),
            "feature" => Ok(IssueType::Feature),
            "chore" => Ok(IssueType::Chore),
            _ => Err(Error::InvalidIssueType(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            IssueType::Epic => "epic",
            IssueType::Task => "task",
            IssueType::Bug => "bug",
            IssueType::Feature => "feature",
            IssueType::Chore => "chore",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub schema_version: u32,
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub created_at: i64,
}

impl Edge {
    pub fn from_json(data: &[u8]) -> Result<Self, Error> {
        let value: serde_json::Value = serde_json::from_slice(data)?;
        let version = value["schema_version"].as_u64().unwrap_or(0) as u32;
        if version != SCHEMA_VERSION {
            return Err(Error::SchemaMismatch {
                expected: SCHEMA_VERSION,
                found: version,
            });
        }
        Ok(serde_json::from_value(value)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    DependsOn,
    Blocks,
    ParentChild,
    RelatesTo,
    Duplicates,
}

impl EdgeType {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "depends_on" | "dependson" | "needs" => Ok(EdgeType::DependsOn),
            "blocks" => Ok(EdgeType::Blocks),
            "parent_child" | "parentchild" | "parent" => Ok(EdgeType::ParentChild),
            "relates_to" | "relatesto" | "relates" => Ok(EdgeType::RelatesTo),
            "duplicates" => Ok(EdgeType::Duplicates),
            _ => Err(Error::InvalidEdgeType(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::DependsOn => "depends_on",
            EdgeType::Blocks => "blocks",
            EdgeType::ParentChild => "parent_child",
            EdgeType::RelatesTo => "relates_to",
            EdgeType::Duplicates => "duplicates",
        }
    }
}

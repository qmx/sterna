use std::fmt;

#[derive(Debug)]
pub enum Error {
    Git(git2::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    SchemaMismatch { expected: u32, found: u32 },
    NoIdentity(String),
    BareRepo,
    NotFound(String),
    AmbiguousId(String, Vec<String>),
    NotInitialized,
    AlreadyClaimed(String),
    NotClaimed(String),
    IsClosed(String),
    AlreadyClosed(String),
    NotClosed(String),
    InvalidPriority(String),
    InvalidIssueType(String),
    InvalidEdgeType(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Git(e) => write!(f, "Git error: {}", e),
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::SchemaMismatch { expected, found } => {
                write!(f, "Schema mismatch: expected {}, found {}", expected, found)
            }
            Error::NoIdentity(msg) => write!(f, "No identity: {}", msg),
            Error::BareRepo => write!(f, "Cannot operate on bare repository"),
            Error::NotFound(id) => write!(f, "Issue not found: {}", id),
            Error::AmbiguousId(prefix, matches) => {
                write!(f, "Ambiguous ID '{}': matches {:?}", prefix, matches)
            }
            Error::NotInitialized => write!(f, "Sterna not initialized. Run 'st init' first."),
            Error::AlreadyClaimed(id) => write!(f, "Issue {} is already claimed", id),
            Error::NotClaimed(id) => write!(f, "Issue {} is not claimed", id),
            Error::IsClosed(id) => write!(f, "Issue {} is closed", id),
            Error::AlreadyClosed(id) => write!(f, "Issue {} is already closed", id),
            Error::NotClosed(id) => write!(f, "Issue {} is not closed", id),
            Error::InvalidPriority(p) => write!(f, "Invalid priority: {}", p),
            Error::InvalidIssueType(t) => write!(f, "Invalid issue type: {}", t),
            Error::InvalidEdgeType(t) => write!(f, "Invalid edge type: {}", t),
        }
    }
}

impl std::error::Error for Error {}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        Error::Git(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

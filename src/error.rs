use std::fmt;

#[derive(Debug)]
pub enum Error {
    Git(git2::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    SchemaMismatch { expected: u32, found: u32 },
    NoIdentity(String),
    NotFound(String),
    AmbiguousId(String, Vec<String>),
    NotInitialized,
    AlreadyInitialized,
    CorruptedSnapshot(String),
    AlreadyClaimed(String),
    NotClaimed(String),
    IsClosed(String),
    AlreadyClosed(String),
    NotClosed(String),
    InvalidPriority(String),
    InvalidIssueType(String),
    NoEdgeTarget,
    SelfReference(String),
    DuplicateEdge(String, String),
    WouldCreateCycle(String, String),
    InvalidSnapshot,
    LockFailed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Git(e) => write!(f, "Git error: {e}"),
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::Json(e) => write!(f, "JSON error: {e}"),
            Error::SchemaMismatch { expected, found } => {
                write!(f, "Schema mismatch: expected {expected}, found {found}")
            }
            Error::NoIdentity(msg) => write!(f, "No identity: {msg}"),
            Error::NotFound(id) => write!(f, "Issue not found: {id}"),
            Error::AmbiguousId(prefix, matches) => {
                write!(f, "Ambiguous ID '{prefix}': matches {matches:?}")
            }
            Error::NotInitialized => write!(f, "Sterna not initialized. Run 'st init' first."),
            Error::AlreadyInitialized => write!(f, "Sterna is already initialized"),
            Error::CorruptedSnapshot(msg) => write!(f, "Corrupted snapshot: {msg}"),
            Error::AlreadyClaimed(id) => write!(f, "Issue {id} is already claimed"),
            Error::NotClaimed(id) => write!(f, "Issue {id} is not claimed"),
            Error::IsClosed(id) => write!(f, "Issue {id} is closed"),
            Error::AlreadyClosed(id) => write!(f, "Issue {id} is already closed"),
            Error::NotClosed(id) => write!(f, "Issue {id} is not closed"),
            Error::InvalidPriority(p) => write!(f, "Invalid priority: {p}"),
            Error::InvalidIssueType(t) => write!(f, "Invalid issue type: {t}"),
            Error::NoEdgeTarget => write!(
                f,
                "Must specify one of: --needs, --blocks, --relates-to, --parent, --duplicates"
            ),
            Error::SelfReference(id) => write!(f, "Cannot create edge to self: {id}"),
            Error::DuplicateEdge(s, t) => write!(f, "Edge already exists: {s} -> {t}"),
            Error::WouldCreateCycle(s, t) => write!(f, "Would create cycle: {s} -> {t}"),
            Error::InvalidSnapshot => write!(f, "Invalid snapshot format"),
            Error::LockFailed(msg) => write!(f, "Failed to acquire lock: {msg}"),
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

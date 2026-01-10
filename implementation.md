# Sterna Implementation Plan

**Goal:** Dogfood ASAP. After Phase 2, use Sterna to track remaining work.

---

## Phase 0: Dev Environment

Set up Nix flake for reproducible development.

**Files to create:**
```
sterna/
├── flake.nix
└── .envrc
```

**flake.nix:**
```nix
{
  description = "Sterna - Git-native issue tracker";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
          ];
        };
      }
    );
}
```

**.envrc:**
```
use flake
```

**Setup:**
```bash
nix flake init  # if starting fresh
direnv allow
```

---

## Phase 1: Bootstrap

Get a minimal working CLI that can create, list, and view issues.

### 1.1 Project Setup

**Files to create:**
```
sterna/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── types.rs
    ├── storage.rs
    ├── index.rs
    ├── id.rs
    └── commands/
        ├── mod.rs
        ├── init.rs
        ├── create.rs
        ├── list.rs
        └── get.rs
```

**Cargo.toml dependencies:**
```toml
[dependencies]
git2 = "0.20"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_repr = "0.1"
clap = { version = "4.5", features = ["derive"] }
sha1 = "0.10"
hex = "0.4"
chrono = "0.4"
```

**main.rs structure:**
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "st")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Create { title: String, #[arg(short, long)] description: Option<String>, ... },
    List { #[arg(long)] status: Option<String>, ... },
    Get { id: String },
    // ... more commands added in later phases
}
```

### 1.2 Core Types (`src/types.rs`)

```rust
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status { Open, InProgress, Closed }

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Priority { Critical = 0, High = 1, Medium = 2, Low = 3, Backlog = 4 }

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum IssueType { Epic, Task, Bug, Feature, Chore }
```

**Schema validation on read:**
```rust
impl Issue {
    pub fn from_json(data: &[u8]) -> Result<Self, Error> {
        let value: serde_json::Value = serde_json::from_slice(data)?;
        let version = value["schema_version"].as_u64().unwrap_or(0) as u32;
        if version != SCHEMA_VERSION {
            return Err(Error::SchemaMismatch { expected: SCHEMA_VERSION, found: version });
        }
        Ok(serde_json::from_value(value)?)
    }
}
```

### 1.3 Storage Layer (`src/storage.rs`)

**Get editor identity:**
```rust
pub fn get_editor() -> Result<String, Error> {
    let repo = git2::Repository::discover(".")?;
    let config = repo.config()?;
    config.get_string("user.email")
        .map_err(|_| Error::NoIdentity("Set git config user.email".into()))
}
```

**Write blob to Git:**
```rust
pub fn write_blob(repo: &Repository, content: &[u8]) -> Result<git2::Oid, Error> {
    Ok(repo.blob(content)?)
}

pub fn read_blob(repo: &Repository, oid: git2::Oid) -> Result<Vec<u8>, Error> {
    let blob = repo.find_blob(oid)?;
    Ok(blob.content().to_vec())
}
```

**Serialize issue:**
```rust
pub fn serialize_issue(issue: &Issue) -> Result<Vec<u8>, Error> {
    // Compact JSON - pipe through `jq .` for pretty viewing
    Ok(serde_json::to_vec(issue)?)
}
```

### 1.4 Index Management (`src/index.rs`)

**Index file format** (`sterna/index/issues`):
```
st-a3f8 abcdef1234567890abcdef1234567890abcdef12
st-b4f9 1234567890abcdef1234567890abcdef12345678
```

**Parse index:**
```rust
pub struct IssueIndex {
    entries: HashMap<String, git2::Oid>,
}

impl IssueIndex {
    pub fn load(repo_path: &Path) -> Result<Self, Error> {
        let path = repo_path.join("sterna/index/issues");
        let mut entries = HashMap::new();
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
        let content: String = self.entries.iter()
            .map(|(id, oid)| format!("{} {}", id, oid))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&tmp, content)?;
        fs::rename(tmp, path)?;  // Atomic update
        Ok(())
    }
}
```

### 1.5 ID Generation (`src/id.rs`)

```rust
use sha1::{Sha1, Digest};

pub fn generate_id(title: &str, description: &str, editor: &str, existing_ids: &HashSet<String>) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let mut length = 4;

    loop {
        let input = format!("{}{}{}{}{}", title, description, editor, timestamp, length);
        let hash = Sha1::digest(input.as_bytes());
        let hex = hex::encode(hash);
        let id = format!("st-{}", &hex[..length]);

        if !existing_ids.contains(&id) {
            return id;
        }
        length += 1;
        if length > 8 {
            panic!("ID collision after 8 chars - should not happen");
        }
    }
}
```

### 1.6 `st init` Command

```rust
pub fn init() -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;

    let index_dir = repo_path.join("sterna/index");
    fs::create_dir_all(&index_dir)?;

    // Create empty index files
    fs::write(index_dir.join("issues"), "")?;
    fs::write(index_dir.join("edges"), "")?;

    println!("Initialized Sterna in {}", repo_path.display());
    Ok(())
}
```

### 1.7 `st create` Command

```rust
pub fn create(title: String, description: Option<String>, priority: Option<String>,
              issue_type: Option<String>, labels: Vec<String>) -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let editor = storage::get_editor()?;

    let mut index = IssueIndex::load(repo_path)?;
    let existing_ids: HashSet<String> = index.entries.keys().cloned().collect();

    let id = id::generate_id(&title, description.as_deref().unwrap_or(""), &editor, &existing_ids);
    let now = chrono::Utc::now().timestamp();

    let issue = Issue {
        schema_version: SCHEMA_VERSION,
        id: id.clone(),
        title,
        description: description.unwrap_or_default(),
        status: Status::Open,
        priority: parse_priority(priority)?,
        issue_type: parse_issue_type(issue_type)?,
        labels,
        created_at: now,
        updated_at: now,
        lamport: 1,
        editor,
        claimed: false,
        claim_context: None,
        claimed_at: None,
        reason: None,
    };

    let content = storage::serialize_issue(&issue)?;
    let oid = storage::write_blob(&repo, &content)?;

    index.entries.insert(id.clone(), oid);
    index.save(repo_path)?;

    println!("{}", id);
    Ok(())
}
```

### 1.8 `st list` Command

```rust
pub fn list(status: Option<String>, issue_type: Option<String>) -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let index = IssueIndex::load(repo_path)?;

    let mut issues: Vec<Issue> = Vec::new();
    for (_, oid) in &index.entries {
        let data = storage::read_blob(&repo, *oid)?;
        let issue = Issue::from_json(&data)?;

        // Filter
        if let Some(ref s) = status {
            if format!("{:?}", issue.status).to_lowercase() != s.to_lowercase() {
                continue;
            }
        }
        if let Some(ref t) = issue_type {
            if format!("{:?}", issue.issue_type).to_lowercase() != t.to_lowercase() {
                continue;
            }
        }
        issues.push(issue);
    }

    // Sort by priority, then created_at
    issues.sort_by(|a, b| {
        (a.priority as u8, a.created_at).cmp(&(b.priority as u8, b.created_at))
    });

    // Print table
    println!("{:<12} {:<12} {:<8} {:<10} {}", "ID", "STATUS", "PRI", "TYPE", "TITLE");
    println!("{}", "-".repeat(60));
    for issue in issues {
        println!("{:<12} {:<12} {:<8} {:<10} {}",
            issue.id,
            format!("{:?}", issue.status).to_lowercase(),
            format!("{:?}", issue.priority).to_lowercase(),
            format!("{:?}", issue.issue_type).to_lowercase(),
            truncate(&issue.title, 40)
        );
    }
    Ok(())
}
```

### 1.9 `st get` Command

```rust
pub fn get(id_prefix: String) -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let index = IssueIndex::load(repo_path)?;

    // Find matching ID
    let matches: Vec<_> = index.entries.iter()
        .filter(|(id, _)| id.starts_with(&id_prefix))
        .collect();

    match matches.len() {
        0 => Err(Error::NotFound(id_prefix)),
        1 => {
            let (id, oid) = matches[0];
            let data = storage::read_blob(&repo, *oid)?;
            let issue = Issue::from_json(&data)?;
            print_issue(&issue);
            Ok(())
        }
        _ => Err(Error::AmbiguousId(id_prefix, matches.iter().map(|(id, _)| id.clone()).collect())),
    }
}

fn print_issue(issue: &Issue) {
    println!("ID:          {}", issue.id);
    println!("Title:       {}", issue.title);
    println!("Status:      {:?}", issue.status);
    println!("Priority:    {:?}", issue.priority);
    println!("Type:        {:?}", issue.issue_type);
    println!("Labels:      {}", issue.labels.join(", "));
    println!("Created:     {}", format_timestamp(issue.created_at));
    println!("Updated:     {}", format_timestamp(issue.updated_at));
    println!("Editor:      {}", issue.editor);
    println!("Claimed:     {}", issue.claimed);
    if let Some(ref ctx) = issue.claim_context {
        println!("Context:     {}", ctx);
    }
    if let Some(ref reason) = issue.reason {
        println!("Reason:      {}", reason);
    }
    println!("\n{}", issue.description);
}
```

---

## Phase 2: Work Cycle

Complete the claim/close workflow to enable dogfooding.

### 2.1 `st claim` Command

**Add to commands:**
```rust
pub fn claim(id_prefix: String, context: Option<String>) -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let mut index = IssueIndex::load(repo_path)?;

    let (id, oid) = find_unique_issue(&index, &id_prefix)?;
    let data = storage::read_blob(&repo, oid)?;
    let mut issue = Issue::from_json(&data)?;

    if issue.claimed {
        return Err(Error::AlreadyClaimed(id));
    }
    if issue.status == Status::Closed {
        return Err(Error::IsClosed(id));
    }

    issue.claimed = true;
    issue.status = Status::InProgress;
    issue.claim_context = context;
    issue.claimed_at = Some(chrono::Utc::now().timestamp() as u64);
    issue.lamport += 1;
    issue.updated_at = chrono::Utc::now().timestamp();
    issue.editor = storage::get_editor()?;

    let content = storage::serialize_issue(&issue)?;
    let new_oid = storage::write_blob(&repo, &content)?;
    index.entries.insert(id.clone(), new_oid);
    index.save(repo_path)?;

    println!("Claimed {}", id);
    Ok(())
}
```

### 2.2 `st release` Command

```rust
pub fn release(id_prefix: String, reason: Option<String>) -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let mut index = IssueIndex::load(repo_path)?;

    let (id, oid) = find_unique_issue(&index, &id_prefix)?;
    let data = storage::read_blob(&repo, oid)?;
    let mut issue = Issue::from_json(&data)?;

    if !issue.claimed {
        return Err(Error::NotClaimed(id));
    }

    issue.claimed = false;
    issue.status = Status::Open;
    issue.claim_context = None;
    issue.claimed_at = None;
    issue.reason = reason;
    issue.lamport += 1;
    issue.updated_at = chrono::Utc::now().timestamp();
    issue.editor = storage::get_editor()?;

    let content = storage::serialize_issue(&issue)?;
    let new_oid = storage::write_blob(&repo, &content)?;
    index.entries.insert(id.clone(), new_oid);
    index.save(repo_path)?;

    println!("Released {}", id);
    Ok(())
}
```

### 2.3 `st close` Command

```rust
pub fn close(id_prefix: String, reason: Option<String>) -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let mut index = IssueIndex::load(repo_path)?;

    let (id, oid) = find_unique_issue(&index, &id_prefix)?;
    let data = storage::read_blob(&repo, oid)?;
    let mut issue = Issue::from_json(&data)?;

    if issue.status == Status::Closed {
        return Err(Error::AlreadyClosed(id));
    }

    issue.status = Status::Closed;
    issue.claimed = false;
    issue.claim_context = None;
    issue.claimed_at = None;
    issue.reason = reason;
    issue.lamport += 1;
    issue.updated_at = chrono::Utc::now().timestamp();
    issue.editor = storage::get_editor()?;

    let content = storage::serialize_issue(&issue)?;
    let new_oid = storage::write_blob(&repo, &content)?;
    index.entries.insert(id.clone(), new_oid);
    index.save(repo_path)?;

    println!("Closed {}", id);
    Ok(())
}
```

### 2.4 `st reopen` Command

```rust
pub fn reopen(id_prefix: String, reason: Option<String>) -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let mut index = IssueIndex::load(repo_path)?;

    let (id, oid) = find_unique_issue(&index, &id_prefix)?;
    let data = storage::read_blob(&repo, oid)?;
    let mut issue = Issue::from_json(&data)?;

    if issue.status != Status::Closed {
        return Err(Error::NotClosed(id));
    }

    issue.status = Status::Open;
    issue.reason = reason;
    issue.lamport += 1;
    issue.updated_at = chrono::Utc::now().timestamp();
    issue.editor = storage::get_editor()?;

    let content = storage::serialize_issue(&issue)?;
    let new_oid = storage::write_blob(&repo, &content)?;
    index.entries.insert(id.clone(), new_oid);
    index.save(repo_path)?;

    println!("Reopened {}", id);
    Ok(())
}
```

### 2.5 `st ready` Command

```rust
pub fn ready() -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let index = IssueIndex::load(repo_path)?;

    let mut issues: Vec<Issue> = Vec::new();
    for (_, oid) in &index.entries {
        let data = storage::read_blob(&repo, *oid)?;
        let issue = Issue::from_json(&data)?;

        // Ready = open AND not claimed
        if issue.status == Status::Open && !issue.claimed {
            issues.push(issue);
        }
    }

    // Sort by priority
    issues.sort_by_key(|i| i.priority as u8);

    println!("{:<12} {:<8} {:<10} {}", "ID", "PRI", "TYPE", "TITLE");
    println!("{}", "-".repeat(50));
    for issue in issues {
        println!("{:<12} {:<8} {:<10} {}",
            issue.id,
            format!("{:?}", issue.priority).to_lowercase(),
            format!("{:?}", issue.issue_type).to_lowercase(),
            truncate(&issue.title, 40)
        );
    }
    Ok(())
}
```

### 2.6 `st update` Command

```rust
pub fn update(id_prefix: String, title: Option<String>, description: Option<String>,
              priority: Option<String>, issue_type: Option<String>,
              labels: Option<Vec<String>>) -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    let repo_path = repo.workdir().ok_or(Error::BareRepo)?;
    let mut index = IssueIndex::load(repo_path)?;

    let (id, oid) = find_unique_issue(&index, &id_prefix)?;
    let data = storage::read_blob(&repo, oid)?;
    let mut issue = Issue::from_json(&data)?;

    if let Some(t) = title { issue.title = t; }
    if let Some(d) = description { issue.description = d; }
    if let Some(p) = priority { issue.priority = parse_priority(Some(p))?; }
    if let Some(t) = issue_type { issue.issue_type = parse_issue_type(Some(t))?; }
    if let Some(l) = labels { issue.labels = l; }

    issue.lamport += 1;
    issue.updated_at = chrono::Utc::now().timestamp();
    issue.editor = storage::get_editor()?;

    let content = storage::serialize_issue(&issue)?;
    let new_oid = storage::write_blob(&repo, &content)?;
    index.entries.insert(id.clone(), new_oid);
    index.save(repo_path)?;

    println!("Updated {}", id);
    Ok(())
}
```

---

## === DOGFOODING STARTS HERE ===

After Phase 2, create Sterna issues for all remaining work:
- `st create "Add Edge type and edges index" --type task`
- `st create "Implement st depend command" --type task`
- `st create "Add cycle detection for DAG" --type task`
- etc.

---

## Phase 3: Dependencies

### 3.1 Edge Type (`src/types.rs`)

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub schema_version: u32,
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub created: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    DependsOn,  // source needs target done first
    Blocks,     // source blocks target
    ParentChild,
    RelatesTo,
    Duplicates,
}
```

### 3.2 Edge Index (`src/index.rs`)

**Format** (`sterna/index/edges`):
```
st-a3f8 st-b4f9 depends_on abcdef1234...
```

```rust
pub struct EdgeIndex {
    entries: Vec<(String, String, EdgeType, git2::Oid)>,
}
```

### 3.3 `st depend` Command

```rust
pub fn depend(source: String, needs: Option<String>, blocks: Option<String>,
              relates_to: Option<String>, parent: Option<String>,
              duplicates: Option<String>) -> Result<(), Error> {
    // Determine edge type and target
    let (target, edge_type) = if let Some(t) = needs {
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
        return Err(Error::NoEdgeType);
    };

    // Validate both issues exist
    // Check for cycles (if applicable)
    // Create edge, save to index
}
```

### 3.4 Cycle Detection (`src/dag.rs`)

```rust
pub fn would_create_cycle(edges: &EdgeIndex, new_edge: &Edge) -> bool {
    // Only check for DependsOn, Blocks, ParentChild
    if matches!(new_edge.edge_type, EdgeType::RelatesTo) {
        return false;
    }

    // Build adjacency list
    // DFS from target to see if we can reach source
    // If yes, adding this edge creates a cycle
}
```

### 3.5 Update `st ready`

Add dependency check:
```rust
fn is_blocked(issue_id: &str, edges: &EdgeIndex, issues: &IssueIndex) -> bool {
    // Find all edges where this issue depends on something
    // Check if those dependencies are closed
    // Recursively check blockers
}
```

---

## Phase 4: Data Management

### 4.1 `st export`

```rust
#[derive(Serialize)]
struct Export {
    version: u32,
    exported_at: i64,
    issues: Vec<Issue>,
    edges: Vec<Edge>,
}
```

### 4.2 `st import`

- Parse JSON
- For each issue: if ID exists, LWW merge by Lamport; else insert
- For each edge: union (skip duplicates)

### 4.3 `st purge`

- Run export first
- Prompt "This will remove all Sterna data. Continue? [y/N]"
- Delete `sterna/` directory
- Delete `refs/sterna/*`
- Delete orphaned blobs (or let git gc handle it)

---

## Phase 5: Sync

### 5.1 Snapshot Structure

```rust
pub struct Snapshot {
    pub schema_version: u32,
    pub version: u64,
    pub created_at: i64,
    pub lamport: u64,
    pub issue_hashes: Vec<String>,
    pub edge_hashes: Vec<String>,
}
```

**Git tree structure:**
```
refs/sterna/snapshot -> commit -> tree
                                  ├── snapshot.json
                                  ├── issues/
                                  │   ├── st-a3f8 -> blob
                                  │   └── st-b4f9 -> blob
                                  └── edges/
                                      └── ... -> blob
```

### 5.2 `st pull`

1. `git fetch origin refs/sterna/snapshot:refs/sterna/remote`
2. Load remote snapshot
3. Merge issues (LWW by Lamport)
4. Merge edges (union)
5. Rebuild local index from merged state
6. Create new local snapshot

### 5.3 `st push`

1. Build snapshot from current index
2. Create commit with tree
3. Update `refs/sterna/snapshot`
4. `git push origin refs/sterna/snapshot`

---

## Phase 6: Agent Integration

### 6.1 `st onboard`

Check `~/.config/sterna/onboard.md` - if exists, print it. Otherwise:
```
Sterna: Git-native issue tracker. Key commands:
- st ready     : show available work
- st claim <id>: take an issue
- st close <id>: finish an issue
- st prime     : full reference
```

### 6.2 `st prime`

Check `~/.config/sterna/prime.md` - if exists, print it. Otherwise print full command reference + `st ready` output.

### 6.3 `--json` Flag

Add to list, get, ready:
```rust
if args.json {
    println!("{}", serde_json::to_string_pretty(&issues)?);
} else {
    // table output
}
```

---

## File Summary

```
src/
├── main.rs           # CLI entry, clap setup
├── types.rs          # Issue, Edge, Snapshot, enums
├── storage.rs        # Git blob read/write, get_editor
├── index.rs          # IssueIndex, EdgeIndex
├── id.rs             # ID generation
├── dag.rs            # Cycle detection
├── error.rs          # Error types
└── commands/
    ├── mod.rs
    ├── init.rs
    ├── create.rs
    ├── list.rs
    ├── get.rs
    ├── claim.rs
    ├── release.rs
    ├── close.rs
    ├── reopen.rs
    ├── ready.rs
    ├── update.rs
    ├── depend.rs
    ├── export.rs
    ├── import.rs
    ├── purge.rs
    ├── pull.rs
    ├── push.rs
    ├── sync.rs
    ├── onboard.rs
    └── prime.rs
```


# Sterna Architecture Document

## Overview

Sterna is a local-first, distributed issue tracker. It stores issues as Git objects, uses CRDTs for conflict-free merging, and supports dependency tracking with DAG validation.

**Core Principles:**
- No daemon - all operations are atomic CLI invocations
- No SQLite - pure file-based storage
- No auto-sync hooks - user explicitly controls remote operations
- Git-native - issues are stored as raw Git objects
- Binary claims with optional context - Simple taken/not-taken state
- Non-invasive - no automatic setup, user controls all integration

## Data Model

### Issue

```rust
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
    pub reason: Option<String>,  // Reason for last state change (close/release/reopen)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,        // JSON: "open"
    InProgress,  // JSON: "in_progress"
    Closed,      // JSON: "closed"
}

// Note: uses serde_repr for numeric serialization
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Priority {
    Critical = 0,  // JSON: 0
    High = 1,      // JSON: 1
    Medium = 2,    // JSON: 2
    Low = 3,       // JSON: 3
    Backlog = 4,   // JSON: 4
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    Epic,     // JSON: "epic"
    Task,     // JSON: "task"
    Bug,      // JSON: "bug"
    Feature,  // JSON: "feature"
    Chore,    // JSON: "chore"
}
```

**Status Values:**
- `open` - Unclaimed, available
- `in_progress` - Claimed and being worked on (status and claimed move together)
- `closed` - Completed or abandoned

**Priority Values (numeric in JSON):**
- `0` - Critical
- `1` - High
- `2` - Medium
- `3` - Low
- `4` - Backlog

**IssueType Values:**
- `epic` - Large initiative
- `task` - Regular work item
- `bug` - Defect
- `feature` - Enhancement request
- `chore` - Maintenance task

### Edge

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
    DependsOn,    // JSON: "depends_on"
    Blocks,       // JSON: "blocks"
    ParentChild,  // JSON: "parent_child"
    RelatesTo,    // JSON: "relates_to"
    Duplicates,   // JSON: "duplicates"
}
```

**Edge Types:**

| Type | Semantics | Blocks? | Cycle Checked? |
|------|-----------|---------|-----------------|
| `depends_on` | A is blocked by B | Yes | Yes |
| `blocks` | A blocks B (alias) | Yes | Yes |
| `parent_child` | A is child of B | Yes | Yes |
| `relates_to` | A is related to B | No | No (bidirectional) |
| `duplicates` | A duplicates B | No | Yes |

### Snapshot

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snapshot {
    pub schema_version: u32,
    pub version: u64,
    pub created_at: i64,
    pub lamport: u64,
    pub issue_hashes: Vec<String>,
    pub edge_hashes: Vec<String>,
}
```

## Schema Versioning

All JSON payloads include `schema_version: u32`. Current version: **1**

**Behavior:** If `schema_version` doesn't match expected, error and stop. No automatic migration.

## History Reconstruction

Issue history is preserved in Git, not in the issue payload itself. Each snapshot commit captures the full state at that moment.

**To reconstruct history for an issue:**
1. Walk snapshot commits from `refs/sterna/snapshot` backwards
2. For each snapshot, find the issue hash for the target ID
3. Read each blob to see the issue state at that point
4. Diff successive states to identify changes

**Future command:** `st history <id>` - walks snapshots and displays state changes with timestamps, editors, and reasons.

## Storage

### Directory Layout

```
.git/
  ├── objects/                          # Standard Git object store
  │   └── ab/cdef123...                 # Issues/edges stored as Git blobs
  └── refs/
      └── sterna/
          └── snapshot                   # Ref to snapshot commit (protects objects from GC)

sterna/
  └── index/                            # Local source of truth for operations
      ├── issues                         # "<id> <object-hash>"
      └── edges                          # "<source> <target> <type> <object-hash>"
```

Objects are stored as proper Git blobs via `git2`. The snapshot commit/tree structure keeps them reachable, preventing garbage collection.

### Index Files

**`sterna/index/issues`:**
```
st-a3f8e9 abcdef1234567890...
st-b4f9f0 cdef4567890abcde...
```

**`sterna/index/edges`:**
```
st-a3f8e9 st-b4f9f0 depends_on cdef4567890abcde...
```

### Object Format

Objects are compact JSON (pipe through `jq .` for pretty viewing):

**Issue:**
```json
{
  "schema_version": 1,
  "id": "st-a3f8e9",
  "title": "Fix authentication bug",
  "description": "Users can't log in when...",
  "status": "open",
  "priority": 0,
  "issue_type": "bug",
  "labels": ["security"],
  "created_at": 1704782400,
  "updated_at": 1704786000,
  "lamport": 42,
  "editor": "user@example.com",
  "claimed": false,
  "claim_context": null,
  "claimed_at": null,
  "reason": null
}
```

**Edge:**
```json
{
  "schema_version": 1,
  "source": "st-a3f8e9",
  "target": "st-b4f9f0",
  "edge_type": "depends_on",
  "created": 1704782400
}
```

### ID Generation

Issue IDs are hash-based: `st-<hash[:n]>`

- Hash = SHA-1 of (title + description + creator + timestamp + nonce)
- Length starts at 4 characters, adapts if collisions exceed 25%
- Collision probability table:

| DB Size | 4-char | 5-char | 6-char |
|---------|--------|--------|--------|
| 500     | 7%     | 0.2%   | <0.01% |
| 1,000   | 26%    | 0.8%   | 0.02%  |

Algorithm increases length when collision probability exceeds 25%.

## CRDT

### Lamport Clocks

Every issue has a Lamport timestamp:

```rust
// Local edit: increment by 1
issue.lamport += 1;

// Receiving remote: witness and increment
issue.lamport = max(local.lamport, remote.lamport) + 1;
```

### LWW Field Merge

When merging local and remote versions of the same issue:

```rust
fn merge_issue(local: Issue, remote: Issue) -> Issue {
    let mut result = local;
    if remote.lamport > local.lamport {
        result.title = remote.title;
        result.description = remote.description;
        result.status = remote.status;
        result.priority = remote.priority;
        result.labels = remote.labels;
    }
    result.lamport = max(local.lamport, remote.lamport);
    result
}
```

### Concurrent Claims

If two agents claim simultaneously, higher Lamport wins. Loser must pick different work.

## DAG Validation

Cycle detection via DFS on every edge insertion:

```rust
fn detect_cycle(edges: &[Edge], edge_type: &str) -> Result<(), String> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for e in edges {
        if should_check_cycle(&e.edge_type) {
            adj.entry(e.source.clone())
                .or_default()
                .push(e.target.clone());
        }
    }

    let mut color: HashMap<String, &str> = HashMap::new();
    for node in adj.keys() {
        if !color.contains_key(node) {
            dfs(node, &adj, &mut color)?;
        }
    }
    Ok(())
}
```

**Skipped for cycles:** `relates_to` (intentionally bidirectional)

### Helper Functions

`should_check_cycle(edge_type: &str) -> bool` - Returns true if the edge type should be checked for cycles. Currently checks for `depends_on`, `blocks`, and `parent_child` edge types.

`dfs(node: &str, adj: &HashMap<String, Vec<String>>, color: &mut HashMap<String, &str>) -> Result<(), String>` - Performs depth-first search to detect cycles in the dependency graph.

## Sync

### Local State

```
refs/sterna/snapshot → commit → tree
                               ├── snapshot.json
                               ├── issues/
                               └── edges/
```

### Pull (`st pull`)

1. Fetch remote snapshot and objects
2. Merge local and remote issues (LWW)
3. Merge local and remote edges (union, resolve conflicts)
4. Create new snapshot with merged objects
5. Update `refs/sterna/snapshot`

### Push (`st push`)

1. Pack all local objects
2. Transfer to remote via Git protocol
3. Remote imports objects and updates its ref

### Ref Protection

Fetch remote objects before push to avoid data loss.

## Agent Integration

### Identity

Agent identity comes from Git config:

```bash
git config user.email  # This is your identity
```

No registration required.

### Tool Onboarding

To use Sterna with Claude Code, add to your project's `AGENTS.md`:

```markdown
## Issue Tracking
This project uses Sterna. Run `st onboard` for context.
```

### Claude Code Hooks

Manually add hooks to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "type": "command",
        "command": "st onboard"
      }
    ],
    "PreCompact": [
      {
        "type": "command",
        "command": "st prime"
      }
    ]
  }
}
```

### Commands

| Command | Purpose | Output |
|---------|---------|---------|
| `st onboard` | Brief tool introduction | ~100-200 tokens |
| `st prime` | Full workflow + current state | ~1-2k tokens |

**`st onboard`** - Brief intro: What is Sterna, key commands, reference to `st prime`

**`st prime`** - Full reference: Complete command list, session close protocol, step-by-step workflow, current ready work

### Configuration

Users can customize command output by creating override files:

| File | Purpose |
|------|---------|
| `~/.config/sterna/onboard.md` | Override `st onboard` output |
| `~/.config/sterna/prime.md` | Override `st prime` output |

If these files exist, their content replaces default output entirely.

### Directory Layout

```
~/.config/
  └── sterna/
      ├── onboard.md    # Optional: custom onboard output
      └── prime.md      # Optional: custom prime output
```

### Workflow

```bash
# See available work
st ready

# Claim an issue
st claim st-a3f8e9

# Do work...

# Release if not completing
st release st-a3f8e9

# Close when done
st close st-a3f8e9 --reason "Completed"

# Reopen if needed
st reopen st-a3f8e9
```

### Concurrent Claims

If two agents claim the same issue, higher Lamport wins. Loser receives an error.

## Commands Reference

### Setup

| Command | Description |
|---------|-------------|
| `st init` | Initialize Sterna (create empty state) |
| `st onboard` | Output brief tool introduction for agents |
| `st prime` | Output full workflow reference + current ready work |
| `st purge` | Export, confirm, then remove all traces |

### Issue Operations

| Command | Description |
|---------|-------------|
| `st create "title" -d "description"` | Create issue |
| `st get <id>` | Show issue |
| `st list [--status open\|closed\|in_progress] [--type epic\|bug\|...]` | List issues |
| `st update <id> --title "..." --priority 2` | Update issue |

### Claim Management

| Command | Description |
|---------|-------------|
| `st claim <id>` | Claim an issue (sets status to in_progress) |
| `st release <id> [--reason "..."]` | Release claim, revert to open |
| `st close <id> [--reason "..."]` | Close issue |
| `st reopen <id> [--reason "..."]` | Re-open closed issue |

### Dependencies

| Command | Description |
|---------|-------------|
| `st depend <id> --needs <other-id>` | A needs B (A depends_on B) |
| `st depend <id> --blocks <other-id>` | A blocks B (B depends_on A) |
| `st depend <id> --relates-to <other-id>` | Add relates_to edge |
| `st depend <id> --parent <other-id>` | Add parent_child edge |
| `st depend <id> --duplicates <other-id>` | Add duplicates edge |
| `st ready` | Show unblocked, unclaimed issues |

### Sync

| Command | Description |
|---------|-------------|
| `st pull` | Fetch and merge from remote |
| `st push` | Push local changes to remote |
| `st sync` | Run `pull` then `push` |

### Data Management

| Command | Description |
|---------|-------------|
| `st export [--output <file>]` | Export all issues/edges to JSON |
| `st import <file>` | Import from exported JSON (merge) |

## Implementation Phases

### Phase 1: Foundation
- Initialize Rust project with `git2` crate
- Git config reading (user.email)
- Hash-based ID generation with collision avoidance
- Write/read objects as Git blobs
- Index file management

### Phase 2: Issue Operations
- `st create`, `st get`, `st list`
- `st update` with LWW logic
- Lamport clock management

### Phase 3: Claims
- `st claim`, `st release`
- `st close`, `st reopen`
- `st ready` query

### Phase 4: Dependencies
- Edge creation for all 5 types
- DAG cycle detection
- Edge persistence

### Phase 5: Sync
- Snapshot creation
- `refs/sterna/snapshot` ref management
- `st pull`, `st push`

### Phase 6: Data Management
- `st export`, `st import`
- `st init`, `st purge`

### Phase 7: Polish
- `st onboard`, `st prime`
- Override file support
- Human-readable ID shorteners
- Clean CLI output
- Error handling
- JSON output for agents

## File Layout

```
sterna/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── onboard.rs
│   │   ├── prime.rs
│   │   ├── create.rs
│   │   ├── get.rs
│   │   ├── list.rs
│   │   ├── update.rs
│   │   ├── claim.rs
│   │   ├── release.rs
│   │   ├── close.rs
│   │   ├── reopen.rs
│   │   ├── depend.rs
│   │   ├── ready.rs
│   │   ├── pull.rs
│   │   ├── push.rs
│   │   ├── sync.rs
│   │   ├── export.rs
│   │   ├── import.rs
│   │   ├── init.rs
│   │   └── purge.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── object.rs
│   │   ├── index.rs
│   │   └── snapshot.rs
│   ├── crdt/
│   │   └── mod.rs
│   └── dag/
│       └── mod.rs
└── sterna/
    └── index/
        ├── issues
        └── edges
```

## Tradeoffs

| Decision | Rationale |
|----------|-----------|
| Git blobs + snapshot ref | GC-safe, invisible to Git history, portable |
| Index as local source of truth | Fast local operations, snapshot for sync |
| No SQLite | Simplicity, no daemon |
| LWW CRDT | Simple conflict resolution |
| No agent registration | Identity from Git config |
| No auto-sync | User controls remote operations |
| JSON serialization | Human-readable, debuggable |
| Single ref | Simpler than per-user refs |
| Manual Claude Code setup | Non-invasive, user-controlled |

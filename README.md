# Sterna

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Persistent task memory for AI coding agents.

## Overview

AI agents lose track on long-horizon tasks. Context windows reset, conversations compact, and suddenly the agent forgets what it was working on or picks up the wrong task.

Sterna gives agents a persistent, structured place to track work:
- **Claim tasks** before starting, so agents know what they're doing
- **Track dependencies** so agents work on unblocked tasks first
- **See what's ready** with `st ready` - no guessing, no duplicated effort
- **Survive context resets** - task state lives in Git, not the conversation

## Installation

**From source:**
```bash
cargo install --path .
```

**With Nix:**
```bash
nix develop
cargo build --release
```

Binary is named `st`.

## Quick Start

```bash
# Initialize in a Git repo
st init

# Create an issue
st create "Fix authentication bug" -d "Users can't log in when..."

# List issues
st list

# See available work
st ready

# Claim and work on an issue
st claim st-a3f8

# Mark done
st close st-a3f8 --reason "Fixed in commit abc123"

# Sync with team
st pull
st push
```

## Commands

### Setup

| Command | Description |
|---------|-------------|
| `st init` | Initialize Sterna in current repo |
| `st purge` | Remove all Sterna data (with confirmation) |

### Issues

| Command | Description |
|---------|-------------|
| `st create <title> [-d desc] [--priority N] [--type T] [--label L]` | Create issue |
| `st get <id> [--json]` | Show issue details |
| `st list [--status S] [--type T] [--json]` | List issues |
| `st update <id> [--title T] [--description D] [--priority N]` | Update issue |

**Status values:** `open`, `in_progress`, `closed`

**Priority values:** `0` (critical), `1` (high), `2` (medium), `3` (low), `4` (backlog)

**Issue types:** `epic`, `task`, `bug`, `feature`, `chore`

### Claims

| Command | Description |
|---------|-------------|
| `st claim <id> [--context "branch"]` | Claim issue (sets status to `in_progress`) |
| `st release <id> [--reason "..."]` | Release claim, revert to `open` |
| `st close <id> [--reason "..."]` | Close issue |
| `st reopen <id> [--reason "..."]` | Reopen closed issue |
| `st ready [--json]` | Show unblocked, unclaimed issues |

### Dependencies

| Command | Description |
|---------|-------------|
| `st dep add <id> --needs <other>` | A depends on B (A blocked by B) |
| `st dep add <id> --blocks <other>` | A blocks B |
| `st dep add <id> --relates-to <other>` | Non-blocking relation |
| `st dep add <id> --parent <other>` | Parent-child hierarchy |
| `st dep add <id> --duplicates <other>` | Mark as duplicate |
| `st dep remove <id> --needs <other>` | Remove a dependency |

Cycle detection prevents circular dependencies.

### Sync

| Command | Description |
|---------|-------------|
| `st pull [remote]` | Fetch and merge from remote |
| `st push [remote]` | Push local changes to remote |
| `st sync [remote]` | Pull then push |

### Data

| Command | Description |
|---------|-------------|
| `st export [--output file]` | Export all issues/edges to JSON |
| `st import <file>` | Import from JSON (merges with existing) |

### Agent Commands

| Command | Description |
|---------|-------------|
| `st onboard` | Brief intro for AI agents (~100-200 tokens) |
| `st prime` | Full workflow reference (~1-2k tokens) |

## Agent Integration

Sterna is designed for use with AI coding agents like Claude Code.

**Add to your project's AGENTS.md:**
```markdown
## Issue Tracking
This project uses Sterna. Run `st onboard` for context.
```

**Optional: Configure Claude Code hooks in `~/.claude/settings.json`:**
```json
{
  "hooks": {
    "SessionStart": [{ "type": "command", "command": "st onboard" }],
    "PreCompact": [{ "type": "command", "command": "st prime" }]
  }
}
```

**Customize output:** Create `~/.config/sterna/onboard.md` or `~/.config/sterna/prime.md` to override defaults.

See [AGENTS.md](AGENTS.md) for development guidelines.

## Workflow

```bash
# Start of session
st ready                              # See available work

# Claim and work
st claim st-a3f8 --context "fix/auth" # Claim with branch context
# ... do work ...
st close st-a3f8 --reason "Completed" # Done

# If interrupted
st release st-a3f8 --reason "Switching tasks"

# Sync with team
st pull                               # Get remote changes
st push                               # Share local changes
```

## Architecture

Git-native, local-first, no daemon, no SQLite.

- **Storage:** All state in `refs/sterna/snapshot` (commit → tree with issues/ and edges/)
- **Merge:** CRDT with Lamport clocks (Last-Write-Wins)
- **Dependencies:** DAG with cycle detection

See [sterna.md](sterna.md) for full architecture documentation.

## Acknowledgments

Inspired by [Beads](https://github.com/steveyegge/beads) by Steve Yegge.

## License

MIT

use std::fs;
use std::path::PathBuf;

use crate::error::Error;

const DEFAULT_ONBOARD: &str = r#"# Sterna

Git-native issue tracker for AI coding agents. Issues persist in Git refs, surviving context resets.

## Getting Started

```bash
st init                    # Initialize in a Git repo
st create "Fix bug" -d "Description" --type bug --priority 2
st ready                   # See available work
```

## Commands

### Issues
- `st create <title> [-d desc] [--type T] [--priority N]` - Create issue
- `st get <id> [--json]` - Show issue details
- `st list [--status S] [--type T] [--json]` - List issues
- `st update <id> [--title T] [--description D] [--priority N]` - Update issue

**Types:** epic, task, bug, feature, chore
**Priority:** 0 (critical) to 4 (backlog)
**Status:** open, in_progress, closed

### Workflow
- `st ready [--json]` - Show unblocked, unclaimed issues
- `st claim <id> [--context "..."]` - Claim issue (sets in_progress)
- `st release <id> [--reason "..."]` - Release claim
- `st close <id> [--reason "..."]` - Close issue
- `st reopen <id> [--reason "..."]` - Reopen issue

### Dependencies
- `st dep add <src> --needs <tgt>` - src depends on tgt
- `st dep add <src> --blocks <tgt>` - src blocks tgt
- `st dep add <src> --relates-to <tgt>` - non-blocking relation
- `st dep add <src> --parent <tgt>` - parent-child hierarchy
- `st dep add <src> --duplicates <tgt>` - mark duplicate
- `st dep remove <src> --needs <tgt>` - remove dependency

### Data
- `st export [--output file]` - Export to JSON
- `st import <file>` - Import from JSON
- `st purge` - Remove all Sterna data

## Session Context

Run `st prime` for lean workflow context (injected automatically via hooks).
"#;

pub fn run(export: bool) -> Result<(), Error> {
    if export {
        print!("{DEFAULT_ONBOARD}");
        return Ok(());
    }

    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        print!("{content}");
    } else {
        print!("{DEFAULT_ONBOARD}");
    }

    Ok(())
}

fn get_config_path() -> Result<PathBuf, Error> {
    let home = std::env::var("HOME").map_err(|_| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME not set",
        ))
    })?;
    Ok(PathBuf::from(home).join(".config/sterna/onboard.md"))
}

use std::fs;
use std::path::PathBuf;

use crate::commands::ready;
use crate::error::Error;

const DEFAULT_PRIME: &str = r#"# Sterna Command Reference

## Core Commands
- st init              Initialize Sterna in repository
- st create <title>    Create new issue
- st list              List all issues
- st get <id>          Show issue details
- st update <id>       Update issue fields

## Workflow Commands
- st ready             Show issues ready for work (open, unclaimed, unblocked)
- st claim <id>        Claim an issue to work on
- st release <id>      Release a claimed issue
- st close <id>        Close an issue
- st reopen <id>       Reopen a closed issue

## Dependencies
- st depend <src> --needs <tgt>    Source needs target done first
- st depend <src> --blocks <tgt>   Source blocks target
- st depend <src> --relates-to <tgt>
- st depend <src> --parent <tgt>
- st depend <src> --duplicates <tgt>

## Data Management
- st export            Export all data to JSON
- st import <file>     Import data from JSON
- st purge             Remove all Sterna data

## Sync
- st push [remote]     Push snapshot to remote
- st pull [remote]     Pull and merge from remote

## Agent Commands
- st onboard           Show onboarding info
- st prime             Show this reference

---
"#;

pub fn run() -> Result<(), Error> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        print!("{}", content);
    } else {
        print!("{}", DEFAULT_PRIME);
    }

    // Always show ready issues
    eprintln!("## Ready Issues");
    ready::run(false)?;

    Ok(())
}

fn get_config_path() -> Result<PathBuf, Error> {
    let home = std::env::var("HOME").map_err(|_| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME not set",
        ))
    })?;
    Ok(PathBuf::from(home).join(".config/sterna/prime.md"))
}

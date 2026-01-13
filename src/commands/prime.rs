use std::fs;
use std::path::PathBuf;

use crate::commands::ready;
use crate::error::Error;

const DEFAULT_PRIME: &str = r#"# Sterna Workflow

## Core Loop
1. `st ready` - Find available work
2. `st claim <id>` - Take ownership
3. Do the work
4. `st close <id>` - Mark complete

## Session Checklist
Before saying "done":
- [ ] All work committed
- [ ] Claimed issues closed or released
- [ ] Discovered work captured via `st create`

## Essential Commands
- `st ready` - Unblocked issues ready for work
- `st claim <id>` - Claim an issue
- `st close <id>` - Close an issue
- `st create "title"` - Create new issue
- `st get <id>` - Show issue details

## Dependencies
- `st dep add <src> --needs <tgt>` - src depends on tgt
- `st dep add <src> --blocks <tgt>` - src blocks tgt
- `st dep remove <src> --needs <tgt>` - Remove dependency

## Rules
- Check `st ready` at session start
- Claim before starting work
- Close or release before ending session

---
"#;

pub fn run(export: bool) -> Result<(), Error> {
    if export {
        print!("{DEFAULT_PRIME}");
        return Ok(());
    }

    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        print!("{content}");
    } else {
        print!("{DEFAULT_PRIME}");
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

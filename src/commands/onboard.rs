use std::fs;
use std::path::PathBuf;

use crate::error::Error;

const DEFAULT_ONBOARD: &str = r#"Sterna: Git-native issue tracker.

## Workflow
1. `st ready` - find available work
2. `st claim <id>` - take ownership
3. Work on the issue
4. `st close <id>` - mark complete

## Session Protocol
- Run `st ready` at session start
- Claim before starting work
- Close issues when done (include "Closes: <id>" in commits)
- Run `st prime` for full reference
"#;

pub fn run(export: bool) -> Result<(), Error> {
    if export {
        print!("{}", DEFAULT_ONBOARD);
        return Ok(());
    }

    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        print!("{}", content);
    } else {
        print!("{}", DEFAULT_ONBOARD);
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

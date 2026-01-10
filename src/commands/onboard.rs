use std::fs;
use std::path::PathBuf;

use crate::error::Error;

const DEFAULT_ONBOARD: &str = r#"Sterna: Git-native issue tracker. Key commands:
- st ready     : show available work
- st claim <id>: take an issue
- st close <id>: finish an issue
- st prime     : full reference
"#;

pub fn run() -> Result<(), Error> {
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

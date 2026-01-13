use crate::commands::{pull, push};
use crate::error::Error;

pub fn run(remote: Option<String>) -> Result<(), Error> {
    pull::run(remote.clone())?;
    push::run(remote)?;
    Ok(())
}

use crate::interface::Interface;
use anyhow::{Context, Result};

mod errors;
mod interface;
mod ioutil;

const DEFAULT_SAVED_DIR: &str = "/etc/wifisaved";

fn main() -> Result<()> {
    validate_saved_dir(DEFAULT_SAVED_DIR)?;
    let mut interface = Interface::build()?;
    if interface.try_saved_connection()?.is_none() {
        interface.try_new_connection()?;
    }

    Ok(())
}

pub fn validate_saved_dir(path: &str) -> Result<()> {
    if std::fs::read_dir(path).is_err() {
        std::fs::create_dir(path)
            .context("could not access wifi saved directory. Did you execute as root?")?;
    }
    ioutil::chmod(path, "400")?; // This ugly function wouldn't exist if PermissionsExt worked in OpenBSD

    Ok(())
}

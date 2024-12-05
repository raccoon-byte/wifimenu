use crate::interface::Interface;
use anyhow::Result;

mod errors;
mod interface;
mod ioutil;

const DEFAULT_SAVED_DIR: &str = "/etc/wifisaved";

fn main() -> Result<()> {
    ioutil::validate_saved_dir(DEFAULT_SAVED_DIR)?;
    let mut interface = Interface::build()?;
    if interface.try_saved_connection()?.is_none() {
        interface.try_new_connection()?;
    }

    Ok(())
}


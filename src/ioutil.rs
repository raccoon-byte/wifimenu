use crate::ioutil;
use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::process;

mod column;

pub fn display_menu(connections: &[String], title: &str) -> Option<usize> {
    if connections.is_empty() {
        return None;
    }
    println!("{title}");

    column::col_print(connections);

    let mut buffer = String::new();
    if io::stdin().read_line(&mut buffer).is_ok() {
        if let Ok(selection) = buffer.trim().parse::<usize>() {
            return Some(selection);
        }
    }
    None
}

pub fn input(prompt: &str) -> String {
    loop {
        if let Ok(pass) = rpassword::prompt_password(prompt) {
            return pass;
        }
        println!();
    }
}

pub fn chmod(path: &str, mode: &str) -> Result<()> {
    process::Command::new("chmod")
        .args([mode.to_string(), path.to_string()])
        .output()?;

    Ok(())
}

pub fn validate_saved_dir(path: &str) -> Result<()> {
    if fs::read_dir(path).is_err() {
        fs::create_dir_all(path)
            .context("could not access wifi saved directory. Did you execute as root?")?;
    }
    ioutil::chmod(path, "400")?; // This ugly function wouldn't exist if PermissionsExt worked in OpenBSD

    Ok(())
}

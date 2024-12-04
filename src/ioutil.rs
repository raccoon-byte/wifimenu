mod column;

use anyhow::Result;
use std::io;
use std::process;

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

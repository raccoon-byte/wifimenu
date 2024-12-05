use anyhow::{Context, Result};
use regex::Regex;
use std::env;
use std::fs;
use std::io::Write;
use std::process;

use crate::errors::Error;
use crate::ioutil::{display_menu, input};

const DEFAULT_SAVED_DIR: &str = "/etc/wifisaved";

#[derive(PartialEq)]
enum WirelessMode {
    Auto,
    M11a,
    M11b,
    M11g,
    M11n,
    M11ac,
}
impl std::fmt::Display for WirelessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, ""),
            Self::M11a => write!(f, "11a"),
            Self::M11b => write!(f, "11b"),
            Self::M11g => write!(f, "11g"),
            Self::M11n => write!(f, "11n"),
            Self::M11ac => write!(f, "11ac"),
        }
    }
}

pub struct Interface {
    name: String,
    mode: WirelessMode,
    ssid: String,
    password: String,
}

impl Interface {
    fn new(name: &str, mode: WirelessMode) -> Self {
        Self {
            name: name.to_string(),
            mode,
            ssid: String::new(),
            password: String::new(),
        }
    }

    pub fn build() -> Result<Self> {
        let args: Vec<String> = env::args().collect();
        match args.len() {
            1 => Err(Error::WrongArgumentsCount).context("Usage: doas wifimenu [interface] [mode]"),
            2 => Ok(Interface::new(&args[1], WirelessMode::Auto)),
            _ => {
                let mode: WirelessMode =
                    match args[2].trim() {
                        "11a" => WirelessMode::M11a,
                        "11b" => WirelessMode::M11b,
                        "11g" => WirelessMode::M11g,
                        "11n" => WirelessMode::M11n,
                        "11ac" => WirelessMode::M11ac,
                        _ => return Err(Error::InvalidWirelessMode).context(
                            "Currently supported modes are '11a', '11b', '11g', '11n' and '11ac'",
                        ),
                    };
                Ok(Interface::new(&args[1], mode))
            }
        }
    }

    pub fn saved_connect(&mut self) -> Result<()> {
        let dest = format!("/etc/hostname.{}", self.name);
        let from = format!("{}/{}.{}", DEFAULT_SAVED_DIR, self.ssid, self.name);
        self.password = std::fs::read_to_string(&from)?
            .split('\n')
            .next()
            .context("couldn't read saved connection data. Did you execute as root?")?
            .trim()
            .strip_prefix("#")
            .unwrap()
            .to_owned();
        std::fs::copy(from, dest)?;
        self.connect()?;

        Ok(())
    }

    pub fn connect(&self) -> Result<()> {
        process::Command::new("ifconfig")
            .args([
                self.name.clone(),
                "nwid".to_string(),
                self.ssid.clone(),
                "wpakey".to_string(),
                self.password.clone(),
            ])
            .output()?;

        if self.mode != WirelessMode::Auto {
            process::Command::new("ifconfig")
                .args([
                    self.name.clone(),
                    "mode".to_string(),
                    format!("{}", self.mode),
                ])
                .output()?;
        }

        Ok(())
    }

    pub fn select_network(&mut self) -> Result<()> {
        let connections = self.scan()?;
        let selected_option: usize;
        loop {
            if let Some(sel) = display_menu(&connections, "Choose your desired option: ") {
                selected_option = sel;
                break;
            }
        }

        self.ssid = connections[selected_option - 1].to_string();
        if let Some(Some(ssid)) = self.ssid.strip_prefix("\"").map(|s| s.strip_suffix("\"")) {
            self.ssid = ssid.to_string();
        }
        self.password = input("Type the password: ");

        self.create_hostname_files()?;

        Ok(())
    }

    fn create_hostname_files(&self) -> Result<()> {
        let connection_dest = format!("/etc/hostname.{}", self.name);
        let mut connection_output = std::fs::File::create(connection_dest)?;
        write!(connection_output, "{}", self.render_hostname())?;
        connection_output
            .metadata()?
            .permissions()
            .set_readonly(true);

        let saved_path = format!("{}/{}.{}", DEFAULT_SAVED_DIR, self.ssid, self.name);
        let mut saved_file = fs::File::create(&saved_path)?;
        write!(saved_file, "{}", self.render_hostname())?;
        saved_file.metadata()?.permissions().set_readonly(true);

        Ok(())
    }

    fn render_hostname(&self) -> String {
        let mut hostname = format!(
            "\
            #{}
            join \"{}\" wpakey \"{}\"
            inet6 autoconf
            inet autoconf\n",
            self.password, self.ssid, self.password
        );
        if self.mode != WirelessMode::Auto {
            hostname += &format!("mode {}", self.mode);
        }
        hostname
    }

    pub fn scan(&self) -> Result<Vec<String>> {
        let mut connections: Vec<String> = Vec::new();
        let re = Regex::new(r"nwid.(.*).chan.*").unwrap();
        let command = process::Command::new("ifconfig")
            .args([self.name.clone(), "scan".to_string()])
            .output()?;

        if !command.stderr.is_empty() {
            return Err(Error::FailedScan).context(
                String::from_utf8(command.stderr)
                    .unwrap()
                    .trim()
                    .to_string(),
            );
        }

        let raw_connections = String::from_utf8(command.stdout)?;

        for (_, [ssid]) in re.captures_iter(&raw_connections).map(|c| c.extract()) {
            connections.push(ssid.to_string());
        }
        Ok(Interface::get_sanitized_ssid_list(connections))
    }

    fn get_sanitized_ssid_list(connections: Vec<String>) -> Vec<String> {
        let mut connections = connections
            .into_iter()
            .filter(|item| !item.starts_with("0x0") && *item != "\"\"")
            .collect::<Vec<String>>();
        connections.sort();
        connections.dedup();

        connections
    }

    pub fn read_saved_connections(&self, path: &str) -> Result<Vec<String>, std::io::Error> {
        let mut file_paths = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if !entry.path().is_file() {
                continue;
            }

            if let Some(file_name) = entry.path().file_name() {
                if let Some(file_name) = file_name.to_str() {
                    if file_name.ends_with(self.name.as_str()) {
                        file_paths.push(
                            entry
                                .path()
                                .file_stem()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string(),
                        );
                    }
                }
            }
        }
        Ok(file_paths)
    }

    pub fn try_saved_connection(&mut self) -> Result<Option<()>> {
        let saved_connections = self
            .read_saved_connections(DEFAULT_SAVED_DIR)
            .with_context(|| {
                format!(
                    "Failed to read saved connections from {}. Did you execute as root?",
                    DEFAULT_SAVED_DIR
                )
            })?;

        let selection = display_menu(
            &saved_connections,
            "Choose your desired option or just press \"Enter\" to scan new networks: ",
        );

        if saved_connections.is_empty() || selection.is_none() {
            return Ok(None);
        }

        self.ssid = saved_connections[selection.unwrap() - 1].to_string();
        self.saved_connect()?;

        Ok(Some(()))
    }

    pub fn try_new_connection(&mut self) -> Result<()> {
        self.select_network()?;
        self.connect()?;
        Ok(())
    }
}

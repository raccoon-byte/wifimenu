use regex::Regex;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process;

mod column;

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
    pub name: String,
    mode: WirelessMode,
    pub ssid: String,
    pub password: String,
}
const DEFAULT_SAVED_DIR: &str = "/etc/wifisaved";

impl Interface {
    fn new(name: &str, mode: WirelessMode) -> Self {
        Self {
            name: name.to_string(),
            mode,
            ssid: String::new(),
            password: String::new(),
        }
    }

    pub fn build() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        match args.len() {
            1 => return Err(String::from("Usage: doas wifimenu [interface] [mode]")),
            2 => return Ok(Interface::new(args.get(1).unwrap(), WirelessMode::Auto)),
            _ => {
                let mode: WirelessMode = match args.get(2).unwrap().trim() {
                    "11a" => WirelessMode::M11a,
                    "11b" => WirelessMode::M11b,
                    "11g" => WirelessMode::M11g,
                    "11n" => WirelessMode::M11n,
                    "11ac" => WirelessMode::M11ac,
                    _ => return Err(String::from("invalid wireless mode selected")),
                };
                return Ok(Interface::new(args.get(1).unwrap(), mode));
            }
        }
    }

    pub fn saved_connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let dest = format!("/etc/hostname.{}", self.name);
        let from = format!("{}/{}.{}", DEFAULT_SAVED_DIR, self.ssid, self.name);
        self.password = std::fs::read_to_string(&from)?
            .split("\n")
            .next()
            .ok_or("couldn't read saved connection data. Did you execute as root?")?
            .trim()
            .strip_prefix("#")
            .unwrap()
            .to_owned();
        std::fs::copy(from, dest)?;
        self.connect()?;

        Ok(())
    }

    pub fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn select_network(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let connections = self.scan()?;
        let selected_option: usize;
        loop {
            if let Some(sel) = display_menu(&connections, "Choose your desired option: ") {
                selected_option = sel;
                break;
            }
        }

        self.ssid = connections.get(selected_option - 1).unwrap().to_string();
        if let Some(Some(ssid)) = self.ssid.strip_prefix("\"").map(|s| s.strip_suffix("\"")) {
            self.ssid = ssid.to_string();
        }
        self.password = input("Type the password");

        self.create_hostname_files()?;

        Ok(())
    }

    fn create_hostname_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let connection_dest = format!("/etc/hostname.{}", self.name);
        let mut connection_output = std::fs::File::create(connection_dest)?;
        write!(connection_output, "{}", self.render_hostname())?;
        connection_output
            .metadata()?
            .permissions()
            .set_readonly(true);

        let saved_dest = format!("{}/{}.{}", DEFAULT_SAVED_DIR, self.ssid, self.name);
        let mut saved_output = std::fs::File::create(saved_dest)?;
        write!(saved_output, "{}", self.render_hostname())?;
        saved_output.metadata()?.permissions().set_readonly(true);

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

    pub fn scan(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut connections: Vec<String> = Vec::new();
        let re = Regex::new(r"nwid.(.*).chan.*").unwrap();
        let command = process::Command::new("ifconfig")
            .args([self.name.clone(), "scan".to_string()])
            .output()?;
        if command.stderr.len() != 0 {
            return Err(String::from_utf8(command.stderr).unwrap().trim().into());
        }
        let raw_connections = String::from_utf8(command.stdout)?;

        for (_, [ssid]) in re.captures_iter(&raw_connections).map(|c| c.extract()) {
            connections.push(ssid.to_string());
        }
        Ok(Interface::sanitize_ssid_list(connections))
    }

    fn sanitize_ssid_list(connections: Vec<String>) -> Vec<String> {
        let mut connections = connections
            .into_iter()
            .filter(|item| !item.starts_with("0x0") && *item != "\"\"")
            .collect::<Vec<String>>();
        connections.sort();
        connections.dedup();

        connections
    }
}

pub fn read_saved_connections(path: &str, int: &str) -> Result<Vec<String>, std::io::Error> {
    let mut file_paths = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if !entry.path().is_file() {
            continue;
        }
        if let Some(file_name) = entry.path().file_name() {
            file_name.to_str().and_then(|fname| {
                if fname.ends_with(int) {
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
                Some(())
            });
        }
    }
    Ok(file_paths)
}

pub fn display_menu(connections: &Vec<String>, prompt: &str) -> Option<usize> {
    if connections.is_empty() {
        return None;
    }
    println!("{prompt}");

    column::col_print(connections);

    let mut buffer = String::new();
    if let Ok(_) = io::stdin().read_line(&mut buffer) {
        if let Ok(selection) = buffer.trim().parse::<usize>() {
            return Some(selection);
        }
    }
    None
}

pub fn input(prompt: &str) -> String {
    let mut buffer = String::new();
    loop {
        println!("{prompt}: ");
        if let Ok(_) = io::stdin().read_line(&mut buffer) {
            return buffer.trim().to_string();
        }
        println!()
    }
}

pub fn validate_saved_dir(path: &str) -> Result<(), String> {
    match std::fs::read_dir(path) {
        io::Result::Err(_) => {
            if std::fs::create_dir(path).is_err() {
                return Err(String::from(
                    "could not access wifi saved directory. Did you execute as root?",
                ));
            } else {
                return Ok(());
            }
        }
        _ => Ok(()),
    }
}

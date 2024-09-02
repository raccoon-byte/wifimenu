use wifimenu::*;

const DEFAULT_SAVED_DIR: &str = "/etc/wifisaved";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interface = Interface::build()?;

    validate_saved_dir(DEFAULT_SAVED_DIR)?;
    let saved_connections =
        read_saved_connections(DEFAULT_SAVED_DIR, &interface.name).map_err(|e| {
            format!(
                "Failed to read saved connections from {}. Did you execute as root?: {}",
                DEFAULT_SAVED_DIR, e
            )
        })?;
    let selection = display_menu(
        &saved_connections,
        "Choose your desired option or just press \"Enter\" to scan new networks: ",
    );

    if !saved_connections.is_empty() && selection != None {
        interface.ssid = saved_connections
            .get(selection.unwrap() - 1)
            .unwrap()
            .to_string();
        interface.saved_connect()?;
        return Ok(());
    }

    interface.select_network()?;
    interface.connect()?;

    Ok(())
}

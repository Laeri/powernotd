use clap::{CommandFactory, Parser};
/// Battery-level notification daemon for linux that sends events according to the 'Desktop Notification Specification' to
/// the user. Notifications are emitted when specific battery-level thresholds are reached or when the
/// battery is fully charged.
// see https://specifications.freedesktop.org/notification-spec/notification-spec-latest.html
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Print the current battery-level to stdout then exit
    #[arg(short = 's', long, default_value_t = false)]
    pub status_level: bool,

    /// Print charging status 'charging', 'discharging', 'full' or 'unknown' to stdout then exit
    #[arg(short = 'c', long, default_value_t = false)]
    pub charging_state: bool,

    /// Set config-file path if needed, otherwise $XDG_CONFIG_HOME/powernotd/config.json is used
    #[arg(short = 'f', long)]
    pub config_file: Option<String>,

    /// Send desktop notification with current battery-level then exit
    #[arg(short = 'n', long, default_value_t = false)]
    pub notify_now: bool,

    /// List all notification thresholds in the format 'a_1%, a_2%, ..., a_n%' that are specified in the config-file
    #[arg(short = 't', long, default_value_t = false)]
    pub list_thresholds: bool,

    /// Display the path to the config-file
    #[arg(short = 'p', long, default_value_t = false)]
    pub show_config_path: bool,

    /// Pass the battery such as 'BAT1' if your system has multiple and you do not want to use the
    /// default (BAT0). Check '/sys/class/power_supply/' to see which batteries you have.
    #[arg(short = 'b', long)]
    pub battery: Option<String>,
}

/// used within build.rs
#[allow(dead_code)]
pub fn build_command() -> clap::Command {
    Args::command()
}

#[test]
fn verify_app() {
    build_command().debug_assert();
}

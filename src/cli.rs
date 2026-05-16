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

    /// Print the default configuration as JSON then exit
    #[arg(long, default_value_t = false)]
    pub emit_default_config: bool,

    /// Pass the battery such as 'BAT1' if your system has multiple and you do not want to use the
    /// default (BAT0). Check '/sys/class/power_supply/' to see which batteries you have.
    #[arg(short = 'b', long)]
    pub battery: Option<String>,

    /// Suppress the stderr warning printed when no desktop notification daemon
    /// (dunst, mako, notification-daemon, xfce4-notifyd, ...) is reachable on D-Bus.
    /// Notifications still attempt to fire; this only silences the diagnostic.
    #[arg(long, default_value_t = false)]
    pub mute_notification_warning: bool,
}

/// used within build.rs
#[allow(dead_code)]
pub fn build_command() -> clap::Command {
    Args::command()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn verify_app() {
        build_command().debug_assert();
    }

    #[test]
    fn cli_parse_no_args_defaults() {
        let a = Args::try_parse_from(["powernotd"]).expect("parse");
        assert!(!a.status_level);
        assert!(!a.charging_state);
        assert!(!a.notify_now);
        assert!(!a.list_thresholds);
        assert!(!a.show_config_path);
        assert!(!a.emit_default_config);
        assert!(a.config_file.is_none());
        assert!(a.battery.is_none());
    }

    #[test]
    fn cli_parse_status_level_short() {
        let a = Args::try_parse_from(["powernotd", "-s"]).expect("parse");
        assert!(a.status_level);
    }

    #[test]
    fn cli_parse_status_level_long() {
        let a = Args::try_parse_from(["powernotd", "--status-level"]).expect("parse");
        assert!(a.status_level);
    }

    #[test]
    fn cli_parse_charging_state_short() {
        let a = Args::try_parse_from(["powernotd", "-c"]).expect("parse");
        assert!(a.charging_state);
    }

    #[test]
    fn cli_parse_notify_now_short() {
        let a = Args::try_parse_from(["powernotd", "-n"]).expect("parse");
        assert!(a.notify_now);
    }

    #[test]
    fn cli_parse_list_thresholds_short() {
        let a = Args::try_parse_from(["powernotd", "-t"]).expect("parse");
        assert!(a.list_thresholds);
    }

    #[test]
    fn cli_parse_show_config_path_short() {
        let a = Args::try_parse_from(["powernotd", "-p"]).expect("parse");
        assert!(a.show_config_path);
    }

    #[test]
    fn cli_parse_emit_default_config_long() {
        let a = Args::try_parse_from(["powernotd", "--emit-default-config"]).expect("parse");
        assert!(a.emit_default_config);
    }

    #[test]
    fn cli_parse_config_file_short() {
        let a = Args::try_parse_from(["powernotd", "-f", "/tmp/x.json"]).expect("parse");
        assert_eq!(a.config_file.as_deref(), Some("/tmp/x.json"));
    }

    #[test]
    fn cli_parse_battery_short() {
        let a = Args::try_parse_from(["powernotd", "-b", "BAT1"]).expect("parse");
        assert_eq!(a.battery.as_deref(), Some("BAT1"));
    }

    #[test]
    fn cli_parse_combined_t_and_f() {
        let a = Args::try_parse_from(["powernotd", "-t", "-f", "/x.json"]).expect("parse");
        assert!(a.list_thresholds);
        assert_eq!(a.config_file.as_deref(), Some("/x.json"));
    }

    #[test]
    fn cli_parse_combined_b_and_s() {
        let a = Args::try_parse_from(["powernotd", "-b", "BAT1", "-s"]).expect("parse");
        assert!(a.status_level);
        assert_eq!(a.battery.as_deref(), Some("BAT1"));
    }

    #[test]
    fn cli_parse_unknown_flag_errors() {
        let res = Args::try_parse_from(["powernotd", "--bogus"]);
        assert!(res.is_err());
    }

    #[test]
    fn cli_parse_help_flag_returns_display_help_kind() {
        let err = Args::try_parse_from(["powernotd", "--help"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
    }

    #[test]
    fn cli_parse_version_flag_returns_display_version_kind() {
        let err = Args::try_parse_from(["powernotd", "--version"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
    }
}

use clap::Parser;
/// Power notification daemon for linux that sends events according to according to the 'Desktop Notification Specification' to
/// the user. Notifications are emitted when specific battery level thresholds are reached or when the
/// battery is fully charged.
// see https://specifications.freedesktop.org/notification-spec/notification-spec-latest.html
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Print the current battery level to stdout then exit
    #[arg(short = 's', long, default_value_t = false)]
    pub status_level: bool,

    /// Print charging status 'charging', 'discharging', 'full' or 'unknown' to stdout then exit
    #[arg(short = 'c', long, default_value_t = false)]
    pub status_state: bool,

    /// Send desktop notification with current battery level then exit
    #[arg(short = 'n', long, default_value_t = false)]
    pub notify_now: bool,
}

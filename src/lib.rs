pub mod config;
pub mod notification;
pub mod upower;

use notification::{BatteryFullNotification, ChargingHook, Urgency};
use std::fs::File;
use std::io::prelude::*;
use std::{collections::HashMap, process::Command};

pub type Battery = str;

pub const DEFAULT_BATTERY: &Battery = "BAT0";

pub fn get_charging_status_path(battery: Option<&Battery>) -> String {
    let battery = battery.unwrap_or(DEFAULT_BATTERY);
    format!("/sys/class/power_supply/{}/status", battery)
}

pub fn get_power_status_path(battery: Option<&Battery>) -> String {
    let battery = battery.unwrap_or(DEFAULT_BATTERY);
    format!("/sys/class/power_supply/{}/capacity", battery)
}

/// Pure parser: maps the contents of `/sys/class/power_supply/*/capacity`
/// to a battery percentage. Trims whitespace and panics on malformed input
/// to preserve the original sysfs-read behaviour.
pub fn parse_power_level(s: &str) -> u32 {
    s.trim().parse().expect("failed to parse number")
}

/// Return the current battery level
pub fn get_current_power(battery: Option<&Battery>) -> u32 {
    let power_status_path = get_power_status_path(battery);
    let mut file = File::open(power_status_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    parse_power_level(&contents)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChargingStatus {
    Charging,
    Discharging,
    Full,
    Unknown,
}

impl ChargingStatus {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ChargingStatus::Charging => "charging",
            ChargingStatus::Discharging => "discharging",
            ChargingStatus::Full => "full",
            ChargingStatus::Unknown => "unknown",
        }
    }
    pub fn as_string(&self) -> String {
        self.as_str().to_owned()
    }

    // Plug/unplug semantics: Charging and Full both mean "AC connected".
    pub fn is_plugged_in(&self) -> bool {
        matches!(self, ChargingStatus::Charging | ChargingStatus::Full)
    }
}

/// Pure parser: maps the contents of `/sys/class/power_supply/*/status`
/// to a [`ChargingStatus`]. Trims whitespace. "Not charging" is treated
/// as Full because kernels report it once the battery hits a charge-stop
/// threshold.
pub fn parse_charging_status(s: &str) -> ChargingStatus {
    match s.trim() {
        "Charging" => ChargingStatus::Charging,
        "Discharging" => ChargingStatus::Discharging,
        "Full" | "Not charging" => ChargingStatus::Full,
        _ => ChargingStatus::Unknown,
    }
}

pub fn get_charging_status(battery: Option<&Battery>) -> ChargingStatus {
    let status_charging_path = get_charging_status_path(battery);
    let mut file = File::open(status_charging_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    parse_charging_status(&contents)
}

pub fn get_charging_status_text(battery: Option<&Battery>) -> String {
    get_charging_status(battery).as_string()
}

/// send a message using linux notify-send api
pub fn send_message(title: &str, message: &str, urgency: &Urgency, time_secs: Option<u32>) {
    let mut notification = notify_rust::Notification::new();

    notification
        .summary(title)
        .body(message)
        .urgency(notify_rust::Urgency::from(urgency));

    if let Some(wait_time) = time_secs {
        notification.timeout(notify_rust::Timeout::Milliseconds(wait_time * 1000));
        //milliseconds
    }
    notification.show().unwrap();
}

pub fn run_command(command: &str) {
    let args_res = shell_words::split(command);
    if args_res.is_err() {
        eprintln!(
            "Could not run command: {}, err: {:?}",
            command.to_owned(),
            args_res
        );
        return;
    }
    let actual_args = args_res.unwrap();
    match actual_args.as_slice() {
        [first, rest @ ..] => {
            let output = Command::new(first)
                .args(rest)
                .output()
                .unwrap_or_else(|_| panic!("Failed to run command {}", command));
            if !output.status.success() {
                eprintln!("status: {}", output.status);
                eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            }
        }
        [] => {
            eprintln!("Missing command for running");
        }
    }
}

/// Send a notification using the rust_notify library. The title and message are used from the
/// Notification if given and templated by replacing '{}' with the current percentage. In addition,
/// a system command is run if specified in the Notification.
pub fn send_notification(level: &u32, notification: &notification::Notification) {
    let title = notification
        .title
        .clone()
        .unwrap_or("Battery Status".to_string());
    let message = notification.message.clone().unwrap_or("{}".to_string());
    let percent = format!("{}", level);

    send_message(
        &title.replace("{}", &percent),
        &message.replace("{}", &percent),
        &notification.urgency,
        notification.time_secs,
    );
    if notification.command.is_some() {
        run_command(notification.command.as_ref().unwrap());
    }
}

// hook to fire notification / run command on plugin/plugout
pub fn fire_plugin_plugout_hook(level: u32, hook: &ChargingHook) {
    if !hook.enabled {
        return;
    }
    if hook.title.is_some() || hook.message.is_some() {
        let title = hook
            .title
            .clone()
            .unwrap_or_else(|| "Battery Status".to_string());
        let message = hook.message.clone().unwrap_or_else(|| "{}%".to_string());
        let percent = format!("{}", level);
        send_message(
            &title.replace("{}", &percent),
            &message.replace("{}", &percent),
            &hook.urgency,
            hook.time_secs,
        );
    }
    if let Some(cmd) = &hook.command {
        run_command(cmd);
    }
}

pub fn notify_now(level: &u32) {
    let percent = format!("{}%", level);
    let default_wait_time = 10; // seconds
    send_message(
        "Battery Status",
        &percent,
        &Urgency::Normal,
        Some(default_wait_time),
    );
}

/// Find lowest threshold which has been passed with the current battery level
pub fn find_lowest_threshold(
    current: u32,
    notified: &HashMap<u32, notification::Notification>,
) -> Option<u32> {
    let keys = notified.keys().cloned().collect::<Vec<_>>();

    keys.into_iter().filter(|&key| key >= current).min()
}

/// Reset all notifications which are not the current threshold_val
pub fn reset_other_notifications(
    threshold_val: &u32,
    notified: &mut HashMap<u32, notification::Notification>,
) {
    for (key, notification) in notified.iter_mut() {
        if *key != *threshold_val {
            notification.notified = false;
        }
    }
}

/// notify if battery is fully charged
pub fn check_notify_full_battery(
    current: &u32,
    last: &u32,
    full_notification: &mut BatteryFullNotification,
) {
    // if already notified then do nothing
    if full_notification.notified || !full_notification.enabled {
        return;
    }

    // if charge is decreasing do not notify again
    if *last >= *current {
        // if battery status is decreasing then we want to notify again if reaching full capacity
        full_notification.notified = false;
        return;
    }

    let title = full_notification
        .title
        .clone()
        .unwrap_or("Battery Status".to_string());
    let message = full_notification
        .message
        .clone()
        .unwrap_or("Fully Charged 100%".to_string());
    if *current >= 100 {
        send_message(&title, &message, &full_notification.urgency, None);
        if full_notification.command.is_some() {
            run_command(full_notification.command.as_ref().unwrap());
        }
        full_notification.notified = true;
    }
}

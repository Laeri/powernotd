pub mod config;
pub mod notification;

use notification::{BatteryFullNotification, Urgency};
use std::fs::File;
use std::io::prelude::*;
use std::{collections::HashMap, process::Command};

/// Return the current battery level
pub fn get_current_power() -> u32 {
    let mut file = File::open("/sys/class/power_supply/BAT0/capacity").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents.trim().parse().expect("failed to parse number")
}

#[derive(Debug)]
pub enum ChargingStatus {
    Charging,
    Discharging,
    Full,
    Unknown,
}

impl ChargingStatus {
    fn as_str(&self) -> &'static str {
        match *self {
            ChargingStatus::Charging => "charging",
            ChargingStatus::Discharging => "discharging",
            ChargingStatus::Full => "full",
            ChargingStatus::Unknown => "unknown",
        }
    }
    fn as_string(&self) -> String {
        self.as_str().to_owned()
    }
}

pub fn get_status_charging() -> String {
    let mut file = File::open("/sys/class/power_supply/BAT0/status").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    match contents.trim() {
        "Charging" => ChargingStatus::Charging.as_string(),
        "Discharging" => ChargingStatus::Discharging.as_string(),
        "Full" | "Not charging" => ChargingStatus::Full.as_string(),
        _ => ChargingStatus::Unknown.as_string(),
    }
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
    let percent = format!("{}%", level);

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
    for (key, mut notification) in notified.iter_mut() {
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

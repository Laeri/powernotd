pub use notify_rust::{self, Urgency};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Notification {
    // threshold level for which a notification should be sent
    pub level: u32,
    // urgency of the message, notification daemon can display them with different styling based on
    // the urgency
    pub urgency: Urgency,
    // notified is true if for the given threshold a notification has been sent out already
    pub notified: bool,

    // how long the notification is displayed
    pub time_secs: Option<u32>,
}

/// Return the current battery level
pub fn get_current_power() -> u32 {
    let mut file = File::open("/sys/class/power_supply/BAT0/capacity").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let current_level: u32 = contents.trim().parse().expect("failed to parse number");
    return current_level;
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
        "Full" => ChargingStatus::Full.as_string(),
        _ => ChargingStatus::Unknown.as_string(),
    }
}

/// send a message using linux notify-send api
pub fn send_message(title: &str, message: &str, urgency: &Urgency, time_secs: Option<u32>) {
    let mut notification = notify_rust::Notification::new();

    notification.summary(title).body(message).urgency(*urgency);

    if let Some(wait_time) = time_secs {
        notification.timeout(notify_rust::Timeout::Milliseconds(wait_time * 1000));
        //milliseconds
    }
    notification.show().unwrap();
}

pub fn send_notification(level: &u32, notification: &Notification) {
    send_message(
        "Battery Status",
        format!("{current_level}%", current_level = *level).as_str(),
        &notification.urgency,
        notification.time_secs,
    );
}

/// Find lowest threshold which has been passed with the current battery level
pub fn find_lowest_threshold(current: u32, notified: &HashMap<u32, Notification>) -> Option<u32> {
    let keys = notified.keys().cloned().collect::<Vec<_>>();

    let result = keys.into_iter().filter(|&key| key >= current).min();

    return result;
}

/// Reset all notifications which are not the current threshold_val
pub fn reset_other_notifications(threshold_val: &u32, notified: &mut HashMap<u32, Notification>) {
    for (key, mut notification) in notified.iter_mut() {
        if *key != *threshold_val {
            notification.notified = false;
        }
    }
}

/// notify if battery is fully charged
pub fn check_notify_full_battery(current: &u32, last: &u32, is_full_notified: &mut bool) {
    // if already notified then do nothing
    if *is_full_notified {
        return;
    }

    // if charge is decreasing do not notify again
    if *last >= *current {
        // if battery status is decreasing then we want to notify again if reaching full capacity
        *is_full_notified = false;
        return;
    }
    if *current == 100 {
        send_message(
            "Battery Status",
            "Fully Charged 100%",
            &Urgency::Normal,
            None,
        );
        *is_full_notified = true;
    }
}

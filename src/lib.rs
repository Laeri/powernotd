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
    if let Some(cmd) = &notification.command {
        run_command(cmd);
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
        if let Some(cmd) = &full_notification.command {
            run_command(cmd);
        }
        full_notification.notified = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::{BatteryFullNotification, Notification, Urgency};

    // ---- ChargingStatus enum --------------------------------------------

    #[test]
    fn charging_status_as_str_charging() {
        assert_eq!(ChargingStatus::Charging.as_str(), "charging");
    }

    #[test]
    fn charging_status_as_str_discharging() {
        assert_eq!(ChargingStatus::Discharging.as_str(), "discharging");
    }

    #[test]
    fn charging_status_as_str_full() {
        assert_eq!(ChargingStatus::Full.as_str(), "full");
    }

    #[test]
    fn charging_status_as_str_unknown() {
        assert_eq!(ChargingStatus::Unknown.as_str(), "unknown");
    }

    #[test]
    fn charging_status_as_string_returns_owned() {
        assert_eq!(
            ChargingStatus::Charging.as_string(),
            String::from("charging")
        );
    }

    #[test]
    fn is_plugged_in_charging_true() {
        assert!(ChargingStatus::Charging.is_plugged_in());
    }

    #[test]
    fn is_plugged_in_full_true() {
        assert!(ChargingStatus::Full.is_plugged_in());
    }

    #[test]
    fn is_plugged_in_discharging_false() {
        assert!(!ChargingStatus::Discharging.is_plugged_in());
    }

    #[test]
    fn is_plugged_in_unknown_false() {
        assert!(!ChargingStatus::Unknown.is_plugged_in());
    }

    // ---- parse_charging_status ------------------------------------------

    #[test]
    fn parse_charging_charging() {
        assert_eq!(parse_charging_status("Charging"), ChargingStatus::Charging);
    }

    #[test]
    fn parse_charging_discharging() {
        assert_eq!(
            parse_charging_status("Discharging"),
            ChargingStatus::Discharging
        );
    }

    #[test]
    fn parse_charging_full() {
        assert_eq!(parse_charging_status("Full"), ChargingStatus::Full);
    }

    #[test]
    fn parse_charging_not_charging_alias() {
        assert_eq!(parse_charging_status("Not charging"), ChargingStatus::Full);
    }

    #[test]
    fn parse_charging_trims_trailing_newline() {
        assert_eq!(
            parse_charging_status("Charging\n"),
            ChargingStatus::Charging
        );
    }

    #[test]
    fn parse_charging_trims_surrounding_whitespace() {
        assert_eq!(parse_charging_status("  Full  \n"), ChargingStatus::Full);
    }

    #[test]
    fn parse_charging_empty_is_unknown() {
        assert_eq!(parse_charging_status(""), ChargingStatus::Unknown);
    }

    #[test]
    fn parse_charging_whitespace_only_is_unknown() {
        assert_eq!(parse_charging_status("\t \n"), ChargingStatus::Unknown);
    }

    #[test]
    fn parse_charging_garbage_is_unknown() {
        assert_eq!(parse_charging_status("banana"), ChargingStatus::Unknown);
    }

    #[test]
    fn parse_charging_case_sensitive_lowercase_is_unknown() {
        assert_eq!(parse_charging_status("charging"), ChargingStatus::Unknown);
    }

    // ---- parse_power_level ----------------------------------------------

    #[test]
    fn parse_power_level_zero() {
        assert_eq!(parse_power_level("0"), 0);
    }

    #[test]
    fn parse_power_level_full() {
        assert_eq!(parse_power_level("100"), 100);
    }

    #[test]
    fn parse_power_level_trims_newline() {
        assert_eq!(parse_power_level("42\n"), 42);
    }

    #[test]
    fn parse_power_level_trims_spaces() {
        assert_eq!(parse_power_level("  77  "), 77);
    }

    #[test]
    #[should_panic(expected = "failed to parse number")]
    fn parse_power_level_panics_on_garbage() {
        parse_power_level("abc");
    }

    #[test]
    #[should_panic(expected = "failed to parse number")]
    fn parse_power_level_panics_on_empty() {
        parse_power_level("");
    }

    #[test]
    #[should_panic(expected = "failed to parse number")]
    fn parse_power_level_panics_on_negative() {
        parse_power_level("-5");
    }

    // ---- Path helpers ---------------------------------------------------

    #[test]
    fn default_charging_status_path_uses_bat0() {
        assert_eq!(
            get_charging_status_path(None),
            "/sys/class/power_supply/BAT0/status"
        );
    }

    #[test]
    fn explicit_charging_status_path_uses_battery_name() {
        assert_eq!(
            get_charging_status_path(Some("BAT1")),
            "/sys/class/power_supply/BAT1/status"
        );
    }

    #[test]
    fn default_power_status_path_uses_bat0() {
        assert_eq!(
            get_power_status_path(None),
            "/sys/class/power_supply/BAT0/capacity"
        );
    }

    #[test]
    fn explicit_power_status_path_uses_battery_name() {
        assert_eq!(
            get_power_status_path(Some("BAT2")),
            "/sys/class/power_supply/BAT2/capacity"
        );
    }

    // ---- find_lowest_threshold ------------------------------------------

    fn mk_notification(level: u32) -> Notification {
        Notification {
            level,
            urgency: Urgency::Low,
            notified: false,
            time_secs: None,
            command: None,
            title: None,
            message: None,
        }
    }

    fn notified_map(levels: &[u32]) -> HashMap<u32, Notification> {
        let mut map = HashMap::new();
        for &l in levels {
            map.insert(l, mk_notification(l));
        }
        map
    }

    #[test]
    fn lowest_threshold_empty_map_returns_none() {
        let map = notified_map(&[]);
        assert_eq!(find_lowest_threshold(50, &map), None);
    }

    #[test]
    fn lowest_threshold_no_key_ge_current_returns_none() {
        let map = notified_map(&[10, 20]);
        assert_eq!(find_lowest_threshold(50, &map), None);
    }

    #[test]
    fn lowest_threshold_single_match() {
        let map = notified_map(&[30]);
        assert_eq!(find_lowest_threshold(20, &map), Some(30));
    }

    #[test]
    fn lowest_threshold_picks_smallest_ge_current() {
        let map = notified_map(&[10, 20, 30, 80]);
        assert_eq!(find_lowest_threshold(15, &map), Some(20));
    }

    #[test]
    fn lowest_threshold_exact_match_wins() {
        let map = notified_map(&[10, 20, 30]);
        assert_eq!(find_lowest_threshold(20, &map), Some(20));
    }

    #[test]
    fn lowest_threshold_all_keys_ge_current() {
        let map = notified_map(&[50, 60, 70]);
        assert_eq!(find_lowest_threshold(10, &map), Some(50));
    }

    // ---- reset_other_notifications --------------------------------------

    fn notified_map_all_true(levels: &[u32]) -> HashMap<u32, Notification> {
        let mut map = notified_map(levels);
        for n in map.values_mut() {
            n.notified = true;
        }
        map
    }

    #[test]
    fn reset_clears_notified_on_others_only() {
        let mut map = notified_map_all_true(&[10, 20, 30]);
        reset_other_notifications(&20, &mut map);
        assert!(!map.get(&10).unwrap().notified);
        assert!(map.get(&20).unwrap().notified);
        assert!(!map.get(&30).unwrap().notified);
    }

    #[test]
    fn reset_threshold_not_in_map_flips_all() {
        let mut map = notified_map_all_true(&[10, 20, 30]);
        reset_other_notifications(&99, &mut map);
        for n in map.values() {
            assert!(!n.notified);
        }
    }

    #[test]
    fn reset_empty_map_no_panic() {
        let mut map: HashMap<u32, Notification> = HashMap::new();
        reset_other_notifications(&10, &mut map);
        assert!(map.is_empty());
    }

    // ---- check_notify_full_battery (no-side-effect branches only) ------
    //
    // The `current >= 100` branch is intentionally NOT covered here; it
    // calls send_message / run_command which require a live D-Bus session
    // and a notification server. Manual smoke-test that path on a real
    // desktop before release.

    fn mk_full_notification() -> BatteryFullNotification {
        BatteryFullNotification {
            urgency: Urgency::Low,
            notified: false,
            time_secs: None,
            enabled: true,
            command: None,
            title: Some("t".to_string()),
            message: Some("m".to_string()),
        }
    }

    #[test]
    fn check_full_already_notified_short_circuits() {
        let mut n = mk_full_notification();
        n.notified = true;
        check_notify_full_battery(&100, &99, &mut n);
        assert!(n.notified, "should remain true");
    }

    #[test]
    fn check_full_disabled_short_circuits() {
        let mut n = mk_full_notification();
        n.enabled = false;
        check_notify_full_battery(&100, &99, &mut n);
        assert!(!n.notified);
    }

    #[test]
    fn check_full_decreasing_resets_notified_and_returns() {
        let mut n = mk_full_notification();
        check_notify_full_battery(&80, &90, &mut n);
        assert!(!n.notified);
    }

    #[test]
    fn check_full_equal_levels_is_decreasing() {
        let mut n = mk_full_notification();
        check_notify_full_battery(&80, &80, &mut n);
        assert!(!n.notified);
    }

    #[test]
    fn check_full_under_100_does_not_notify() {
        let mut n = mk_full_notification();
        check_notify_full_battery(&99, &98, &mut n);
        assert!(!n.notified);
    }
}

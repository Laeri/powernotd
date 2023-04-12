use notify_rust::Urgency;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};

#[derive(Debug)]
struct Notification {
    // threshold level for which a notification should be sent
    level: u32,
    // urgency of the message, notification daemon can display them with different styling based on
    // the urgency
    urgency: Urgency,
    // notified is true if for the given threshold a notification has been sent out already
    notified: bool,

    // how long the notification is displayed
    time_secs: Option<u32>,
}

/// Return the current battery level
fn get_current_power() -> u32 {
    let mut file = File::open("/sys/class/power_supply/BAT0/capacity").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let current_level: u32 = contents.trim().parse().expect("failed to parse number");
    return current_level;
}

/// send a message using linux notify-send api
fn send_message(title: &str, message: &str, urgency: &Urgency, time_secs: Option<u32>) {
    let mut notification = notify_rust::Notification::new();

    notification.summary(title).body(message).urgency(*urgency);

    if let Some(wait_time) = time_secs {
        notification.timeout(notify_rust::Timeout::Milliseconds(wait_time * 1000));
        //milliseconds
    }
    notification.show().unwrap();
}

/// Find lowest threshold which has been passed with the current battery level
fn find_lowest_threshold(current: u32, notified: &HashMap<u32, Notification>) -> Option<u32> {
    let keys = notified.keys().cloned().collect::<Vec<_>>();

    let result = keys.into_iter().filter(|&key| key >= current).min();

    return result;
}

/// Reset all notifications which are not the current threshold_val
fn reset_other_notifications(threshold_val: &u32, notified: &mut HashMap<u32, Notification>) {
    for (key, mut notification) in notified.iter_mut() {
        if *key != *threshold_val {
            notification.notified = false;
        }
    }
}

/// notify if battery is fully charged
fn check_notify_full_battery(current: &u32, last: &u32, is_full_notified: &mut bool) {
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

fn main() {
    let sleep_time = time::Duration::from_secs(60);
    let mut last_battery_level: u32 = 100;

    const CRITICAL_WAIT_TIME_SECS: u32 = 10000;

    // notify once when battery is fully charged
    let mut is_full_notified = false;

    let notifications = vec![
        Notification {
            level: 30,
            urgency: Urgency::Low,
            notified: false,
            time_secs: None,
        },
        Notification {
            level: 20,
            urgency: Urgency::Normal,
            notified: false,
            time_secs: None,
        },
        Notification {
            level: 15,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
        },
        Notification {
            level: 10,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
        },
        Notification {
            level: 5,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
        },
        Notification {
            level: 2,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
        },
        Notification {
            level: 1,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
        },
    ];

    let mut notified = HashMap::new();

    for notification in notifications {
        notified.insert(notification.level, notification);
    }

    loop {
        let level = get_current_power();
        println!("level {}", level.to_string());
        let current_threshold = find_lowest_threshold(level, &notified);
        for (key, value) in &notified {
            println!("{}: {:?}", key, value);
        }
        if let Some(threshold_val) = current_threshold {
            if let Some(notification) = notified.get_mut(&threshold_val) {
                if !notification.notified && level < last_battery_level {
                    send_message(
                        "Battery Status",
                        format!("{current_level}%", current_level = level).as_str(),
                        &notification.urgency,
                        notification.time_secs,
                    );
                    notification.notified = true;
                }
            }
            reset_other_notifications(&threshold_val, &mut notified);
        }

        check_notify_full_battery(&level, &last_battery_level, &mut is_full_notified);

        last_battery_level = level;

        thread::sleep(sleep_time);
    }
}

mod cli;

use crate::cli::Args;
use clap::Parser;
use notify_rust::Urgency;
use powernotd::*;
use std::collections::HashMap;
use std::{thread, time};

fn main() {
    let args = Args::parse();

    if args.status_level {
        let current = get_current_power();
        println!("{}%", current);
        return;
    }

    if args.status_state {
        let status = get_status_charging();
        println!("{}", status);
        return;
    }

    if args.notify_now {}

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
                    send_notification(&level, &notification);
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

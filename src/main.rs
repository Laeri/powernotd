mod cli;

use crate::cli::Args;
use clap::Parser;
use powernotd::config;
use powernotd::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{thread, time};

use powernotd::notification::Notification;

fn main() {
    let args = Args::parse();

    // these paths are required for reading power supply status
    let required_paths = vec![
        PathBuf::from("/sys/class/power_supply/BAT0/status"),
        PathBuf::from("/sys/class/power_supply/BAT0/capacity"),
    ];
    for path in required_paths {
        if !path.exists() {
            eprintln!(
                "Require file at path: {} order to read power status!",
                path.to_string_lossy()
            );
            std::process::exit(1);
        }
    }

    if args.status_level {
        let current = get_current_power();
        println!("{}%", current);
        return;
    }

    if args.charging_state {
        let status = get_status_charging();
        println!("{}", status);
        return;
    }

    if args.notify_now {}

    let sleep_time = time::Duration::from_secs(60);
    let mut last_battery_level: u32 = 100;

    let mut notified: HashMap<u32, Notification> = HashMap::new();

    let mut config = match args.config_file {
        Some(string) => {
            let path = PathBuf::from(string);
            config::get_specific_config(path)
        }
        None => config::get_or_create_config(),
    };

    for notification in config.notifications {
        notified.insert(notification.level, notification);
    }

    loop {
        let level = get_current_power();
        let current_threshold = find_lowest_threshold(level, &notified);
        if let Some(threshold_val) = current_threshold {
            if let Some(notification) = notified.get_mut(&threshold_val) {
                if !notification.notified && level < last_battery_level {
                    send_notification(&level, notification);
                    notification.notified = true;
                }
            }
            reset_other_notifications(&threshold_val, &mut notified);
        }

        check_notify_full_battery(&level, &last_battery_level, &mut config.full_notification);

        last_battery_level = level;

        thread::sleep(sleep_time);
    }
}

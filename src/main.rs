mod cli;

use crate::cli::Args;
use clap::Parser;
use powernotd::config;
use powernotd::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::{thread, time};

use powernotd::notification::{ChargingHook, Notification};

fn plugin_plugout_check_fallback(
    battery: Option<&Battery>,
    level: u32,
    last_charging_status: &mut Option<ChargingStatus>,
    charging_start: &Option<Arc<ChargingHook>>,
    charging_stop: &Option<Arc<ChargingHook>>,
) {
    let current = get_charging_status(battery);
    if matches!(current, ChargingStatus::Unknown) {
        return;
    }
    if let Some(prev) = last_charging_status {
        let was_plugged = prev.is_plugged_in();
        let is_plugged = current.is_plugged_in();
        if !was_plugged && is_plugged {
            if let Some(hook) = charging_start {
                fire_plugin_plugout_hook(level, hook);
            }
        } else if was_plugged && !is_plugged {
            if let Some(hook) = charging_stop {
                fire_plugin_plugout_hook(level, hook);
            }
        }
    }
    *last_charging_status = Some(current);
}

fn main() {
    let args = Args::parse();

    let battery: Option<&Battery> = args.battery.as_deref();

    // these paths are required for reading power supply status
    let required_paths = vec![
        PathBuf::from(get_power_status_path(battery)),
        PathBuf::from(get_charging_status_path(battery)),
    ];
    for path in required_paths {
        if !path.exists() {
            eprintln!(
                "Require file at path: {} order to read power status! If you have a different battery such as BAT1 pass it using the -b flag.",
                path.to_string_lossy()
            );
            std::process::exit(1);
        }
    }

    if args.status_level {
        let current = get_current_power(battery);
        println!("{}%", current);
        return;
    }

    if args.charging_state {
        let status = get_charging_status_text(battery);
        println!("{}", status);
        return;
    }

    if args.notify_now {
        let current = get_current_power(battery);
        notify_now(&current);
        return;
    }

    let mut config = match args.config_file {
        Some(string) => {
            let path = PathBuf::from(string);
            config::get_specific_config(path)
        }
        None => config::get_or_create_config(),
    };

    if args.list_thresholds {
        let mut levels = config
            .notifications
            .iter()
            .map(|notification| notification.level)
            .collect::<Vec<u32>>();

        levels.sort();

        let output = levels
            .iter()
            .map(|level| format!("{}%", level))
            .collect::<Vec<String>>();
        println!("{}", &output.join(", "));
        return;
    }

    if args.show_config_path {
        let config_path = config::get_default_config_path();
        println!("{}", config_path.unwrap_or_default().to_string_lossy());
        return;
    }

    let sleep_time = time::Duration::from_secs(config.poll_interval_secs.max(1) as u64);
    let mut last_battery_level: u32 = 100;
    let mut last_charging_status: Option<ChargingStatus> = None;

    let charging_start_arc = config.charging_start.take().map(Arc::new);
    let charging_stop_arc = config.charging_stop.take().map(Arc::new);

    // Prefer event-driven plug/unplug detection via UPower. If unavailable
    // (no D-Bus, UPower not running, or device path not found), fall back to
    // detecting transitions in the poll loop.
    let upower_active = match upower::try_spawn_listener(
        args.battery.clone(),
        charging_start_arc.clone(),
        charging_stop_arc.clone(),
    ) {
        Ok(()) => true,
        Err(err) => {
            eprintln!(
                "UPower unavailable ({}), using poll-based plug/unplug detection",
                err
            );
            false
        }
    };

    let mut notified: HashMap<u32, Notification> = HashMap::new();

    for notification in config.notifications {
        notified.insert(notification.level, notification);
    }

    loop {
        let level = get_current_power(battery);
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

        if !upower_active {
            plugin_plugout_check_fallback(
                battery,
                level,
                &mut last_charging_status,
                &charging_start_arc,
                &charging_stop_arc,
            );
        }

        thread::sleep(sleep_time);
    }
}

use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::notification::{
    BatteryFullNotification, ChargingHookBase, ChargingStartHook, ChargingStopHook, Notification,
    Urgency,
};

pub const CRITICAL_WAIT_TIME_SECS: u32 = 10000;
pub const DEFAULT_POLL_INTERVAL_SECS: u32 = 60;

const CONFIG_NAME: &str = "config.json";

fn default_poll_interval_secs() -> u32 {
    DEFAULT_POLL_INTERVAL_SECS
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(default)]
pub struct PluggedInStartupNotification {
    pub show_full: bool,
    pub show_threshold: bool,
}

impl Default for PluggedInStartupNotification {
    fn default() -> Self {
        Self {
            show_full: true,
            show_threshold: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub notifications: Vec<Notification>,
    pub full_notification: BatteryFullNotification,

    #[serde(default)]
    pub plugged_in_startup_notification: PluggedInStartupNotification,

    // Optional hook fired when the AC adapter is plugged in
    // (Discharging -> Charging or Discharging -> Full).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charging_start: Option<ChargingStartHook>,

    // Optional hook fired when the AC adapter is unplugged
    // (Charging/Full -> Discharging).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charging_stop: Option<ChargingStopHook>,

    // How often the poll loop reads the battery (seconds). Also bounds how
    // quickly plug/unplug hooks fire when UPower is unavailable.
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u32,
}

#[derive(Debug)]
pub enum Error {
    LoadConfigError,
    SaveDefaultConfigError,
}

impl From<std::io::Error> for Error {
    fn from(_value: std::io::Error) -> Self {
        Error::LoadConfigError
    }
}

pub fn get_specific_config(file_path: PathBuf) -> Config {
    match load_config_from_file(&file_path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!(
                "Could not load file_path {}, error: {:?}",
                file_path.to_string_lossy(),
                err
            );
            get_default_config()
        }
    }
}

pub fn get_or_create_config() -> Config {
    if let Some(config_file) = get_default_config_path() {
        if !config_file.exists() {
            if let Err(err) = save_default_config() {
                eprintln!(
                    "could not save default configuration file to path: {}, err: {:?}",
                    config_file.to_string_lossy(),
                    err
                );
            };
            get_default_config()
        } else {
            load_config_from_file(&config_file).unwrap_or_else(|err| {
                eprintln!(
                    "Could not load config: {}, error: {:?}",
                    config_file.to_string_lossy(),
                    err
                );
                get_default_config()
            })
        }
    } else {
        get_default_config()
    }
}

pub fn load_config_from_file(path: &PathBuf) -> Result<Config, Error> {
    let text = std::fs::read_to_string(path)?;
    match serde_json::from_str::<Config>(&text) {
        Ok(config) => Ok(config),
        Err(error) => {
            eprintln!("Error loading config from file, error: {}", error);
            Err(Error::LoadConfigError)
        }
    }
}

pub fn get_default_config_path() -> Option<PathBuf> {
    let dir_result = get_config_dir()?;
    let file_path = dir_result.join(CONFIG_NAME);
    Some(file_path)
}

fn save_default_config() -> Result<(), Error> {
    let path = get_default_config_path();

    if path.is_none() {
        return Err(Error::LoadConfigError);
    }
    let config_file = path.unwrap();

    let dir_result = config_file.parent();
    if dir_result.is_none() {
        return Err(Error::LoadConfigError);
    }

    let config_dir = dir_result.unwrap();
    // create config directory if needed
    if !config_dir.exists() && std::fs::create_dir(config_dir).is_err() {
        return Err(Error::SaveDefaultConfigError);
    }

    // config file should not exist yet
    if config_file.exists() {
        return Err(Error::SaveDefaultConfigError);
    }
    let default_config = get_default_config();
    let string = serde_json::to_string_pretty(&default_config);
    if string.is_err() {
        return Err(Error::SaveDefaultConfigError);
    }
    let result = std::fs::write(config_file, string.unwrap());
    match result {
        Ok(_) => Ok(()),
        Err(_) => {
            eprintln!("Could not write default config to file!");
            Err(Error::SaveDefaultConfigError)
        }
    }
}

fn get_config_dir() -> Option<PathBuf> {
    ProjectDirs::from("me", "laeri", "powernotd").map(|dir| dir.config_dir().to_owned())
}

pub fn get_default_config() -> Config {
    let default_title = "Battery Status";
    let default_message = "{}%";
    let notifications = vec![
        Notification {
            level: 30,
            urgency: Urgency::Low,
            notified: false,
            time_secs: None,
            command: None,
            title: Some(default_title.to_string()),
            message: Some(default_message.to_string()),
        },
        Notification {
            level: 20,
            urgency: Urgency::Normal,
            notified: false,
            time_secs: None,
            command: None,
            title: Some(default_title.to_string()),
            message: Some(default_message.to_string()),
        },
        Notification {
            level: 15,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
            command: None,
            title: Some(default_title.to_string()),
            message: Some(default_message.to_string()),
        },
        Notification {
            level: 10,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
            command: None,
            title: Some(default_title.to_string()),
            message: Some(default_message.to_string()),
        },
        Notification {
            level: 5,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
            command: None,
            title: Some("Critical Battery Status".to_string()),
            message: Some(default_message.to_string()),
        },
        Notification {
            level: 2,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
            command: None,
            title: Some("Critical Battery Status".to_string()),
            message: Some(default_message.to_string()),
        },
        Notification {
            level: 1,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(CRITICAL_WAIT_TIME_SECS),
            command: None,
            title: Some("Critical Battery Status".to_string()),
            message: Some(default_message.to_string()),
        },
    ];

    let full_notification = crate::notification::BatteryFullNotification {
        urgency: Urgency::Low,
        notified: false,
        time_secs: None,
        enabled: true,
        command: None,
        title: Some("Battery Status".to_string()),
        message: Some("Fully Charged 100%".to_string()),
    };

    let charging_start = ChargingStartHook {
        base: ChargingHookBase {
            urgency: Urgency::Low,
            enabled: false,
            time_secs: None,
            command: Some("paplay /usr/share/sounds/freedesktop/stereo/power-plug.oga".to_string()),
            title: Some("Charging".to_string()),
            message: Some("Plugged in at {}%".to_string()),
        },
    };

    let charging_stop = ChargingStopHook {
        base: ChargingHookBase {
            urgency: Urgency::Low,
            enabled: false,
            time_secs: None,
            command: Some("paplay /usr/share/sounds/freedesktop/stereo/power-unplug.oga".to_string()),
            title: Some("Discharging".to_string()),
            message: Some("Unplugged at {}%".to_string()),
        },
        show_threshold_warning_on_unplug: true,
    };

    Config {
        notifications,
        full_notification,
        plugged_in_startup_notification: PluggedInStartupNotification::default(),
        charging_start: Some(charging_start),
        charging_stop: Some(charging_stop),
        poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::Urgency;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_tmp(contents: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("create tempfile");
        f.write_all(contents.as_bytes()).expect("write tempfile");
        f.flush().expect("flush tempfile");
        f
    }

    // ---- load_config_from_file -----------------------------------------

    #[test]
    fn load_config_from_file_valid_json() {
        let json = r#"{
          "notifications": [
            {"level": 25, "urgency": "Low"},
            {"level": 10, "urgency": "Critical"}
          ],
          "full_notification": {"urgency": "Low", "enabled": true},
          "poll_interval_secs": 30
        }"#;
        let f = write_tmp(json);
        let cfg = load_config_from_file(&f.path().to_path_buf()).expect("load ok");
        assert_eq!(cfg.notifications.len(), 2);
        assert_eq!(cfg.poll_interval_secs, 30);
        assert!(cfg.full_notification.enabled);
    }

    #[test]
    fn load_config_from_file_minimal_applies_defaults() {
        let json = r#"{
          "notifications": [],
          "full_notification": {"urgency": "Low", "enabled": false}
        }"#;
        let f = write_tmp(json);
        let cfg = load_config_from_file(&f.path().to_path_buf()).expect("load ok");
        assert_eq!(cfg.poll_interval_secs, DEFAULT_POLL_INTERVAL_SECS);
        assert!(cfg.charging_start.is_none());
        assert!(cfg.charging_stop.is_none());
        assert!(cfg.plugged_in_startup_notification.show_full);
        assert!(!cfg.plugged_in_startup_notification.show_threshold);
    }

    #[test]
    fn load_config_from_file_partial_plugged_in_startup_applies_defaults() {
        let json = r#"{
          "notifications": [],
          "full_notification": {"urgency": "Low", "enabled": false},
          "plugged_in_startup_notification": {"show_threshold": true}
        }"#;
        let f = write_tmp(json);
        let cfg = load_config_from_file(&f.path().to_path_buf()).expect("load ok");
        assert!(cfg.plugged_in_startup_notification.show_full);
        assert!(cfg.plugged_in_startup_notification.show_threshold);
    }

    #[test]
    fn load_config_from_file_custom_poll_interval() {
        let json = r#"{
          "notifications": [],
          "full_notification": {"urgency": "Low", "enabled": false},
          "poll_interval_secs": 5
        }"#;
        let f = write_tmp(json);
        let cfg = load_config_from_file(&f.path().to_path_buf()).expect("load ok");
        assert_eq!(cfg.poll_interval_secs, 5);
    }

    #[test]
    fn load_config_from_file_malformed_json_errors() {
        let f = write_tmp("{ not json");
        let err = load_config_from_file(&f.path().to_path_buf()).unwrap_err();
        assert!(matches!(err, Error::LoadConfigError));
    }

    #[test]
    fn load_config_from_file_missing_path_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist.json");
        let err = load_config_from_file(&missing).unwrap_err();
        assert!(matches!(err, Error::LoadConfigError));
    }

    // ---- get_specific_config fallback ----------------------------------

    #[test]
    fn get_specific_config_falls_back_on_malformed() {
        let f = write_tmp("{ not json");
        let cfg = get_specific_config(f.path().to_path_buf());
        assert_eq!(cfg.notifications.len(), 7);
    }

    #[test]
    fn get_specific_config_falls_back_on_missing_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("missing.json");
        let cfg = get_specific_config(missing);
        assert_eq!(cfg.notifications.len(), 7);
    }

    // ---- get_default_config sanity -------------------------------------

    #[test]
    fn get_default_config_has_seven_thresholds() {
        let cfg = get_default_config();
        assert_eq!(cfg.notifications.len(), 7);
    }

    #[test]
    fn get_default_config_threshold_levels_match() {
        let cfg = get_default_config();
        let mut levels: Vec<u32> = cfg.notifications.iter().map(|n| n.level).collect();
        levels.sort();
        assert_eq!(levels, vec![1, 2, 5, 10, 15, 20, 30]);
    }

    #[test]
    fn get_default_config_full_notification_enabled() {
        let cfg = get_default_config();
        assert!(cfg.full_notification.enabled);
        assert!(!cfg.full_notification.notified);
    }

    #[test]
    fn get_default_config_charging_hooks_present_but_disabled() {
        let cfg = get_default_config();
        let start = cfg.charging_start.expect("charging_start present");
        let stop = cfg.charging_stop.expect("charging_stop present");
        assert!(!start.base.enabled);
        assert!(!stop.base.enabled);
        assert!(stop.show_threshold_warning_on_unplug);
    }

    #[test]
    fn get_default_config_charging_hooks_have_example_commands() {
        let cfg = get_default_config();
        let start = cfg.charging_start.expect("charging_start present");
        let stop = cfg.charging_stop.expect("charging_stop present");
        assert!(start.base.command.is_some());
        assert!(stop.base.command.is_some());
        assert!(start
            .base
            .command
            .as_ref()
            .unwrap()
            .contains("power-plug"));
        assert!(stop
            .base
            .command
            .as_ref()
            .unwrap()
            .contains("power-unplug"));
    }

    #[test]
    fn get_default_config_poll_interval_is_60() {
        let cfg = get_default_config();
        assert_eq!(cfg.poll_interval_secs, 60);
    }

    #[test]
    fn get_default_config_plugged_in_startup_notification_defaults() {
        let cfg = get_default_config();
        assert!(cfg.plugged_in_startup_notification.show_full);
        assert!(!cfg.plugged_in_startup_notification.show_threshold);
    }

    #[test]
    fn get_default_config_serde_roundtrip_preserves_show_threshold_warning_on_unplug() {
        let cfg = get_default_config();
        let s = serde_json::to_string(&cfg).expect("serialize default config");
        let back: Config = serde_json::from_str(&s).expect("deserialize default config");
        let stop = back.charging_stop.expect("charging_stop present");
        assert!(stop.show_threshold_warning_on_unplug);
    }

    #[test]
    fn get_default_config_critical_levels_have_wait_time() {
        let cfg = get_default_config();
        for n in &cfg.notifications {
            match n.level {
                15 | 10 | 5 | 2 | 1 => assert_eq!(
                    n.time_secs,
                    Some(CRITICAL_WAIT_TIME_SECS),
                    "level {} should have critical wait time",
                    n.level
                ),
                30 | 20 => assert_eq!(
                    n.time_secs, None,
                    "level {} should have no wait time",
                    n.level
                ),
                other => panic!("unexpected default level {}", other),
            }
        }
    }

    #[test]
    fn get_default_config_urgencies_at_expected_levels() {
        let cfg = get_default_config();
        let by_level: std::collections::HashMap<u32, Urgency> = cfg
            .notifications
            .iter()
            .map(|n| (n.level, n.urgency))
            .collect();
        assert_eq!(by_level.get(&30).copied(), Some(Urgency::Low));
        assert_eq!(by_level.get(&20).copied(), Some(Urgency::Normal));
        assert_eq!(by_level.get(&15).copied(), Some(Urgency::Critical));
        assert_eq!(by_level.get(&1).copied(), Some(Urgency::Critical));
    }
}

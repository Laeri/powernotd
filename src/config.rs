use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::notification::{BatteryFullNotification, Notification, Urgency};

pub const CRITICAL_WAIT_TIME_SECS: u32 = 10000;

const CONFIG_NAME: &str = "config.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub notifications: Vec<Notification>,
    pub full_notification: BatteryFullNotification,
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

fn load_config_from_file(path: &PathBuf) -> Result<Config, Error> {
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

fn get_default_config() -> Config {
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

    Config {
        notifications,
        full_notification,
    }
}

use notify_rust::Urgency as SendUrgency;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Urgency {
    /// The behaviour for `Low` urgency depends on the notification server.
    Low = 0,
    /// The behaviour for `Normal` urgency depends on the notification server.
    Normal = 1,
    /// A critical notification will not time out.
    Critical = 2,
}

impl From<&Urgency> for SendUrgency {
    fn from(value: &Urgency) -> Self {
        match value {
            Urgency::Low => SendUrgency::Low,
            Urgency::Normal => SendUrgency::Normal,
            Urgency::Critical => SendUrgency::Critical,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    // threshold level for which a notification should be sent
    pub level: u32,
    // urgency of the message, notification daemon can display them with different styling based on
    // the urgency
    pub urgency: Urgency,
    // notified is true if for the given threshold a notification has been sent out already
    #[serde(default, skip_serializing)]
    pub notified: bool,

    // how long the notification is displayed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_secs: Option<u32>,

    // optional command/script that should should be run on notification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    // optional title to use for notification message
    // use {} for inserting percentage into template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    // optional template to use for notification message
    // use {} for inserting percentage into template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatteryFullNotification {
    pub urgency: Urgency,
    #[serde(default, skip_serializing)]
    pub notified: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_secs: Option<u32>,
    // if disabled no notification is sent when battery is full
    pub enabled: bool,

    // optional script to run on notification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    // optional title to use for notification message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    // optional template to use for notification message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

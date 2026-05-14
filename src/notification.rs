use notify_rust::Urgency as SendUrgency;
use serde::{Deserialize, Serialize};

fn default_show_threshold_warning() -> bool {
    true
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChargingHook {
    pub urgency: Urgency,
    // if disabled the hook does not fire
    pub enabled: bool,

    // optional notification display duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_secs: Option<u32>,

    // optional script to run on plug/unplug event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    // optional notification title; '{}' is replaced with the current battery level.
    // If both title and message are absent, no notification is shown (command-only hook).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    // optional notification message; '{}' is replaced with the current battery level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    // Only consulted on the `charging_stop` hook (i.e. when the AC adapter is
    // unplugged): show the matching threshold warning on unplug if the current
    // battery level is already below a configured threshold. On `charging_start`
    // this field is parsed but ignored — by the time plug-in fires, any
    // relevant threshold warning has already either fired during the preceding
    // discharge or did not apply. Kept on the shared `ChargingHook` type so the
    // config format stays uniform.
    #[serde(default = "default_show_threshold_warning")]
    pub show_threshold_warning: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Urgency -> notify_rust::Urgency ---------------------------------

    #[test]
    fn urgency_into_notify_rust_low() {
        assert!(matches!(SendUrgency::from(&Urgency::Low), SendUrgency::Low));
    }

    #[test]
    fn urgency_into_notify_rust_normal() {
        assert!(matches!(
            SendUrgency::from(&Urgency::Normal),
            SendUrgency::Normal
        ));
    }

    #[test]
    fn urgency_into_notify_rust_critical() {
        assert!(matches!(
            SendUrgency::from(&Urgency::Critical),
            SendUrgency::Critical
        ));
    }

    // ---- Urgency serde ---------------------------------------------------

    #[test]
    fn urgency_serializes_as_named_variant_low() {
        assert_eq!(serde_json::to_string(&Urgency::Low).unwrap(), "\"Low\"");
    }

    #[test]
    fn urgency_serializes_as_named_variant_normal() {
        assert_eq!(
            serde_json::to_string(&Urgency::Normal).unwrap(),
            "\"Normal\""
        );
    }

    #[test]
    fn urgency_serializes_as_named_variant_critical() {
        assert_eq!(
            serde_json::to_string(&Urgency::Critical).unwrap(),
            "\"Critical\""
        );
    }

    #[test]
    fn urgency_serde_roundtrip_low() {
        let s = serde_json::to_string(&Urgency::Low).unwrap();
        let back: Urgency = serde_json::from_str(&s).unwrap();
        assert_eq!(back, Urgency::Low);
    }

    #[test]
    fn urgency_serde_roundtrip_normal() {
        let s = serde_json::to_string(&Urgency::Normal).unwrap();
        let back: Urgency = serde_json::from_str(&s).unwrap();
        assert_eq!(back, Urgency::Normal);
    }

    #[test]
    fn urgency_serde_roundtrip_critical() {
        let s = serde_json::to_string(&Urgency::Critical).unwrap();
        let back: Urgency = serde_json::from_str(&s).unwrap();
        assert_eq!(back, Urgency::Critical);
    }

    // ---- Notification serde ---------------------------------------------

    fn full_notification() -> Notification {
        Notification {
            level: 42,
            urgency: Urgency::Critical,
            notified: false,
            time_secs: Some(10),
            command: Some("notify-send hi".to_string()),
            title: Some("t".to_string()),
            message: Some("{}%".to_string()),
        }
    }

    #[test]
    fn notification_roundtrip_full() {
        let n = full_notification();
        let s = serde_json::to_string(&n).unwrap();
        let back: Notification = serde_json::from_str(&s).unwrap();
        assert_eq!(back.level, 42);
        assert_eq!(back.urgency, Urgency::Critical);
        assert_eq!(back.time_secs, Some(10));
        assert_eq!(back.command.as_deref(), Some("notify-send hi"));
        assert_eq!(back.title.as_deref(), Some("t"));
        assert_eq!(back.message.as_deref(), Some("{}%"));
    }

    #[test]
    fn notification_roundtrip_minimal_applies_defaults() {
        let json = r#"{"level":15,"urgency":"Normal"}"#;
        let back: Notification = serde_json::from_str(json).unwrap();
        assert_eq!(back.level, 15);
        assert_eq!(back.urgency, Urgency::Normal);
        assert!(!back.notified);
        assert!(back.time_secs.is_none());
        assert!(back.command.is_none());
        assert!(back.title.is_none());
        assert!(back.message.is_none());
    }

    #[test]
    fn notification_skips_none_fields_on_serialize() {
        let n = Notification {
            level: 5,
            urgency: Urgency::Low,
            notified: false,
            time_secs: None,
            command: None,
            title: None,
            message: None,
        };
        let s = serde_json::to_string(&n).unwrap();
        assert!(!s.contains("time_secs"), "got: {s}");
        assert!(!s.contains("command"), "got: {s}");
        assert!(!s.contains("title"), "got: {s}");
        assert!(!s.contains("message"), "got: {s}");
    }

    #[test]
    fn notification_notified_not_serialized() {
        let mut n = full_notification();
        n.notified = true;
        let s = serde_json::to_string(&n).unwrap();
        assert!(!s.contains("notified"), "got: {s}");
        let back: Notification = serde_json::from_str(&s).unwrap();
        assert!(!back.notified);
    }

    // ---- ChargingHook serde ---------------------------------------------

    #[test]
    fn charging_hook_roundtrip_full() {
        let h = ChargingHook {
            urgency: Urgency::Normal,
            enabled: true,
            time_secs: Some(5),
            command: Some("xset dpms force on".to_string()),
            title: Some("Charging".to_string()),
            message: Some("Plugged in at {}%".to_string()),
            show_threshold_warning: true,
        };
        let s = serde_json::to_string(&h).unwrap();
        let back: ChargingHook = serde_json::from_str(&s).unwrap();
        assert_eq!(back.urgency, Urgency::Normal);
        assert!(back.enabled);
        assert_eq!(back.time_secs, Some(5));
        assert_eq!(back.command.as_deref(), Some("xset dpms force on"));
        assert_eq!(back.title.as_deref(), Some("Charging"));
        assert_eq!(back.message.as_deref(), Some("Plugged in at {}%"));
        assert!(back.show_threshold_warning);
    }

    #[test]
    fn charging_hook_roundtrip_minimal() {
        let json = r#"{"urgency":"Low","enabled":false}"#;
        let back: ChargingHook = serde_json::from_str(json).unwrap();
        assert_eq!(back.urgency, Urgency::Low);
        assert!(!back.enabled);
        assert!(back.time_secs.is_none());
        assert!(back.command.is_none());
        assert!(back.title.is_none());
        assert!(back.message.is_none());
        assert!(back.show_threshold_warning);
    }

    #[test]
    fn charging_hook_skips_none_command_on_serialize() {
        let h = ChargingHook {
            urgency: Urgency::Low,
            enabled: false,
            time_secs: None,
            command: None,
            title: None,
            message: None,
            show_threshold_warning: false,
        };
        let s = serde_json::to_string(&h).unwrap();
        assert!(!s.contains("command"), "got: {s}");
        assert!(!s.contains("title"), "got: {s}");
        assert!(!s.contains("message"), "got: {s}");
        assert!(!s.contains("time_secs"), "got: {s}");
    }

    #[test]
    fn charging_hook_show_threshold_warning_false_survives_roundtrip() {
        let h = ChargingHook {
            urgency: Urgency::Low,
            enabled: false,
            time_secs: None,
            command: None,
            title: None,
            message: None,
            show_threshold_warning: false,
        };
        let s = serde_json::to_string(&h).unwrap();
        let back: ChargingHook = serde_json::from_str(&s).unwrap();
        assert!(!back.show_threshold_warning);
    }

    #[test]
    fn charging_hook_show_threshold_warning_always_serialized() {
        let h = ChargingHook {
            urgency: Urgency::Low,
            enabled: false,
            time_secs: None,
            command: None,
            title: None,
            message: None,
            show_threshold_warning: false,
        };
        let s = serde_json::to_string(&h).unwrap();
        assert!(s.contains("\"show_threshold_warning\":false"), "got: {s}");
    }

    #[test]
    fn charging_hook_minimal_json_defaults_show_threshold_warning_to_true() {
        let json = r#"{"urgency":"Low","enabled":false}"#;
        let back: ChargingHook = serde_json::from_str(json).unwrap();
        assert!(back.show_threshold_warning);
    }

    // ---- BatteryFullNotification serde ----------------------------------

    #[test]
    fn battery_full_roundtrip_full() {
        let b = BatteryFullNotification {
            urgency: Urgency::Low,
            notified: false,
            time_secs: Some(10),
            enabled: true,
            command: Some("echo full".to_string()),
            title: Some("Battery Status".to_string()),
            message: Some("Fully Charged 100%".to_string()),
        };
        let s = serde_json::to_string(&b).unwrap();
        let back: BatteryFullNotification = serde_json::from_str(&s).unwrap();
        assert_eq!(back.urgency, Urgency::Low);
        assert!(back.enabled);
        assert_eq!(back.time_secs, Some(10));
        assert_eq!(back.command.as_deref(), Some("echo full"));
        assert_eq!(back.title.as_deref(), Some("Battery Status"));
        assert_eq!(back.message.as_deref(), Some("Fully Charged 100%"));
    }

    #[test]
    fn battery_full_roundtrip_minimal() {
        let json = r#"{"urgency":"Critical","enabled":true}"#;
        let back: BatteryFullNotification = serde_json::from_str(json).unwrap();
        assert_eq!(back.urgency, Urgency::Critical);
        assert!(back.enabled);
        assert!(!back.notified);
        assert!(back.time_secs.is_none());
        assert!(back.command.is_none());
        assert!(back.title.is_none());
        assert!(back.message.is_none());
    }

    #[test]
    fn battery_full_notified_not_serialized() {
        let b = BatteryFullNotification {
            urgency: Urgency::Low,
            notified: true,
            time_secs: None,
            enabled: true,
            command: None,
            title: None,
            message: None,
        };
        let s = serde_json::to_string(&b).unwrap();
        assert!(!s.contains("notified"), "got: {s}");
        let back: BatteryFullNotification = serde_json::from_str(&s).unwrap();
        assert!(!back.notified);
    }
}

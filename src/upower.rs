use crate::notification::{ChargingStartHook, ChargingStopHook};
use crate::{ChargingStatus, DEFAULT_BATTERY, fire_plugin_plugout_hook, get_current_power};
use std::sync::Arc;
use std::thread;
use zbus::blocking::{Connection, fdo::PropertiesProxy};
use zbus::names::InterfaceName;
use zbus::zvariant::ObjectPath;

const UPOWER_DEST: &str = "org.freedesktop.UPower";
const UPOWER_DEVICE_IFACE: &str = "org.freedesktop.UPower.Device";
const UPOWER_DEVICE_PATH_PREFIX: &str = "/org/freedesktop/UPower/devices/battery_";

#[derive(Debug, PartialEq, Eq)]
enum UPowerState {
    Unknown,
    Charging,
    Discharging,
    Empty,
    FullyCharged,
    PendingCharge,
    PendingDischarge,
}

impl UPowerState {
    fn from_raw(raw: u32) -> Self {
        match raw {
            1 => Self::Charging,
            2 => Self::Discharging,
            3 => Self::Empty,
            4 => Self::FullyCharged,
            5 => Self::PendingCharge,
            6 => Self::PendingDischarge,
            _ => Self::Unknown,
        }
    }
}

fn map_upower_state(state: u32) -> ChargingStatus {
    match UPowerState::from_raw(state) {
        UPowerState::Charging | UPowerState::PendingCharge => ChargingStatus::Charging,
        UPowerState::Discharging | UPowerState::PendingDischarge => ChargingStatus::Discharging,
        UPowerState::FullyCharged => ChargingStatus::Full,
        UPowerState::Unknown | UPowerState::Empty => ChargingStatus::Unknown,
    }
}

// convert into zbus interface type
fn iface_name() -> InterfaceName<'static> {
    InterfaceName::try_from(UPOWER_DEVICE_IFACE)
        .expect("UPOWER_DEVICE_IFACE is a valid interface name")
}

// Try setting up a UPower D-Bus listener for the given battery to check for plugin/plugout
// -> on error we have a fallback at the caller
pub fn try_spawn_listener(
    battery: Option<String>,
    charging_start: Option<Arc<ChargingStartHook>>,
    charging_stop: Option<Arc<ChargingStopHook>>,
) -> zbus::Result<()> {
    let conn = Connection::system()?;

    let bat_name = battery
        .clone()
        .unwrap_or_else(|| DEFAULT_BATTERY.to_string());
    let device_path: ObjectPath<'static> =
        ObjectPath::try_from(format!("{}{}", UPOWER_DEVICE_PATH_PREFIX, bat_name))?.into_owned();

    let proxy = PropertiesProxy::builder(&conn)
        .destination(UPOWER_DEST)?
        .path(device_path)?
        .build()?;

    let initial_value = proxy.get(iface_name(), "State")?;
    let initial_raw = u32::try_from(&initial_value)?;
    let mut last_status = map_upower_state(initial_raw);

    thread::spawn(move || {
        let signals = match proxy.receive_properties_changed() {
            Ok(s) => s,
            Err(err) => {
                eprintln!("UPower: failed to subscribe to PropertiesChanged: {}", err);
                return;
            }
        };

        let bat_owned = battery;

        for signal in signals {
            let args = match signal.args() {
                Ok(a) => a,
                Err(err) => {
                    eprintln!("UPower: bad signal args: {}", err);
                    continue;
                }
            };

            if args.interface_name().as_str() != UPOWER_DEVICE_IFACE {
                continue;
            }

            let value = match proxy.get(iface_name(), "State") {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("UPower: read State failed: {}", err);
                    continue;
                }
            };
            let raw: u32 = match u32::try_from(&value) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let current = map_upower_state(raw);

            if matches!(current, ChargingStatus::Unknown) {
                continue;
            }
            if current == last_status {
                continue;
            }

            let was_plugged = last_status.is_plugged_in();
            let is_plugged = current.is_plugged_in();
            let bat = bat_owned.as_deref();

            if !was_plugged
                && is_plugged
                && let Some(hook) = &charging_start
            {
                let level = get_current_power(bat);
                fire_plugin_plugout_hook(level, &hook.base);
            } else if was_plugged
                && !is_plugged
                && let Some(hook) = &charging_stop
            {
                let level = get_current_power(bat);
                fire_plugin_plugout_hook(level, &hook.base);
            }

            last_status = current;
        }

        eprintln!("UPower: signal stream ended; charging hooks will no longer fire until restart");
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- UPowerState::from_raw ------------------------------------------

    #[test]
    fn upower_state_from_raw_unknown_0() {
        assert_eq!(UPowerState::from_raw(0), UPowerState::Unknown);
    }

    #[test]
    fn upower_state_from_raw_charging_1() {
        assert_eq!(UPowerState::from_raw(1), UPowerState::Charging);
    }

    #[test]
    fn upower_state_from_raw_discharging_2() {
        assert_eq!(UPowerState::from_raw(2), UPowerState::Discharging);
    }

    #[test]
    fn upower_state_from_raw_empty_3() {
        assert_eq!(UPowerState::from_raw(3), UPowerState::Empty);
    }

    #[test]
    fn upower_state_from_raw_fully_charged_4() {
        assert_eq!(UPowerState::from_raw(4), UPowerState::FullyCharged);
    }

    #[test]
    fn upower_state_from_raw_pending_charge_5() {
        assert_eq!(UPowerState::from_raw(5), UPowerState::PendingCharge);
    }

    #[test]
    fn upower_state_from_raw_pending_discharge_6() {
        assert_eq!(UPowerState::from_raw(6), UPowerState::PendingDischarge);
    }

    #[test]
    fn upower_state_from_raw_seven_is_unknown() {
        assert_eq!(UPowerState::from_raw(7), UPowerState::Unknown);
    }

    #[test]
    fn upower_state_from_raw_large_is_unknown() {
        assert_eq!(UPowerState::from_raw(u32::MAX), UPowerState::Unknown);
    }

    // ---- map_upower_state -----------------------------------------------

    #[test]
    fn map_upower_state_unknown_0() {
        assert_eq!(map_upower_state(0), ChargingStatus::Unknown);
    }

    #[test]
    fn map_upower_state_charging_1() {
        assert_eq!(map_upower_state(1), ChargingStatus::Charging);
    }

    #[test]
    fn map_upower_state_discharging_2() {
        assert_eq!(map_upower_state(2), ChargingStatus::Discharging);
    }

    #[test]
    fn map_upower_state_empty_3_is_unknown() {
        assert_eq!(map_upower_state(3), ChargingStatus::Unknown);
    }

    #[test]
    fn map_upower_state_fully_charged_4() {
        assert_eq!(map_upower_state(4), ChargingStatus::Full);
    }

    #[test]
    fn map_upower_state_pending_charge_5_is_charging() {
        assert_eq!(map_upower_state(5), ChargingStatus::Charging);
    }

    #[test]
    fn map_upower_state_pending_discharge_6_is_discharging() {
        assert_eq!(map_upower_state(6), ChargingStatus::Discharging);
    }

    #[test]
    fn map_upower_state_out_of_range_is_unknown() {
        assert_eq!(map_upower_state(7), ChargingStatus::Unknown);
        assert_eq!(map_upower_state(u32::MAX), ChargingStatus::Unknown);
    }
}

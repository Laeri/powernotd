use crate::notification::ChargingHook;
use crate::{fire_plugin_plugout_hook, get_current_power, ChargingStatus, DEFAULT_BATTERY};
use std::sync::Arc;
use std::thread;
use zbus::blocking::{fdo::PropertiesProxy, Connection};
use zbus::names::InterfaceName;
use zbus::zvariant::ObjectPath;

const UPOWER_DEST: &str = "org.freedesktop.UPower";
const UPOWER_DEVICE_IFACE: &str = "org.freedesktop.UPower.Device";
const UPOWER_DEVICE_PATH_PREFIX: &str = "/org/freedesktop/UPower/devices/battery_";

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
    charging_start: Option<Arc<ChargingHook>>,
    charging_stop: Option<Arc<ChargingHook>>,
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

            if !was_plugged && is_plugged {
                if let Some(hook) = &charging_start {
                    let level = get_current_power(bat);
                    fire_plugin_plugout_hook(level, hook);
                }
            } else if was_plugged && !is_plugged {
                if let Some(hook) = &charging_stop {
                    let level = get_current_power(bat);
                    fire_plugin_plugout_hook(level, hook);
                }
            }

            last_status = current;
        }

        eprintln!("UPower: signal stream ended; charging hooks will no longer fire until restart");
    });

    Ok(())
}

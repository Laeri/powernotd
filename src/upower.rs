use crate::notification::ChargingHook;
use crate::{fire_charging_hook, get_current_power, ChargingStatus, DEFAULT_BATTERY};
use std::sync::Arc;
use std::thread;
use zbus::blocking::{fdo::PropertiesProxy, Connection};
use zbus::names::InterfaceName;
use zbus::zvariant::ObjectPath;

const UPOWER_DEST: &str = "org.freedesktop.UPower";
const UPOWER_DEVICE_IFACE: &str = "org.freedesktop.UPower.Device";

fn map_upower_state(state: u32) -> ChargingStatus {
    // UPower state enum:
    // 0 Unknown, 1 Charging, 2 Discharging, 3 Empty,
    // 4 FullyCharged, 5 PendingCharge, 6 PendingDischarge
    match state {
        1 | 5 => ChargingStatus::Charging,
        2 | 6 => ChargingStatus::Discharging,
        4 => ChargingStatus::Full,
        _ => ChargingStatus::Unknown,
    }
}

fn iface_name() -> InterfaceName<'static> {
    InterfaceName::try_from(UPOWER_DEVICE_IFACE)
        .expect("UPOWER_DEVICE_IFACE is a valid interface name")
}

/// Try to set up a UPower D-Bus listener for the given battery. On success, spawn a
/// watcher thread and return Ok(()). On failure (no system bus, no UPower service,
/// device path not found), return Err so the caller can fall back to polling.
pub fn try_spawn_listener(
    battery: Option<String>,
    charging_start: Option<Arc<ChargingHook>>,
    charging_stop: Option<Arc<ChargingHook>>,
) -> zbus::Result<()> {
    let conn = Connection::system()?;

    let bat_name = battery
        .clone()
        .unwrap_or_else(|| DEFAULT_BATTERY.to_string());
    let device_path: ObjectPath<'static> = ObjectPath::try_from(format!(
        "/org/freedesktop/UPower/devices/battery_{}",
        bat_name
    ))?
    .into_owned();

    let proxy = PropertiesProxy::builder(&conn)
        .destination(UPOWER_DEST)?
        .path(device_path)?
        .build()?;

    // Verify the device exists and read the initial State once. This will fail
    // if UPower isn't running or doesn't expose this battery, which the caller
    // uses as the signal to fall back to polling.
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

            // Re-read State on every relevant PropertiesChanged. Cheap, and robust
            // to whether State was carried in changed_properties or just invalidated.
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
                    fire_charging_hook(level, hook);
                }
            } else if was_plugged && !is_plugged {
                if let Some(hook) = &charging_stop {
                    let level = get_current_power(bat);
                    fire_charging_hook(level, hook);
                }
            }

            last_status = current;
        }

        eprintln!("UPower: signal stream ended; charging hooks will no longer fire until restart");
    });

    Ok(())
}

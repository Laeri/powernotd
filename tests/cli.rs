// Integration tests for the powernotd binary.
//
// The following flags are intentionally NOT covered here because they
// require host state that does not exist on CI:
//   -s / --status-level    reads /sys/class/power_supply/BAT0/capacity
//   -c / --charging-state  reads /sys/class/power_supply/BAT0/status
//   -n / --notify-now      opens a D-Bus session and talks to a notification daemon
// They are exercised manually on a real desktop before release.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn powernotd() -> Command {
    Command::cargo_bin("powernotd").expect("binary should build")
}

fn write_tmp(contents: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create tempfile");
    f.write_all(contents.as_bytes()).expect("write tempfile");
    f.flush().expect("flush tempfile");
    f
}

#[test]
fn help_prints_usage_and_exits_zero() {
    powernotd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: powernotd"));
}

#[test]
fn version_prints_crate_version() {
    powernotd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn list_thresholds_with_custom_config_prints_sorted() {
    let json = r#"{
      "notifications": [
        {"level": 80, "urgency": "Low"},
        {"level": 5,  "urgency": "Critical"},
        {"level": 50, "urgency": "Normal"}
      ],
      "full_notification": {"urgency": "Low", "enabled": false}
    }"#;
    let f = write_tmp(json);
    powernotd()
        .arg("-t")
        .arg("-f")
        .arg(f.path())
        .assert()
        .success()
        .stdout("5%, 50%, 80%\n");
}

#[test]
fn list_thresholds_malformed_config_falls_back_to_defaults() {
    let f = write_tmp("{ not json");
    powernotd()
        .arg("-t")
        .arg("-f")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("30%"));
}

#[test]
fn show_config_path_prints_a_path() {
    powernotd()
        .arg("-p")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn emit_default_config_prints_json_without_battery_files() {
    powernotd()
        .arg("--emit-default-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"notifications\""))
        .stdout(predicate::str::contains("\"full_notification\""))
        .stdout(predicate::str::contains("\"poll_interval_secs\": 60"));
}

#[test]
fn unknown_flag_fails() {
    powernotd()
        .arg("--no-such-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected"));
}

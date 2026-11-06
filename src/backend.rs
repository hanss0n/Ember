//! Talking to the `ember` CLI and systemd.

use crate::config::{Mode, Monitor};
use crate::paths;
use crate::status::Status;
use tokio::process::Command;

const TIMER_UNIT: &str = "ember.timer";

/// Run the CLI and return stdout, or stderr as the error.
async fn run_cli(args: &[&str]) -> Result<String, String> {
    let out = Command::new(paths::cli())
        .args(args)
        .output()
        .await
        .map_err(|e| format!("could not run ember: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

/// Current mode, schedule and warmth level.
pub async fn status() -> Result<Status, String> {
    let out = run_cli(&["status", "--json"]).await?;
    serde_json::from_str(out.trim()).map_err(|e| format!("bad status output: {e}"))
}

/// Apply the saved config to the monitors.
pub async fn apply() -> Result<(), String> {
    run_cli(&["apply"]).await.map(drop)
}

/// Switch mode (saves to config and applies).
pub async fn set_mode(mode: Mode) -> Result<(), String> {
    run_cli(&[mode.cli_arg()]).await.map(drop)
}

/// Show full warmth for `seconds`, then return to the scheduled level.
pub async fn preview(seconds: u32) -> Result<(), String> {
    run_cli(&["preview", &seconds.to_string()]).await.map(drop)
}

/// Detect DDC/CI monitors. The CLI also saves them to the config.
pub async fn detect_monitors() -> Result<Vec<Monitor>, String> {
    let out = run_cli(&["detect"]).await?;
    serde_json::from_str(out.trim()).map_err(|e| format!("bad detect output: {e}"))
}

/// Enable or disable the background systemd timer.
pub async fn set_timer(enabled: bool) -> Result<(), String> {
    run_cli(&["timer", if enabled { "on" } else { "off" }]).await.map(drop)
}

/// Whether the background systemd timer is enabled.
pub async fn timer_enabled() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-enabled", "--quiet", TIMER_UNIT])
        .status()
        .await
        .is_ok_and(|s| s.success())
}

/// Launch the settings app.
pub fn open_settings_app() {
    if let Err(e) = std::process::Command::new(paths::local_bin("cosmic-ember")).spawn() {
        eprintln!("could not launch cosmic-ember: {e}");
    }
}

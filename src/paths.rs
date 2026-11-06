//! Filesystem locations shared with the `ember` CLI.

use std::path::PathBuf;

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

/// `~/.config/ember/config.json`
pub fn config_file() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
        .join("ember")
        .join("config.json")
}

/// A program in `~/.local/bin` if installed there, otherwise looked up on `$PATH`.
/// The panel doesn't necessarily have `~/.local/bin` on its `PATH`.
pub fn local_bin(name: &str) -> PathBuf {
    let local = home().join(".local/bin").join(name);
    if local.exists() { local } else { PathBuf::from(name) }
}

/// The `ember` CLI.
pub fn cli() -> PathBuf {
    local_bin("ember")
}

//! Filesystem locations shared with the `nightlight` CLI.

use std::path::PathBuf;

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn env_dir(var: &str, fallback: impl FnOnce() -> PathBuf) -> PathBuf {
    std::env::var_os(var).map(PathBuf::from).unwrap_or_else(fallback)
}

/// `~/.config/nightlight/config.json`
pub fn config_file() -> PathBuf {
    env_dir("XDG_CONFIG_HOME", || home().join(".config"))
        .join("nightlight")
        .join("config.json")
}

/// `$XDG_RUNTIME_DIR/nightlight.state`, which records the level last applied by the CLI.
pub fn state_file() -> PathBuf {
    env_dir("XDG_RUNTIME_DIR", || PathBuf::from("/tmp")).join("nightlight.state")
}

/// A program in `~/.local/bin` if installed there, otherwise looked up on `$PATH`.
/// The panel doesn't necessarily have `~/.local/bin` on its `PATH`.
pub fn local_bin(name: &str) -> PathBuf {
    let local = home().join(".local/bin").join(name);
    if local.exists() { local } else { PathBuf::from(name) }
}

/// The `nightlight` CLI.
pub fn cli() -> PathBuf {
    local_bin("nightlight")
}

//! User configuration, stored as JSON and shared with the CLI.

use crate::paths;
use crate::schedule::{self, ClockTime};
use serde::{Deserialize, Serialize};

/// How the night light decides what to show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Follow the warm/cool schedule.
    #[default]
    Auto,
    /// Stay fully warm.
    Night,
    /// Stay at normal colors.
    Day,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Auto, Mode::Night, Mode::Day];

    /// The CLI subcommand that switches to this mode.
    pub fn cli_arg(self) -> &'static str {
        match self {
            Mode::Auto => "auto",
            Mode::Night => "night",
            Mode::Day => "day",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mode::Auto => "Automatic",
            Mode::Night => "Always warm",
            Mode::Day => "Off",
        }
    }
}

/// A DDC/CI-capable monitor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Monitor {
    pub bus: u32,
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub mode: Mode,
    /// Evening time ("HH:MM") when warming starts.
    pub warm_start: String,
    /// Morning time ("HH:MM") when cooling back to normal starts.
    pub cool_start: String,
    /// Length of each fade in minutes (0 = instant).
    pub fade_minutes: u32,
    /// Warmth at full night, 0–100.
    pub strength: u32,
    pub monitors: Vec<Monitor>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Auto,
            warm_start: "21:00".into(),
            cool_start: "06:00".into(),
            fade_minutes: 60,
            strength: 50,
            monitors: Vec::new(),
        }
    }
}

impl Config {
    /// Load the config, falling back to defaults if missing or unreadable.
    pub fn load() -> Self {
        std::fs::read_to_string(paths::config_file())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Atomically write the config.
    pub fn save(&self) -> std::io::Result<()> {
        let path = paths::config_file();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)? + "\n")?;
        std::fs::rename(tmp, path)
    }

    pub fn warm_time(&self) -> ClockTime {
        ClockTime::parse(&self.warm_start)
    }

    pub fn cool_time(&self) -> ClockTime {
        ClockTime::parse(&self.cool_start)
    }

    pub fn set_warm_time(&mut self, t: ClockTime) {
        self.warm_start = t.to_string();
    }

    pub fn set_cool_time(&mut self, t: ClockTime) {
        self.cool_start = t.to_string();
    }

    /// Level the monitors should be at right now, considering the mode.
    pub fn target_level(&self) -> u32 {
        match self.mode {
            Mode::Night => schedule::STEPS,
            Mode::Day => 0,
            Mode::Auto => schedule::level_now(self.warm_time(), self.cool_time(), self.fade_minutes),
        }
    }
}

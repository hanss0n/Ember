//! User configuration, stored as JSON and shared with the CLI.

use crate::clock::ClockTime;
use crate::paths;
use serde::{Deserialize, Serialize};

/// What the screen should do right now.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Follow the schedule.
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

/// Where the automatic schedule's times come from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Schedule {
    /// Fixed warm-up and cool-down times.
    #[default]
    Clock,
    /// Warm up at sunset, cool down at sunrise.
    Sun,
}

impl Schedule {
    pub const ALL: [Schedule; 2] = [Schedule::Clock, Schedule::Sun];

    pub fn label(self) -> &'static str {
        match self {
            Schedule::Clock => "Fixed times",
            Schedule::Sun => "Sunset to sunrise",
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
    pub schedule: Schedule,
    /// Fixed schedule: evening time ("HH:MM") when warming starts.
    pub warm_start: String,
    /// Fixed schedule: morning time ("HH:MM") when cooling starts.
    pub cool_start: String,
    /// Sun schedule location. `None` means "estimate from the time zone".
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
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
            schedule: Schedule::Clock,
            warm_start: "21:00".into(),
            cool_start: "06:00".into(),
            latitude: None,
            longitude: None,
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

    /// Set a manual location, or `None` to estimate it from the time zone.
    pub fn set_location(&mut self, location: Option<(f64, f64)>) {
        (self.latitude, self.longitude) = match location {
            Some((lat, lon)) => (Some(lat), Some(lon)),
            None => (None, None),
        };
    }
}

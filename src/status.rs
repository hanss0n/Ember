//! What the CLI reports about the current state, and how to show it.

use crate::config::{Mode, Schedule};
use serde::Deserialize;

pub const ICON_SUN: &str = "weather-clear-symbolic";
pub const ICON_MOON: &str = "weather-clear-night-symbolic";

/// Output of `ember status --json`.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Status {
    pub mode: Mode,
    pub schedule: Schedule,
    pub steps: u32,
    pub target_level: u32,
    /// Level last written to the monitors; `None` if unknown or mid-preview.
    pub applied_level: Option<u32>,
    /// Warm-up time in use today (sunset for the sun schedule).
    pub warm_start: String,
    /// Cool-down time in use today (sunrise for the sun schedule).
    pub cool_start: String,
    /// "clock", "sun", "clock-polar" or "clock-no-location".
    pub schedule_source: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    /// "config", "timezone", or `None` if no location is known.
    pub location_source: Option<String>,
}

impl Status {
    /// The level the screen is at (or heading to, if nothing was applied yet).
    pub fn level(&self) -> u32 {
        self.applied_level.unwrap_or(self.target_level)
    }

    pub fn is_warm(&self) -> bool {
        self.level() > 0
    }

    /// Moon while any warmth is applied, sun otherwise.
    pub fn icon(&self) -> &'static str {
        if self.is_warm() { ICON_MOON } else { ICON_SUN }
    }

    /// Short headline, e.g. "Warm until 07:20" or "Fading (60% warm)".
    pub fn headline(&self) -> String {
        let level = self.level();
        match self.mode {
            Mode::Night => "Always warm".into(),
            Mode::Day => "Off".into(),
            Mode::Auto if level == 0 => format!("Normal until {}", self.warm_start),
            Mode::Auto if level >= self.steps => format!("Warm until {}", self.cool_start),
            Mode::Auto => format!("Fading ({}% warm)", level * 100 / self.steps.max(1)),
        }
    }

    /// Explains today's schedule, e.g. "Sunset 15:43 · Sunrise 07:20".
    pub fn schedule_summary(&self) -> String {
        match self.schedule_source.as_str() {
            "sun" => format!("Sunset {} · Sunrise {}", self.warm_start, self.cool_start),
            "clock-polar" => format!(
                "The sun doesn't set or rise today, so using {} – {}",
                self.warm_start, self.cool_start
            ),
            "clock-no-location" => format!(
                "Location unknown, so using {} – {}",
                self.warm_start, self.cool_start
            ),
            _ => format!("{} – {}", self.warm_start, self.cool_start),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(mode: Mode, level: u32) -> Status {
        Status {
            mode,
            steps: 10,
            target_level: level,
            warm_start: "21:00".into(),
            cool_start: "06:00".into(),
            schedule_source: "clock".into(),
            ..Default::default()
        }
    }

    #[test]
    fn headlines() {
        assert_eq!(status(Mode::Auto, 0).headline(), "Normal until 21:00");
        assert_eq!(status(Mode::Auto, 10).headline(), "Warm until 06:00");
        assert_eq!(status(Mode::Auto, 6).headline(), "Fading (60% warm)");
        assert_eq!(status(Mode::Night, 10).headline(), "Always warm");
    }

    #[test]
    fn applied_level_wins_over_target() {
        let mut s = status(Mode::Auto, 10);
        s.applied_level = Some(0);
        assert_eq!(s.icon(), ICON_SUN);
    }

    #[test]
    fn parses_cli_json() {
        let json = r#"{"mode": "auto", "schedule": "sun", "steps": 10, "target_level": 10,
            "applied_level": null, "warm_start": "15:43", "cool_start": "07:20",
            "schedule_source": "sun", "latitude": 59.33, "longitude": 18.05,
            "location_source": "timezone"}"#;
        let s: Status = serde_json::from_str(json).unwrap();
        assert_eq!(s.schedule, Schedule::Sun);
        assert_eq!(s.schedule_summary(), "Sunset 15:43 · Sunrise 07:20");
    }
}

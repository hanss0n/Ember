//! Human-readable status and icon choice.

use crate::config::{Config, Mode};
use crate::schedule::STEPS;

pub const ICON_SUN: &str = "weather-clear-symbolic";
pub const ICON_MOON: &str = "weather-clear-night-symbolic";

/// Moon while any warmth is applied, sun otherwise.
pub fn icon_for(level: u32) -> &'static str {
    if level > 0 { ICON_MOON } else { ICON_SUN }
}

/// Short status line, e.g. "Warm until 06:00" or "Fading (60% warm)".
pub fn describe(cfg: &Config, level: u32) -> String {
    match cfg.mode {
        Mode::Night => "Always warm".into(),
        Mode::Day => "Off".into(),
        Mode::Auto if level == 0 => format!("Normal until {}", cfg.warm_start),
        Mode::Auto if level >= STEPS => format!("Warm until {}", cfg.cool_start),
        Mode::Auto => format!("Fading ({}% warm)", level * 100 / STEPS),
    }
}

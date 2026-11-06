//! Schedule math. Mirrors `scheduled_level` in the CLI.
//!
//! Uses the computer's local clock, so daylight saving time is handled by the system.

use chrono::{Local, NaiveTime, Timelike};
use std::fmt;

/// Number of discrete warmth levels; 0 = normal, `STEPS` = fully warm.
pub const STEPS: u32 = 10;

const MINUTES_PER_DAY: i32 = 24 * 60;

/// A time of day with minute precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockTime {
    pub hour: u32,
    pub minute: u32,
}

impl ClockTime {
    pub fn new(hour: u32, minute: u32) -> Self {
        Self { hour: hour.min(23), minute: minute.min(59) }
    }

    /// Parse "HH:MM"; invalid parts become 0.
    pub fn parse(s: &str) -> Self {
        let mut parts = s.split(':').map(|p| p.trim().parse::<u32>().unwrap_or(0));
        Self::new(parts.next().unwrap_or(0), parts.next().unwrap_or(0))
    }

    fn minutes(self) -> i32 {
        (self.hour * 60 + self.minute) as i32
    }

    fn from_naive(t: NaiveTime) -> Self {
        Self::new(t.hour(), t.minute())
    }
}

impl fmt::Display for ClockTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

/// Warmth level at time `now` for a schedule.
pub fn level_at(now: ClockTime, warm: ClockTime, cool: ClockTime, fade_minutes: u32) -> u32 {
    let since_warm = (now.minutes() - warm.minutes()).rem_euclid(MINUTES_PER_DAY) as f64;
    let since_cool = (now.minutes() - cool.minutes()).rem_euclid(MINUTES_PER_DAY) as f64;
    let fade = fade_minutes as f64;

    let warmth = if since_cool < since_warm {
        // Most recent event was the morning cool-down.
        if fade == 0.0 { 0.0 } else { 1.0 - (since_cool / fade).min(1.0) }
    } else {
        // Most recent event was the evening warm-up.
        if fade == 0.0 { 1.0 } else { (since_warm / fade).min(1.0) }
    };
    (warmth * STEPS as f64).round() as u32
}

/// Warmth level right now, by the local system clock.
pub fn level_now(warm: ClockTime, cool: ClockTime, fade_minutes: u32) -> u32 {
    level_at(ClockTime::from_naive(Local::now().time()), warm, cool, fade_minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> ClockTime {
        ClockTime::parse(s)
    }

    #[test]
    fn evening_fade() {
        let (w, c) = (t("21:00"), t("06:00"));
        assert_eq!(level_at(t("20:59"), w, c, 60), 0);
        assert_eq!(level_at(t("21:30"), w, c, 60), 5);
        assert_eq!(level_at(t("22:00"), w, c, 60), STEPS);
        assert_eq!(level_at(t("03:00"), w, c, 60), STEPS);
    }

    #[test]
    fn morning_fade() {
        let (w, c) = (t("21:00"), t("06:00"));
        assert_eq!(level_at(t("06:30"), w, c, 60), 5);
        assert_eq!(level_at(t("07:00"), w, c, 60), 0);
        assert_eq!(level_at(t("12:00"), w, c, 60), 0);
    }

    #[test]
    fn instant_switch() {
        let (w, c) = (t("21:00"), t("06:00"));
        assert_eq!(level_at(t("21:00"), w, c, 0), STEPS);
        assert_eq!(level_at(t("06:00"), w, c, 0), 0);
    }

    #[test]
    fn parse_and_display() {
        assert_eq!(t("7:5").to_string(), "07:05");
        assert_eq!(t("99:99").to_string(), "23:59");
        assert_eq!(t("junk").to_string(), "00:00");
    }
}

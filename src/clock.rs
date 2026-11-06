//! Time-of-day values as stored in the config ("HH:MM").

use std::fmt;

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
}

impl fmt::Display for ClockTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

#[cfg(test)]
mod tests {
    use super::ClockTime;

    #[test]
    fn parses_and_pads() {
        assert_eq!(ClockTime::parse("7:5").to_string(), "07:05");
        assert_eq!(ClockTime::parse("21:00"), ClockTime::new(21, 0));
    }

    #[test]
    fn clamps_out_of_range() {
        assert_eq!(ClockTime::parse("99:99").to_string(), "23:59");
    }

    #[test]
    fn invalid_becomes_midnight() {
        assert_eq!(ClockTime::parse("junk").to_string(), "00:00");
    }
}

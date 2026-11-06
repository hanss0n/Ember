//! Text fields for latitude and longitude, validated before saving.

use cosmic_ember::Config;

/// What the user has typed. Empty fields mean "estimate from the time zone".
#[derive(Clone, Debug, Default)]
pub struct LocationInput {
    pub latitude: String,
    pub longitude: String,
}

impl LocationInput {
    pub fn from_config(config: &Config) -> Self {
        let fmt = |v: Option<f64>| v.map(|v| format!("{v:.4}")).unwrap_or_default();
        Self { latitude: fmt(config.latitude), longitude: fmt(config.longitude) }
    }

    /// Both coordinates, if both fields are valid.
    pub fn parse(&self) -> Option<(f64, f64)> {
        Some((parse_coord(&self.latitude, 90.0)?, parse_coord(&self.longitude, 180.0)?))
    }

    pub fn latitude_valid(&self) -> bool {
        self.latitude.is_empty() || parse_coord(&self.latitude, 90.0).is_some()
    }

    pub fn longitude_valid(&self) -> bool {
        self.longitude.is_empty() || parse_coord(&self.longitude, 180.0).is_some()
    }

    pub fn is_manual(&self) -> bool {
        !self.latitude.is_empty() || !self.longitude.is_empty()
    }
}

/// Parse a decimal degree within ±limit. Accepts a comma as the decimal separator.
fn parse_coord(text: &str, limit: f64) -> Option<f64> {
    let value: f64 = text.trim().replace(',', ".").parse().ok()?;
    (value.is_finite() && value.abs() <= limit).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(lat: &str, lon: &str) -> LocationInput {
        LocationInput { latitude: lat.into(), longitude: lon.into() }
    }

    #[test]
    fn parses_valid_coordinates() {
        assert_eq!(input("59.33", "18,07").parse(), Some((59.33, 18.07)));
        assert_eq!(input("-33.9", "151.2").parse(), Some((-33.9, 151.2)));
    }

    #[test]
    fn rejects_out_of_range_or_partial() {
        assert_eq!(input("91", "0").parse(), None);
        assert_eq!(input("0", "181").parse(), None);
        assert_eq!(input("59.3", "").parse(), None);
        assert!(!input("abc", "").latitude_valid());
    }
}

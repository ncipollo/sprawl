//! The chart item's string tokens: series kind, plot kind, card size,
//! scrub trigger, and weekday names.

use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError};

/// How a chart's x values are interpreted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartSeries {
    Time,
    Weekday,
    Numeric,
}

/// One way of drawing the samples; several may stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartPlot {
    Line,
    Bar,
    Points,
}

/// The card's footprint in the content grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// When the pointer scrubber shows the sample nearest the pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartScrub {
    #[default]
    Off,
    Hover,
    Press,
}

/// A day of the week, as named by a weekday series' x values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// What a weekday x value may look like, for error messages.
pub const WEEKDAY_EXPECTED: &str = "a day name like monday or mon (any case)";

impl<'de> Deserialize<'de> for ChartSeries {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_token(
            deserializer,
            "series",
            "time, weekday, or numeric",
            parse_series,
        )
    }
}

impl<'de> Deserialize<'de> for ChartPlot {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_token(deserializer, "plot", "line, bar, or points", parse_plot)
    }
}

impl<'de> Deserialize<'de> for ChartSize {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_token(deserializer, "size", "small, medium, or large", parse_size)
    }
}

impl<'de> Deserialize<'de> for ChartScrub {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_token(deserializer, "scrub", "off, hover, or press", parse_scrub)
    }
}

/// Reads a string and parses it, naming the field and the valid tokens when
/// the text is not one of them.
fn deserialize_token<'de, D: Deserializer<'de>, T>(
    deserializer: D,
    what: &str,
    expected: &str,
    parse: fn(&str) -> Option<T>,
) -> Result<T, D::Error> {
    let text = String::deserialize(deserializer)?;
    parse(&text)
        .ok_or_else(|| DeError::custom(format!("unknown {what} {text:?}: expected {expected}")))
}

fn parse_series(text: &str) -> Option<ChartSeries> {
    match text {
        "time" => Some(ChartSeries::Time),
        "weekday" => Some(ChartSeries::Weekday),
        "numeric" => Some(ChartSeries::Numeric),
        _ => None,
    }
}

fn parse_plot(text: &str) -> Option<ChartPlot> {
    match text {
        "line" => Some(ChartPlot::Line),
        "bar" => Some(ChartPlot::Bar),
        "points" => Some(ChartPlot::Points),
        _ => None,
    }
}

fn parse_size(text: &str) -> Option<ChartSize> {
    match text {
        "small" => Some(ChartSize::Small),
        "medium" => Some(ChartSize::Medium),
        "large" => Some(ChartSize::Large),
        _ => None,
    }
}

fn parse_scrub(text: &str) -> Option<ChartScrub> {
    match text {
        "off" => Some(ChartScrub::Off),
        "hover" => Some(ChartScrub::Hover),
        "press" => Some(ChartScrub::Press),
        _ => None,
    }
}

/// Parses a full or three-letter day name in any case.
pub fn parse_weekday(text: &str) -> Option<Weekday> {
    match text.to_ascii_lowercase().as_str() {
        "monday" | "mon" => Some(Weekday::Monday),
        "tuesday" | "tue" => Some(Weekday::Tuesday),
        "wednesday" | "wed" => Some(Weekday::Wednesday),
        "thursday" | "thu" => Some(Weekday::Thursday),
        "friday" | "fri" => Some(Weekday::Friday),
        "saturday" | "sat" => Some(Weekday::Saturday),
        "sunday" | "sun" => Some(Weekday::Sunday),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse<T: for<'de> Deserialize<'de>>(token: &str) -> Result<T, serde_json::Error> {
        serde_json::from_str(&format!("\"{token}\""))
    }

    #[test]
    fn every_series_token_parses() {
        for (token, expected) in [
            ("time", ChartSeries::Time),
            ("weekday", ChartSeries::Weekday),
            ("numeric", ChartSeries::Numeric),
        ] {
            let series: ChartSeries = parse(token).expect("should parse");
            assert_eq!(series, expected);
        }
    }

    #[test]
    fn every_plot_token_parses() {
        for (token, expected) in [
            ("line", ChartPlot::Line),
            ("bar", ChartPlot::Bar),
            ("points", ChartPlot::Points),
        ] {
            let plot: ChartPlot = parse(token).expect("should parse");
            assert_eq!(plot, expected);
        }
    }

    #[test]
    fn every_size_token_parses() {
        for (token, expected) in [
            ("small", ChartSize::Small),
            ("medium", ChartSize::Medium),
            ("large", ChartSize::Large),
        ] {
            let size: ChartSize = parse(token).expect("should parse");
            assert_eq!(size, expected);
        }
    }

    #[test]
    fn size_defaults_to_medium() {
        assert_eq!(ChartSize::default(), ChartSize::Medium);
    }

    #[test]
    fn every_scrub_token_parses() {
        for (token, expected) in [
            ("off", ChartScrub::Off),
            ("hover", ChartScrub::Hover),
            ("press", ChartScrub::Press),
        ] {
            let scrub: ChartScrub = parse(token).expect("should parse");
            assert_eq!(scrub, expected);
        }
    }

    #[test]
    fn scrub_defaults_to_off() {
        assert_eq!(ChartScrub::default(), ChartScrub::Off);
    }

    #[test]
    fn every_weekday_name_and_abbreviation_parses_in_any_case() {
        for (names, expected) in [
            (["Monday", "monday", "Mon", "MON"], Weekday::Monday),
            (["Tuesday", "tuesday", "Tue", "TUE"], Weekday::Tuesday),
            (["Wednesday", "wednesday", "Wed", "WED"], Weekday::Wednesday),
            (["Thursday", "thursday", "Thu", "THU"], Weekday::Thursday),
            (["Friday", "friday", "Fri", "FRI"], Weekday::Friday),
            (["Saturday", "saturday", "Sat", "SAT"], Weekday::Saturday),
            (["Sunday", "sunday", "Sun", "SUN"], Weekday::Sunday),
        ] {
            for name in names {
                assert_eq!(parse_weekday(name), Some(expected), "{name}");
            }
        }
    }

    #[test]
    fn an_unknown_weekday_does_not_parse() {
        for bad in ["someday", "m", "", "mondays"] {
            assert_eq!(parse_weekday(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn malformed_tokens_are_rejected_with_the_offending_text() {
        let series = parse::<ChartSeries>("pie").expect_err("should fail");
        let plot = parse::<ChartPlot>("area").expect_err("should fail");
        let size = parse::<ChartSize>("huge").expect_err("should fail");

        assert!(
            series.to_string().contains("unknown series \"pie\""),
            "{series}"
        );
        assert!(plot.to_string().contains("unknown plot \"area\""), "{plot}");
        assert!(size.to_string().contains("unknown size \"huge\""), "{size}");
    }
}

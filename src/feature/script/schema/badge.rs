//! Badge labels and their colours.

use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError};

/// A coloured-dot label on a tile.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct BadgeItem {
    pub label: String,
    pub color: BadgeColor,
}

/// A badge colour: a named palette token or a raw `#RRGGBB` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeColor {
    Success,
    Warning,
    Danger,
    Neutral,
    Hex(u32),
}

impl<'de> Deserialize<'de> for BadgeColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        parse_color(&text).ok_or_else(|| {
            DeError::custom(format!(
                "unknown color {text:?}: expected success, warning, danger, \
                 neutral, or #RRGGBB"
            ))
        })
    }
}

fn parse_color(text: &str) -> Option<BadgeColor> {
    match text {
        "success" => Some(BadgeColor::Success),
        "warning" => Some(BadgeColor::Warning),
        "danger" => Some(BadgeColor::Danger),
        "neutral" => Some(BadgeColor::Neutral),
        _ => parse_hex(text),
    }
}

/// Parses a strict `#RRGGBB` colour.
fn parse_hex(text: &str) -> Option<BadgeColor> {
    let digits = text.strip_prefix('#')?;
    if digits.len() != 6 {
        return None;
    }
    u32::from_str_radix(digits, 16).ok().map(BadgeColor::Hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_named_color_token_parses() {
        for (name, expected) in [
            ("success", BadgeColor::Success),
            ("warning", BadgeColor::Warning),
            ("danger", BadgeColor::Danger),
            ("neutral", BadgeColor::Neutral),
        ] {
            let json = format!("\"{name}\"");
            let color: BadgeColor = serde_json::from_str(&json).expect("should parse");
            assert_eq!(color, expected);
        }
    }

    #[test]
    fn a_hex_color_parses_to_its_value() {
        let color: BadgeColor = serde_json::from_str("\"#4ec9b0\"").expect("should parse");

        assert_eq!(color, BadgeColor::Hex(0x4ec9b0));
    }

    #[test]
    fn malformed_colors_are_rejected() {
        for bad in ["\"#xyzxyz\"", "\"#fff\"", "\"blue\"", "\"4ec9b0\""] {
            assert!(serde_json::from_str::<BadgeColor>(bad).is_err(), "{bad}");
        }
    }
}

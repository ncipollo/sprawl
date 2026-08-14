//! The shape a config script's final value must have. Pure serde: the ui
//! layer maps these into gpui components.

use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError};

/// Everything a script provides for one section: its sidebar title and the
/// items shown in the content pane.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SectionConfig {
    pub title: String,
    pub items: Vec<SectionItem>,
}

/// One item in a section, discriminated by its `type` field.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SectionItem {
    Tile(TileItem),
}

/// The data behind a tile card.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TileItem {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub badges: Vec<BadgeItem>,
    #[serde(default)]
    pub url: Option<String>,
}

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
    fn a_full_section_deserializes() {
        let json = r#"{
            "title": "My PRs",
            "items": [{
                "type": "tile",
                "title": "Fix the bug",
                "subtitle": "o/r #7",
                "badges": [{"label": "Approved", "color": "success"}],
                "url": "https://github.com/o/r/pull/7"
            }]
        }"#;

        let config: SectionConfig = serde_json::from_str(json).expect("should parse");

        assert_eq!(config.title, "My PRs");
        let SectionItem::Tile(tile) = &config.items[0];
        assert_eq!(tile.title, "Fix the bug");
        assert_eq!(tile.subtitle, "o/r #7");
        assert_eq!(tile.badges[0].label, "Approved");
        assert_eq!(tile.badges[0].color, BadgeColor::Success);
        assert_eq!(tile.url.as_deref(), Some("https://github.com/o/r/pull/7"));
    }

    #[test]
    fn subtitle_badges_and_url_are_optional() {
        let json = r#"{"title": "T", "items": [{"type": "tile", "title": "only"}]}"#;

        let config: SectionConfig = serde_json::from_str(json).expect("should parse");

        let SectionItem::Tile(tile) = &config.items[0];
        assert_eq!(tile.subtitle, "");
        assert!(tile.badges.is_empty());
        assert_eq!(tile.url, None);
    }

    #[test]
    fn an_unknown_item_type_is_rejected() {
        let json = r#"{"title": "T", "items": [{"type": "chart", "title": "x"}]}"#;

        assert!(serde_json::from_str::<SectionConfig>(json).is_err());
    }

    #[test]
    fn a_tile_missing_its_title_is_rejected() {
        let json = r#"{"title": "T", "items": [{"type": "tile"}]}"#;

        assert!(serde_json::from_str::<SectionConfig>(json).is_err());
    }

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

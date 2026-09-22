//! The tile card item.

use crate::feature::script::schema::badge::BadgeItem;
use serde::Deserialize;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtitle_badges_and_url_are_optional() {
        let json = r#"{"title": "only"}"#;

        let tile: TileItem = serde_json::from_str(json).expect("should parse");

        assert_eq!(tile.subtitle, "");
        assert!(tile.badges.is_empty());
        assert_eq!(tile.url, None);
    }

    #[test]
    fn a_tile_missing_its_title_is_rejected() {
        let json = r#"{"subtitle": "no title"}"#;

        assert!(serde_json::from_str::<TileItem>(json).is_err());
    }
}

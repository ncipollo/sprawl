//! The shape a config script's final value must have. Pure serde: the ui
//! layer maps these into gpui components.

pub mod badge;
pub mod chart;
pub mod group;
pub mod tile;

pub use badge::{BadgeColor, BadgeItem};
pub use chart::{
    ChartItem, ChartPlot, ChartSamples, ChartScrub, ChartScrubTrigger, ChartSeries, ChartSize,
    Weekday,
};
pub use group::{GroupItem, LeafItem};
pub use tile::TileItem;

use serde::Deserialize;

/// Everything a script provides for one section: its sidebar title and the
/// items shown in the content pane.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SectionConfig {
    pub title: String,
    pub items: Vec<SectionItem>,
}

/// One item in a section, discriminated by its `type` field.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SectionItem {
    Tile(TileItem),
    Group(GroupItem),
    Chart(ChartItem),
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
        let SectionItem::Tile(tile) = &config.items[0] else {
            panic!("expected a tile")
        };
        assert_eq!(tile.title, "Fix the bug");
        assert_eq!(tile.subtitle, "o/r #7");
        assert_eq!(tile.badges[0].label, "Approved");
        assert_eq!(tile.badges[0].color, BadgeColor::Success);
        assert_eq!(tile.url.as_deref(), Some("https://github.com/o/r/pull/7"));
    }

    #[test]
    fn a_chart_item_is_a_section_item() {
        let json = r#"{
            "title": "Stats",
            "items": [{"type": "chart", "title": "Load", "series": "numeric", "samples": [[1, 2]]}]
        }"#;

        let config: SectionConfig = serde_json::from_str(json).expect("should parse");

        assert!(matches!(config.items[0], SectionItem::Chart(_)));
    }

    #[test]
    fn an_unknown_item_type_is_rejected() {
        let json = r#"{"title": "T", "items": [{"type": "widget", "title": "x"}]}"#;

        assert!(serde_json::from_str::<SectionConfig>(json).is_err());
    }
}

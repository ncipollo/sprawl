//! A labelled container of leaf items. Nesting is one level deep by design:
//! `LeafItem` rejects a group so the ui never has to recurse.

use crate::feature::script::schema::{SectionItem, TileItem};
use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError};

/// The data behind a group container.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GroupItem {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub items: Vec<LeafItem>,
}

/// An item allowed inside a group: any section item except another group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeafItem {
    Tile(TileItem),
}

impl<'de> Deserialize<'de> for LeafItem {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match SectionItem::deserialize(deserializer)? {
            SectionItem::Tile(tile) => Ok(LeafItem::Tile(tile)),
            SectionItem::Group(_) => Err(DeError::custom(
                "a group cannot contain another group: nest only tiles inside items",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<SectionItem, serde_json::Error> {
        serde_json::from_str(json)
    }

    fn expect_group(item: SectionItem) -> GroupItem {
        let SectionItem::Group(group) = item else {
            panic!("expected a group")
        };
        group
    }

    #[test]
    fn a_group_with_title_and_tiles_deserializes() {
        let json = r#"{
            "type": "group",
            "title": "Review",
            "items": [{"type": "tile", "title": "x"}, {"type": "tile", "title": "y"}]
        }"#;

        let group = expect_group(parse(json).expect("should parse"));

        assert_eq!(group.title.as_deref(), Some("Review"));
        assert_eq!(group.items.len(), 2);
        let LeafItem::Tile(tile) = &group.items[0];
        assert_eq!(tile.title, "x");
    }

    #[test]
    fn a_group_without_a_title_deserializes() {
        let json = r#"{"type": "group", "items": [{"type": "tile", "title": "x"}]}"#;

        let group = expect_group(parse(json).expect("should parse"));

        assert_eq!(group.title, None);
        assert_eq!(group.items.len(), 1);
    }

    #[test]
    fn a_group_without_items_is_empty() {
        let json = r#"{"type": "group", "title": "Nothing yet"}"#;

        let group = expect_group(parse(json).expect("should parse"));

        assert!(group.items.is_empty());
    }

    #[test]
    fn a_nested_group_is_rejected_with_a_descriptive_error() {
        let json = r#"{
            "type": "group",
            "items": [{"type": "group", "items": [{"type": "tile", "title": "x"}]}]
        }"#;

        let error = parse(json).expect_err("nested groups should fail");

        assert!(
            error.to_string().contains("cannot contain another group"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn a_non_tile_leaf_type_is_rejected() {
        let json = r#"{"type": "group", "items": [{"type": "chart", "title": "x"}]}"#;

        assert!(parse(json).is_err());
    }
}

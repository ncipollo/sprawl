//! Pure mapping from a script's section items to ui components. Kept
//! separate from the view so it's testable without gpui.

use crate::feature::script::schema::{
    BadgeColor, BadgeItem, ChartItem, GroupItem, LeafItem, SectionItem, TileItem,
};
use crate::ui::colors;
use crate::ui::components::chart_card::ChartCard;
use crate::ui::components::group::{Group, GroupChild};
use crate::ui::components::tile::{Tile, TileBadge};
use crate::ui::section_pane::chart;
use gpui::{App, IntoElement, RenderOnce, Window};

/// The rendered form of one section item. An enum rather than `AnyElement`
/// so the grid gets one child type while tests can still inspect the result.
#[derive(IntoElement)]
pub enum ItemElement {
    Tile(Tile),
    Group(Group),
    Chart(ChartCard),
}

impl RenderOnce for ItemElement {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        match self {
            ItemElement::Tile(tile) => tile.into_any_element(),
            ItemElement::Group(group) => group.into_any_element(),
            ItemElement::Chart(card) => card.into_any_element(),
        }
    }
}

/// Renders one section item into its component.
pub fn render_item(index: usize, item: &SectionItem) -> ItemElement {
    match item {
        SectionItem::Tile(data) => ItemElement::Tile(tile(index, data)),
        SectionItem::Group(data) => ItemElement::Group(group(index, data)),
        SectionItem::Chart(data) => ItemElement::Chart(chart_card(index, data)),
    }
}

// Nested items reuse `("tile", child_index)` and `("chart", child_index)`:
// gpui scopes element ids by their ancestor path, so the group's id keeps
// them distinct from top-level items.
fn group(index: usize, data: &GroupItem) -> Group {
    let mut group = Group::new(("group", index));
    if let Some(title) = data.title.clone() {
        group = group.title(title);
    }
    for (child_index, leaf_item) in data.items.iter().enumerate() {
        group = group.child(leaf(child_index, leaf_item));
    }
    group
}

fn leaf(index: usize, leaf: &LeafItem) -> GroupChild {
    match leaf {
        LeafItem::Tile(data) => GroupChild::Tile(tile(index, data)),
        LeafItem::Chart(data) => GroupChild::Chart(chart_card(index, data)),
    }
}

/// Builds the graph now, so a builder failure lands as text in the card
/// rather than a panic in the view.
fn chart_card(index: usize, data: &ChartItem) -> ChartCard {
    let graph = chart::build_graph(data).map_err(|error| error.to_string());
    ChartCard::new(("chart", index), data.title.clone(), graph).size(chart::card_size(data.size))
}

fn tile(index: usize, data: &TileItem) -> Tile {
    let mut tile = Tile::new(("tile", index), data.title.clone()).subtitle(data.subtitle.clone());
    for badge_item in &data.badges {
        tile = tile.badge(badge(badge_item));
    }
    if let Some(url) = data.url.clone() {
        tile = tile.on_click(move |_event, _window, cx| cx.open_url(&url));
    }
    tile
}

/// Maps a script badge to a tile badge.
pub fn badge(badge: &BadgeItem) -> TileBadge {
    TileBadge::new(color_value(badge.color), badge.label.clone())
}

/// Resolves a badge colour to a raw hex value: named tokens map to the
/// palette, hex values pass through.
fn color_value(color: BadgeColor) -> u32 {
    match color {
        BadgeColor::Success => colors::SUCCESS,
        BadgeColor::Warning => colors::WARNING,
        BadgeColor::Danger => colors::DANGER,
        BadgeColor::Neutral => colors::NEUTRAL,
        BadgeColor::Hex(value) => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::script::schema::{ChartSamples, ChartSize};
    use crate::ui::components::chart_card::ChartCardSize;
    use gpui::SharedString;

    fn tile_item(title: &str) -> TileItem {
        TileItem {
            title: title.to_string(),
            subtitle: String::new(),
            badges: Vec::new(),
            url: None,
        }
    }

    fn chart_item(title: &str) -> ChartItem {
        ChartItem {
            title: title.to_string(),
            plot: Vec::new(),
            samples: ChartSamples::Numeric(vec![(0.0, 1.0), (1.0, 2.0)]),
            y_range: None,
            size: ChartSize::Small,
        }
    }

    fn group_item(title: Option<&str>, tiles: &[&str]) -> SectionItem {
        SectionItem::Group(GroupItem {
            title: title.map(str::to_string),
            items: tiles
                .iter()
                .map(|title| LeafItem::Tile(tile_item(title)))
                .collect(),
        })
    }

    fn expect_group(element: ItemElement) -> Group {
        let ItemElement::Group(group) = element else {
            panic!("expected a group")
        };
        group
    }

    #[test]
    fn named_tokens_map_to_the_palette() {
        assert_eq!(color_value(BadgeColor::Success), colors::SUCCESS);
        assert_eq!(color_value(BadgeColor::Warning), colors::WARNING);
        assert_eq!(color_value(BadgeColor::Danger), colors::DANGER);
        assert_eq!(color_value(BadgeColor::Neutral), colors::NEUTRAL);
    }

    #[test]
    fn hex_colors_pass_through_unchanged() {
        assert_eq!(color_value(BadgeColor::Hex(0x123456)), 0x123456);
    }

    #[test]
    fn a_badge_keeps_its_label_and_resolved_color() {
        let item = BadgeItem {
            label: "Approved".to_string(),
            color: BadgeColor::Success,
        };

        let badge = badge(&item);

        assert_eq!(badge.label(), "Approved");
        assert_eq!(badge.color(), colors::SUCCESS);
    }

    #[test]
    fn a_tile_item_renders_as_a_tile() {
        let item = SectionItem::Tile(tile_item("hello"));

        assert!(matches!(render_item(0, &item), ItemElement::Tile(_)));
    }

    #[test]
    fn a_group_item_renders_its_title_and_tiles() {
        let item = group_item(Some("Review"), &["a", "b"]);

        let group = expect_group(render_item(1, &item));

        assert_eq!(group.title_text().map(SharedString::as_ref), Some("Review"));
        assert_eq!(group.children().len(), 2);
    }

    #[test]
    fn a_title_less_group_has_no_header() {
        let item = group_item(None, &["a"]);

        let group = expect_group(render_item(0, &item));

        assert!(group.title_text().is_none());
        assert_eq!(group.children().len(), 1);
    }

    #[test]
    fn a_chart_item_renders_as_a_chart_card() {
        let item = SectionItem::Chart(chart_item("Load"));

        let ItemElement::Chart(card) = render_item(0, &item) else {
            panic!("expected a chart card")
        };

        assert_eq!(card.title_text(), "Load");
        assert_eq!(card.card_size(), ChartCardSize::Small);
        assert!(card.error().is_none());
    }

    #[test]
    fn a_group_renders_charts_beside_tiles() {
        let item = SectionItem::Group(GroupItem {
            title: None,
            items: vec![
                LeafItem::Tile(tile_item("a")),
                LeafItem::Chart(chart_item("Load")),
            ],
        });

        let group = expect_group(render_item(0, &item));

        assert_eq!(group.children().len(), 2);
        assert!(matches!(group.children()[1], GroupChild::Chart(_)));
    }
}

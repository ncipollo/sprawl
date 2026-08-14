//! Pure mapping from a script's section items to ui components. Kept
//! separate from the view so it's testable without gpui.

use crate::feature::script::schema::{BadgeColor, BadgeItem, SectionItem, TileItem};
use crate::ui::colors;
use crate::ui::components::tile::{Tile, TileBadge};

/// Renders one section item into its component.
pub fn render_item(index: usize, item: &SectionItem) -> Tile {
    match item {
        SectionItem::Tile(data) => tile(index, data),
    }
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
}

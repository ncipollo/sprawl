//! A reusable, clickable card used to show a single item in a grid.

use crate::ui::colors;
use gpui::{
    App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
    rgb,
};

/// The fixed tile width the content grid wraps against.
pub const TILE_WIDTH: f32 = 300.0;
const DOT_SIZE: f32 = 8.0;

/// A short status marker shown along the bottom of a tile: a coloured dot
/// and a label. Colours are hex values matching the rest of the ui layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileBadge {
    color: u32,
    label: SharedString,
}

impl TileBadge {
    pub fn new(color: u32, label: impl Into<SharedString>) -> Self {
        Self {
            color,
            label: label.into(),
        }
    }

    pub fn color(&self) -> u32 {
        self.color
    }

    pub fn label(&self) -> &SharedString {
        &self.label
    }

    fn render(self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(div().size(px(DOT_SIZE)).rounded_full().bg(rgb(self.color)))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors::SECONDARY_TEXT))
                    .child(self.label),
            )
    }
}

/// A tile's click handler.
type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A clickable card. Built fluently, then rendered by the grid that owns it.
#[derive(IntoElement)]
pub struct Tile {
    id: ElementId,
    title: SharedString,
    subtitle: SharedString,
    badges: Vec<TileBadge>,
    on_click: Option<ClickHandler>,
}

impl Tile {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: SharedString::default(),
            badges: Vec::new(),
            on_click: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn badge(mut self, badge: TileBadge) -> Self {
        self.badges.push(badge);
        self
    }

    pub fn badges(&self) -> &[TileBadge] {
        &self.badges
    }

    /// Makes the tile clickable, which also gives it a pointer cursor.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    fn title_row(title: SharedString) -> impl IntoElement {
        div()
            .text_sm()
            .text_color(rgb(colors::PRIMARY_TEXT))
            .line_clamp(2)
            .child(title)
    }

    fn subtitle_row(subtitle: SharedString) -> impl IntoElement {
        div()
            .text_xs()
            .text_color(rgb(colors::SECONDARY_TEXT))
            .truncate()
            .child(subtitle)
    }

    fn badge_row(badges: Vec<TileBadge>) -> impl IntoElement {
        div()
            .flex()
            .flex_wrap()
            .gap_2()
            .children(badges.into_iter().map(TileBadge::render))
    }
}

impl RenderOnce for Tile {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let Tile {
            id,
            title,
            subtitle,
            badges,
            on_click,
        } = self;
        div()
            .id(id)
            .w(px(TILE_WIDTH))
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .rounded_md()
            .bg(rgb(colors::SIDEBAR_BACKGROUND))
            .border_1()
            .border_color(rgb(colors::BORDER))
            .hover(|tile| tile.bg(rgb(colors::ROW_HOVER_BACKGROUND)))
            .when(on_click.is_some(), |tile| tile.cursor_pointer())
            .when_some(on_click, |tile, handler| tile.on_click(handler))
            .child(Self::title_row(title))
            .child(Self::subtitle_row(subtitle))
            .child(Self::badge_row(badges))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_with_no_badges() {
        let tile = Tile::new("tile", "title");

        assert!(tile.badges().is_empty());
    }

    #[test]
    fn badge_keeps_the_order_it_was_added_in() {
        let tile = Tile::new("tile", "title")
            .badge(TileBadge::new(colors::SUCCESS, "first"))
            .badge(TileBadge::new(colors::DANGER, "second"));

        assert_eq!(tile.badges()[0].label(), "first");
        assert_eq!(tile.badges()[1].label(), "second");
    }
}

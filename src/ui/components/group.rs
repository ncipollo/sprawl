//! A bordered container that clusters related tiles under an optional title.

use crate::ui::colors;
use crate::ui::components::tile::Tile;
use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, rgb};

/// A full-width outline around a nested wrapping grid of tiles. Built
/// fluently, then rendered by the grid that owns it.
#[derive(IntoElement)]
pub struct Group {
    id: ElementId,
    title: Option<SharedString>,
    children: Vec<Tile>,
}

impl Group {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            children: Vec::new(),
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn child(mut self, tile: Tile) -> Self {
        self.children.push(tile);
        self
    }

    pub fn title_text(&self) -> Option<&SharedString> {
        self.title.as_ref()
    }

    pub fn tiles(&self) -> &[Tile] {
        &self.children
    }

    fn header_row(title: SharedString) -> impl IntoElement {
        div()
            .text_sm()
            .text_color(rgb(colors::PRIMARY_TEXT))
            .truncate()
            .child(title)
    }

    fn grid(children: Vec<Tile>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap_3()
            .children(children)
    }
}

impl RenderOnce for Group {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let Group {
            id,
            title,
            children,
        } = self;
        div()
            .id(id)
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(rgb(colors::BORDER))
            .when_some(title, |group, title| group.child(Self::header_row(title)))
            .child(Self::grid(children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_without_a_title_or_tiles() {
        let group = Group::new("group");

        assert!(group.title_text().is_none());
        assert!(group.tiles().is_empty());
    }

    #[test]
    fn title_is_recorded() {
        let group = Group::new("group").title("Review");

        assert_eq!(group.title_text().map(SharedString::as_ref), Some("Review"));
    }

    #[test]
    fn child_keeps_insertion_order() {
        let group = Group::new("group")
            .child(Tile::new(("tile", 0usize), "first"))
            .child(Tile::new(("tile", 1usize), "second"));

        assert_eq!(group.tiles().len(), 2);
    }
}

//! A bordered container that clusters related tiles and charts under an
//! optional title.

use crate::ui::colors;
use crate::ui::components::chart_card::ChartCard;
use crate::ui::components::tile::Tile;
use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, rgb};

/// An item a group can hold: anything but another group.
#[derive(IntoElement)]
pub enum GroupChild {
    Tile(Tile),
    Chart(ChartCard),
}

impl RenderOnce for GroupChild {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        match self {
            GroupChild::Tile(tile) => tile.into_any_element(),
            GroupChild::Chart(card) => card.into_any_element(),
        }
    }
}

impl From<Tile> for GroupChild {
    fn from(tile: Tile) -> Self {
        GroupChild::Tile(tile)
    }
}

impl From<ChartCard> for GroupChild {
    fn from(card: ChartCard) -> Self {
        GroupChild::Chart(card)
    }
}

/// A full-width outline around a nested wrapping grid of items. Built
/// fluently, then rendered by the grid that owns it.
#[derive(IntoElement)]
pub struct Group {
    id: ElementId,
    title: Option<SharedString>,
    children: Vec<GroupChild>,
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

    pub fn child(mut self, child: impl Into<GroupChild>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn title_text(&self) -> Option<&SharedString> {
        self.title.as_ref()
    }

    pub fn children(&self) -> &[GroupChild] {
        &self.children
    }

    fn header_row(title: SharedString) -> impl IntoElement {
        div()
            .text_sm()
            .text_color(rgb(colors::PRIMARY_TEXT))
            .truncate()
            .child(title)
    }

    fn grid(children: Vec<GroupChild>) -> impl IntoElement {
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
    use gpui_charts::GraphBuilder;

    #[test]
    fn new_starts_without_a_title_or_children() {
        let group = Group::new("group");

        assert!(group.title_text().is_none());
        assert!(group.children().is_empty());
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

        assert_eq!(group.children().len(), 2);
    }

    #[test]
    fn child_accepts_tiles_and_charts() {
        let card = ChartCard::new(("chart", 0usize), "Load", Ok(GraphBuilder::new().build()));

        let group = Group::new("group")
            .child(Tile::new(("tile", 0usize), "first"))
            .child(card);

        assert!(matches!(group.children()[0], GroupChild::Tile(_)));
        assert!(matches!(group.children()[1], GroupChild::Chart(_)));
    }
}

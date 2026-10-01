//! A fixed-size card that hosts a graph under a title.

use crate::ui::colors;
use crate::ui::components::tile::TILE_WIDTH;
use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
    rgb,
};
use gpui_charts::Graph;

/// The section grid's `gap_3`, so a medium card lines up with two tiles.
const GRID_GAP: f32 = 12.0;

/// Room above the graph for the scrub ring and value label, which gpui-charts
/// paints above the plot area's top edge and the body's clip would cut off.
const SCRUB_HEADROOM: f32 = 18.0;

/// How much of the grid a chart card takes up.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartCardSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ChartCardSize {
    /// The fixed width in pixels; `None` spans the full grid width.
    pub fn width(self) -> Option<f32> {
        match self {
            ChartCardSize::Small => Some(TILE_WIDTH),
            ChartCardSize::Medium => Some(2.0 * TILE_WIDTH + GRID_GAP),
            ChartCardSize::Large => None,
        }
    }

    pub fn height(self) -> f32 {
        match self {
            ChartCardSize::Small => 160.0,
            ChartCardSize::Medium => 200.0,
            ChartCardSize::Large => 240.0,
        }
    }
}

/// A titled graph card. A failed graph shows its error text in place of the
/// plot, so a bad series never takes the app down.
#[derive(IntoElement)]
pub struct ChartCard {
    id: ElementId,
    title: SharedString,
    size: ChartCardSize,
    graph: Result<Graph, String>,
}

impl ChartCard {
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        graph: Result<Graph, String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            size: ChartCardSize::default(),
            graph,
        }
    }

    pub fn size(mut self, size: ChartCardSize) -> Self {
        self.size = size;
        self
    }

    pub fn title_text(&self) -> &SharedString {
        &self.title
    }

    pub fn card_size(&self) -> ChartCardSize {
        self.size
    }

    /// The graph's build error, when there is one.
    pub fn error(&self) -> Option<&str> {
        self.graph.as_ref().err().map(String::as_str)
    }

    fn title_row(title: SharedString) -> impl IntoElement {
        div()
            .text_sm()
            .text_color(rgb(colors::PRIMARY_TEXT))
            .truncate()
            .child(title)
    }

    // The graph fills whatever size it is given, so it needs a bounded box:
    // `flex_1` inside the fixed-height card, clipped so axis labels can't
    // push the card taller.
    fn body(graph: Result<Graph, String>) -> AnyElement {
        match graph {
            Ok(graph) => div()
                .flex_1()
                .min_h_0()
                .pt(px(SCRUB_HEADROOM))
                .overflow_hidden()
                .child(graph)
                .into_any_element(),
            Err(message) => div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .text_color(rgb(colors::DANGER))
                .child(SharedString::from(message))
                .into_any_element(),
        }
    }
}

impl RenderOnce for ChartCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let ChartCard {
            id,
            title,
            size,
            graph,
        } = self;
        div()
            .id(id)
            .h(px(size.height()))
            .map(|card| match size.width() {
                Some(width) => card.w(px(width)),
                None => card.w_full(),
            })
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .rounded_md()
            .bg(rgb(colors::SIDEBAR_BACKGROUND))
            .border_1()
            .border_color(rgb(colors::BORDER))
            .child(Self::title_row(title))
            .child(Self::body(graph))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_charts::{DEFAULT_PLOT_PADDING, GraphBuilder};

    fn graph() -> Result<Graph, String> {
        Ok(GraphBuilder::new().build())
    }

    #[test]
    fn new_defaults_to_a_medium_card() {
        let card = ChartCard::new("chart", "Load", graph());

        assert_eq!(card.title_text(), "Load");
        assert_eq!(card.card_size(), ChartCardSize::Medium);
        assert!(card.error().is_none());
    }

    #[test]
    fn size_is_recorded() {
        let card = ChartCard::new("chart", "Load", graph()).size(ChartCardSize::Large);

        assert_eq!(card.card_size(), ChartCardSize::Large);
    }

    #[test]
    fn large_spans_the_full_width() {
        assert!(ChartCardSize::Large.width().is_none());
    }

    #[test]
    fn smaller_sizes_have_fixed_widths() {
        assert_eq!(ChartCardSize::Small.width(), Some(TILE_WIDTH));
        assert_eq!(
            ChartCardSize::Medium.width(),
            Some(2.0 * TILE_WIDTH + GRID_GAP)
        );
    }

    #[test]
    fn heights_grow_with_size() {
        assert!(ChartCardSize::Small.height() < ChartCardSize::Medium.height());
        assert!(ChartCardSize::Medium.height() < ChartCardSize::Large.height());
    }

    #[test]
    fn headroom_fits_the_scrub_ring_and_value_label() {
        // 7px ring radius + 2px gap + 11px label font (~14px line height).
        let needed = 7.0 + 2.0 + 14.0;
        let available = f32::from(DEFAULT_PLOT_PADDING) + SCRUB_HEADROOM;

        assert!(available >= needed);
    }

    #[test]
    fn a_failed_graph_keeps_its_message() {
        let card = ChartCard::new("chart", "Load", Err("boom".to_string()));

        assert_eq!(card.error(), Some("boom"));
    }
}

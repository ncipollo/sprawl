//! Pure mapping from a chart item to a gpui-charts graph. Kept separate from
//! the view so the enum mapping and builder calls are testable without gpui.

use crate::feature::script::schema::{
    BadgeColor, ChartItem, ChartPlot, ChartSamples, ChartScrub, ChartScrubTrigger, ChartSize,
    Weekday,
};
use crate::ui::colors;
use crate::ui::components::chart_card::ChartCardSize;
use crate::ui::section_pane::item::color_value;
use gpui::{Rgba, rgb};
use gpui_charts::{
    ChartError, Graph, NumericSeriesBuilder, PlotKind, ScrubOptions, TimeSeriesBuilder, ValueAxis,
    Weekday as ChartWeekday, WeekdayBuilder,
};

/// Builds the graph for `item` with the builder matching its series kind.
pub fn build_graph(item: &ChartItem) -> Result<Graph, ChartError> {
    match &item.samples {
        ChartSamples::Time(samples) => time_graph(item, samples),
        ChartSamples::Weekday(samples) => weekday_graph(item, samples),
        ChartSamples::Numeric(samples) => numeric_graph(item, samples),
    }
}

pub fn plot_kind(plot: ChartPlot) -> PlotKind {
    match plot {
        ChartPlot::Line => PlotKind::Line,
        ChartPlot::Bar => PlotKind::Bar,
        ChartPlot::Points => PlotKind::Points,
    }
}

pub fn weekday(day: Weekday) -> ChartWeekday {
    match day {
        Weekday::Monday => ChartWeekday::Monday,
        Weekday::Tuesday => ChartWeekday::Tuesday,
        Weekday::Wednesday => ChartWeekday::Wednesday,
        Weekday::Thursday => ChartWeekday::Thursday,
        Weekday::Friday => ChartWeekday::Friday,
        Weekday::Saturday => ChartWeekday::Saturday,
        Weekday::Sunday => ChartWeekday::Sunday,
    }
}

pub fn card_size(size: ChartSize) -> ChartCardSize {
    match size {
        ChartSize::Small => ChartCardSize::Small,
        ChartSize::Medium => ChartCardSize::Medium,
        ChartSize::Large => ChartCardSize::Large,
    }
}

/// The gpui-charts scrubber for a chart's scrub config; an `Off` trigger
/// leaves the library's disabled default.
pub fn scrub_options(scrub: &ChartScrub) -> ScrubOptions {
    let base = match scrub.trigger {
        ChartScrubTrigger::Off => ScrubOptions::default(),
        ChartScrubTrigger::Hover => ScrubOptions::hover(),
        ChartScrubTrigger::Press => ScrubOptions::press_and_hold(),
    };
    let options = base
        .with_show_value(scrub.value)
        .with_guide(scrub.guide)
        .with_point(scrub.point);
    apply_scrub_colors(options, scrub)
}

fn apply_scrub_colors(mut options: ScrubOptions, scrub: &ChartScrub) -> ScrubOptions {
    if let Some(color) = scrub.guide_color {
        options = options.with_guide_color(scrub_color(color));
    }
    if let Some(color) = scrub.point_color {
        options = options.with_point_color(scrub_color(color));
    }
    if let Some(color) = scrub.value_color {
        options = options.with_value_color(scrub_color(color));
    }
    options
}

fn scrub_color(color: BadgeColor) -> Rgba {
    rgb(color_value(color))
}

// The three builders share no trait, so each series kind gets its own
// near-identical helper. An empty `plot` leaves the builder's default.
fn time_graph(item: &ChartItem, samples: &[(i64, f64)]) -> Result<Graph, ChartError> {
    let mut builder = TimeSeriesBuilder::new()
        .samples(samples.iter().copied())
        .y_axis(y_axis(item))
        .color(series_color())
        .scrub(scrub_options(&item.scrub));
    for plot in &item.plot {
        builder = builder.plot_kind(plot_kind(*plot));
    }
    builder.build()
}

fn weekday_graph(item: &ChartItem, samples: &[(Weekday, f64)]) -> Result<Graph, ChartError> {
    let mut builder = WeekdayBuilder::new()
        .values(samples.iter().map(|(day, value)| (weekday(*day), *value)))
        .y_axis(y_axis(item))
        .color(series_color())
        .scrub(scrub_options(&item.scrub));
    for plot in &item.plot {
        builder = builder.plot_kind(plot_kind(*plot));
    }
    builder.build()
}

fn numeric_graph(item: &ChartItem, samples: &[(f64, f64)]) -> Result<Graph, ChartError> {
    let mut builder = NumericSeriesBuilder::new()
        .samples(samples.iter().copied())
        .y_axis(y_axis(item))
        .color(series_color())
        .scrub(scrub_options(&item.scrub));
    for plot in &item.plot {
        builder = builder.plot_kind(plot_kind(*plot));
    }
    builder.build()
}

fn y_axis(item: &ChartItem) -> ValueAxis {
    match item.y_range {
        Some((low, high)) => ValueAxis::default().range(low, high),
        None => ValueAxis::default(),
    }
}

fn series_color() -> Rgba {
    rgb(colors::CHART_SERIES)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart_item(
        samples: ChartSamples,
        plot: Vec<ChartPlot>,
        y_range: Option<(f64, f64)>,
    ) -> ChartItem {
        ChartItem {
            title: "Load".to_string(),
            plot,
            samples,
            y_range,
            size: ChartSize::Medium,
            scrub: ChartScrub::default(),
        }
    }

    #[test]
    fn every_plot_token_maps_to_its_gpui_charts_kind() {
        assert_eq!(plot_kind(ChartPlot::Line), PlotKind::Line);
        assert_eq!(plot_kind(ChartPlot::Bar), PlotKind::Bar);
        assert_eq!(plot_kind(ChartPlot::Points), PlotKind::Points);
    }

    #[test]
    fn every_weekday_maps_to_its_gpui_charts_day() {
        for (day, expected) in [
            (Weekday::Monday, ChartWeekday::Monday),
            (Weekday::Tuesday, ChartWeekday::Tuesday),
            (Weekday::Wednesday, ChartWeekday::Wednesday),
            (Weekday::Thursday, ChartWeekday::Thursday),
            (Weekday::Friday, ChartWeekday::Friday),
            (Weekday::Saturday, ChartWeekday::Saturday),
            (Weekday::Sunday, ChartWeekday::Sunday),
        ] {
            assert_eq!(weekday(day), expected);
        }
    }

    #[test]
    fn every_size_token_maps_to_a_card_size() {
        assert_eq!(card_size(ChartSize::Small), ChartCardSize::Small);
        assert_eq!(card_size(ChartSize::Medium), ChartCardSize::Medium);
        assert_eq!(card_size(ChartSize::Large), ChartCardSize::Large);
    }

    fn scrub_with(trigger: ChartScrubTrigger) -> ChartScrub {
        ChartScrub::from_trigger(trigger)
    }

    #[test]
    fn scrub_tokens_map_to_their_gpui_charts_options() {
        assert!(!scrub_options(&scrub_with(ChartScrubTrigger::Off)).enabled);
        assert_eq!(
            scrub_options(&scrub_with(ChartScrubTrigger::Hover)),
            ScrubOptions::hover()
        );
        assert_eq!(
            scrub_options(&scrub_with(ChartScrubTrigger::Press)),
            ScrubOptions::press_and_hold()
        );
    }

    #[test]
    fn hiding_the_ring_keeps_the_guide_and_value() {
        let scrub = ChartScrub {
            point: false,
            ..scrub_with(ChartScrubTrigger::Hover)
        };

        let options = scrub_options(&scrub);

        assert_eq!(options, ScrubOptions::hover().with_point(false));
        assert!(options.style.show_guide);
        assert!(options.show_value);
    }

    #[test]
    fn hiding_the_value_and_guide_maps_to_the_options() {
        let scrub = ChartScrub {
            value: false,
            guide: false,
            ..scrub_with(ChartScrubTrigger::Press)
        };

        let options = scrub_options(&scrub);

        assert!(!options.show_value);
        assert!(!options.style.show_guide);
        assert!(options.style.show_point);
    }

    #[test]
    fn scrub_colors_resolve_named_tokens_and_hex() {
        let scrub = ChartScrub {
            guide_color: Some(BadgeColor::Success),
            point_color: Some(BadgeColor::Hex(0x123456)),
            value_color: Some(BadgeColor::Danger),
            ..scrub_with(ChartScrubTrigger::Hover)
        };

        let style = scrub_options(&scrub).style;

        assert_eq!(style.guide_color, Some(rgb(colors::SUCCESS).into()));
        assert_eq!(style.point_color, Some(rgb(0x123456).into()));
        assert_eq!(style.value_color, Some(rgb(colors::DANGER).into()));
    }

    #[test]
    fn unset_scrub_colors_stay_unset() {
        let style = scrub_options(&scrub_with(ChartScrubTrigger::Hover)).style;

        assert_eq!(style.guide_color, None);
        assert_eq!(style.point_color, None);
        assert_eq!(style.value_color, None);
    }

    #[test]
    fn a_chart_without_scrub_builds_a_graph_that_is_not_scrubbable() {
        let item = chart_item(ChartSamples::Numeric(vec![(0.0, 1.0)]), Vec::new(), None);

        let graph = build_graph(&item).expect("should build");

        assert!(!graph.is_scrubbable());
    }

    #[test]
    fn a_scrub_token_makes_every_plot_scrubbable() {
        let item = ChartItem {
            scrub: scrub_with(ChartScrubTrigger::Hover),
            ..chart_item(
                ChartSamples::Time(vec![(1_700_000_000, 3.0), (1_700_086_400, 5.5)]),
                vec![ChartPlot::Bar, ChartPlot::Points],
                None,
            )
        };

        let graph = build_graph(&item).expect("should build");

        assert!(graph.is_scrubbable());
        assert!(
            graph
                .plots()
                .iter()
                .all(|plot| plot.scrub_options() == ScrubOptions::hover())
        );
    }

    #[test]
    fn a_time_series_builds_one_plot_per_requested_kind() {
        let item = chart_item(
            ChartSamples::Time(vec![(1_700_000_000, 3.0), (1_700_086_400, 5.5)]),
            vec![ChartPlot::Line, ChartPlot::Points],
            None,
        );

        let graph = build_graph(&item).expect("should build");

        assert_eq!(graph.plots().len(), 2);
    }

    #[test]
    fn a_weekday_series_builds_a_graph_with_the_default_plot() {
        let item = chart_item(
            ChartSamples::Weekday(vec![(Weekday::Monday, 1.0), (Weekday::Friday, 4.0)]),
            Vec::new(),
            None,
        );

        let graph = build_graph(&item).expect("should build");

        assert_eq!(graph.plots().len(), 1);
    }

    #[test]
    fn a_numeric_series_builds_a_graph() {
        let item = chart_item(
            ChartSamples::Numeric(vec![(0.0, 1.0), (2.5, 4.0)]),
            vec![ChartPlot::Bar],
            None,
        );

        let graph = build_graph(&item).expect("should build");

        assert_eq!(graph.plots().len(), 1);
    }

    #[test]
    fn an_empty_series_still_builds() {
        let item = chart_item(ChartSamples::Time(Vec::new()), Vec::new(), None);

        assert!(build_graph(&item).is_ok());
    }

    #[test]
    fn a_fixed_y_range_pins_the_axis_labels() {
        let item = chart_item(
            ChartSamples::Numeric(vec![(0.0, 3.0), (1.0, 7.0)]),
            Vec::new(),
            Some((0.0, 10.0)),
        );

        let graph = build_graph(&item).expect("should build");

        let labels = graph.axes().vertical().expect("y axis").labels();
        assert_eq!(labels.first().map(|label| label.text.as_ref()), Some("0"));
        assert_eq!(labels.last().map(|label| label.text.as_ref()), Some("10"));
    }
}

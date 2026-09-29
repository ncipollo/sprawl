//! The chart item: a titled data series drawn as a graph card.

pub mod sample;
pub mod token;

pub use sample::ChartSamples;
pub use token::{ChartPlot, ChartScrub, ChartSeries, ChartSize, Weekday};

use sample::RawSamples;
use serde::Deserialize;

/// The data behind a chart card. The series kind lives in the `samples`
/// variant, so the ui matches on it directly.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "RawChartItem")]
pub struct ChartItem {
    pub title: String,
    /// Empty means the series' default plot.
    pub plot: Vec<ChartPlot>,
    pub samples: ChartSamples,
    pub y_range: Option<(f64, f64)>,
    pub size: ChartSize,
    pub scrub: ChartScrub,
}

/// The item as written, before its samples are checked against the series.
#[derive(Deserialize)]
struct RawChartItem {
    title: String,
    series: ChartSeries,
    #[serde(default)]
    plot: Vec<ChartPlot>,
    samples: RawSamples,
    #[serde(default)]
    y_range: Option<(f64, f64)>,
    #[serde(default)]
    size: ChartSize,
    #[serde(default)]
    scrub: ChartScrub,
}

impl TryFrom<RawChartItem> for ChartItem {
    type Error = String;

    fn try_from(raw: RawChartItem) -> Result<Self, String> {
        Ok(ChartItem {
            title: raw.title,
            plot: raw.plot,
            samples: ChartSamples::convert(raw.series, raw.samples)?,
            y_range: checked_y_range(raw.y_range)?,
            size: raw.size,
            scrub: raw.scrub,
        })
    }
}

/// Rejects an empty or inverted range up front, where the message can name
/// the field, instead of letting it surface as a graph error at render time.
fn checked_y_range(range: Option<(f64, f64)>) -> Result<Option<(f64, f64)>, String> {
    match range {
        Some((low, high)) if low >= high => Err(format!(
            "y_range must be [low, high] with low < high, got [{low}, {high}]"
        )),
        range => Ok(range),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::script::schema::SectionItem;

    fn parse(json: &str) -> Result<SectionItem, serde_json::Error> {
        serde_json::from_str(json)
    }

    fn expect_chart(item: SectionItem) -> ChartItem {
        let SectionItem::Chart(chart) = item else {
            panic!("expected a chart")
        };
        chart
    }

    fn error_of(json: &str) -> String {
        parse(json).expect_err("should fail").to_string()
    }

    #[test]
    fn a_time_chart_deserializes_with_every_field() {
        let json = r#"{
            "type": "chart",
            "title": "Commits",
            "series": "time",
            "plot": ["line", "points"],
            "samples": [[1700000000, 3], [1700086400, 5.5]],
            "y_range": [0, 10],
            "size": "large",
            "scrub": "hover"
        }"#;

        let chart = expect_chart(parse(json).expect("should parse"));

        assert_eq!(chart.title, "Commits");
        assert_eq!(chart.plot, vec![ChartPlot::Line, ChartPlot::Points]);
        assert_eq!(
            chart.samples,
            ChartSamples::Time(vec![(1_700_000_000, 3.0), (1_700_086_400, 5.5)])
        );
        assert_eq!(chart.y_range, Some((0.0, 10.0)));
        assert_eq!(chart.size, ChartSize::Large);
        assert_eq!(chart.scrub, ChartScrub::Hover);
    }

    #[test]
    fn a_weekday_chart_deserializes_names_in_any_case() {
        let json = r#"{
            "type": "chart", "title": "Load", "series": "weekday",
            "samples": [["Monday", 1], ["tue", 2], ["WED", 3]]
        }"#;

        let chart = expect_chart(parse(json).expect("should parse"));

        assert_eq!(
            chart.samples,
            ChartSamples::Weekday(vec![
                (Weekday::Monday, 1.0),
                (Weekday::Tuesday, 2.0),
                (Weekday::Wednesday, 3.0),
            ])
        );
    }

    #[test]
    fn a_numeric_chart_deserializes() {
        let json = r#"{
            "type": "chart", "title": "Curve", "series": "numeric",
            "samples": [[0, 1], [2.5, 4]]
        }"#;

        let chart = expect_chart(parse(json).expect("should parse"));

        assert_eq!(
            chart.samples,
            ChartSamples::Numeric(vec![(0.0, 1.0), (2.5, 4.0)])
        );
    }

    #[test]
    fn plot_y_range_size_and_scrub_are_optional() {
        let json = r#"{"type": "chart", "title": "T", "series": "numeric", "samples": [[1, 1]]}"#;

        let chart = expect_chart(parse(json).expect("should parse"));

        assert!(chart.plot.is_empty());
        assert_eq!(chart.y_range, None);
        assert_eq!(chart.size, ChartSize::Medium);
        assert_eq!(chart.scrub, ChartScrub::Off);
    }

    #[test]
    fn an_empty_samples_array_is_allowed() {
        let json = r#"{"type": "chart", "title": "T", "series": "time", "samples": []}"#;

        let chart = expect_chart(parse(json).expect("should parse"));

        assert_eq!(chart.samples, ChartSamples::Time(Vec::new()));
    }

    #[test]
    fn a_chart_missing_samples_is_rejected() {
        let error = error_of(r#"{"type": "chart", "title": "T", "series": "time"}"#);

        assert!(error.contains("samples"), "{error}");
    }

    #[test]
    fn an_unknown_series_is_rejected_with_a_descriptive_error() {
        let error = error_of(r#"{"type": "chart", "title": "T", "series": "pie", "samples": []}"#);

        assert!(error.contains("unknown series \"pie\""), "{error}");
    }

    #[test]
    fn an_unknown_plot_is_rejected_with_a_descriptive_error() {
        let error = error_of(
            r#"{"type": "chart", "title": "T", "series": "time", "plot": ["area"], "samples": []}"#,
        );

        assert!(error.contains("unknown plot \"area\""), "{error}");
    }

    #[test]
    fn an_unknown_size_is_rejected_with_a_descriptive_error() {
        let error = error_of(
            r#"{"type": "chart", "title": "T", "series": "time", "size": "huge", "samples": []}"#,
        );

        assert!(error.contains("unknown size \"huge\""), "{error}");
    }

    #[test]
    fn an_unknown_scrub_is_rejected_with_a_descriptive_error() {
        let error = error_of(
            r#"{"type": "chart", "title": "T", "series": "time", "scrub": "tap", "samples": []}"#,
        );

        assert!(error.contains("unknown scrub \"tap\""), "{error}");
    }

    #[test]
    fn a_text_x_in_a_time_series_is_rejected_naming_the_sample() {
        let error = error_of(
            r#"{"type": "chart", "title": "T", "series": "time",
                "samples": [[1700000000, 1], ["yesterday", 2]]}"#,
        );

        assert!(error.contains("sample 1"), "{error}");
        assert!(error.contains("integer unix seconds"), "{error}");
    }

    #[test]
    fn an_inverted_y_range_is_rejected() {
        let error = error_of(
            r#"{"type": "chart", "title": "T", "series": "numeric", "samples": [], "y_range": [5, 5]}"#,
        );

        assert!(error.contains("y_range must be [low, high]"), "{error}");
    }
}

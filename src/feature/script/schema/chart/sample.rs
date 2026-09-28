//! The `[x, y]` pairs of a chart. Pairs are read raw first because the type
//! of `x` depends on the sibling `series` field, which serde cannot express
//! directly; `ChartSamples::convert` then checks every pair against it.

use crate::feature::script::schema::chart::token::{
    ChartSeries, WEEKDAY_EXPECTED, Weekday, parse_weekday,
};
use serde::Deserialize;
use std::fmt;

/// One value of a pair before the series kind is known.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged, expecting = "a number or a weekday name")]
pub(super) enum RawValue {
    Number(f64),
    Text(String),
}

impl fmt::Display for RawValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RawValue::Number(number) => write!(f, "{number}"),
            RawValue::Text(text) => write!(f, "{text:?}"),
        }
    }
}

/// Every pair as written by the script, any length, checked in `convert`.
pub(super) type RawSamples = Vec<Vec<RawValue>>;

/// The samples of a chart, typed by their series kind.
#[derive(Clone, Debug, PartialEq)]
pub enum ChartSamples {
    /// `(unix seconds, value)`.
    Time(Vec<(i64, f64)>),
    Weekday(Vec<(Weekday, f64)>),
    Numeric(Vec<(f64, f64)>),
}

impl ChartSamples {
    /// Checks every raw pair against `series`, naming the first bad sample.
    pub(super) fn convert(series: ChartSeries, raw: RawSamples) -> Result<Self, String> {
        match series {
            ChartSeries::Time => time_samples(raw).map(ChartSamples::Time),
            ChartSeries::Weekday => weekday_samples(raw).map(ChartSamples::Weekday),
            ChartSeries::Numeric => numeric_samples(raw).map(ChartSamples::Numeric),
        }
    }
}

fn time_samples(raw: RawSamples) -> Result<Vec<(i64, f64)>, String> {
    pairs(raw)
        .map(|pair| {
            let (index, x, y) = pair?;
            match x {
                RawValue::Number(seconds) if seconds.fract() == 0.0 => Ok((seconds as i64, y)),
                other => Err(format!(
                    "sample {index}: a time series needs integer unix seconds for x, got {other}"
                )),
            }
        })
        .collect()
}

fn weekday_samples(raw: RawSamples) -> Result<Vec<(Weekday, f64)>, String> {
    pairs(raw)
        .map(|pair| {
            let (index, x, y) = pair?;
            match x {
                RawValue::Text(name) => parse_weekday(&name).map(|day| (day, y)).ok_or_else(|| {
                    format!("sample {index}: unknown weekday {name:?}: expected {WEEKDAY_EXPECTED}")
                }),
                other => Err(format!(
                    "sample {index}: a weekday series needs a day name for x, got {other}"
                )),
            }
        })
        .collect()
}

fn numeric_samples(raw: RawSamples) -> Result<Vec<(f64, f64)>, String> {
    pairs(raw)
        .map(|pair| {
            let (index, x, y) = pair?;
            match x {
                RawValue::Number(x) => Ok((x, y)),
                other => Err(format!(
                    "sample {index}: a numeric series needs a number for x, got {other}"
                )),
            }
        })
        .collect()
}

/// Splits every raw sample into its index, x, and numeric y, rejecting pairs
/// of the wrong length or with a non-numeric y.
fn pairs(raw: RawSamples) -> impl Iterator<Item = Result<(usize, RawValue, f64), String>> {
    raw.into_iter().enumerate().map(|(index, sample)| {
        let [x, y] = <[RawValue; 2]>::try_from(sample).map_err(|sample| {
            format!(
                "sample {index}: expected an [x, y] pair, got {} values",
                sample.len()
            )
        })?;
        let RawValue::Number(y) = y else {
            return Err(format!("sample {index}: y must be a number, got {y}"));
        };
        Ok((index, x, y))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(json: &str) -> RawSamples {
        serde_json::from_str(json).expect("should parse")
    }

    #[test]
    fn an_x_y_pair_parses_as_a_raw_sample() {
        let samples = raw(r#"[[1, 2.5], ["mon", 3]]"#);

        assert_eq!(
            samples,
            vec![
                vec![RawValue::Number(1.0), RawValue::Number(2.5)],
                vec![RawValue::Text("mon".to_string()), RawValue::Number(3.0)],
            ]
        );
    }

    #[test]
    fn a_sample_with_the_wrong_arity_is_rejected_naming_the_sample() {
        let error = ChartSamples::convert(ChartSeries::Numeric, raw("[[1, 2], [1, 2, 3]]"))
            .expect_err("should fail");

        assert!(error.contains("sample 1"), "{error}");
        assert!(error.contains("got 3 values"), "{error}");
    }

    #[test]
    fn a_non_number_y_is_rejected_naming_the_sample() {
        let error = ChartSamples::convert(ChartSeries::Numeric, raw(r#"[[1, "two"]]"#))
            .expect_err("should fail");

        assert!(error.contains("sample 0: y must be a number"), "{error}");
    }

    #[test]
    fn time_samples_require_integer_x() {
        let error = ChartSamples::convert(ChartSeries::Time, raw("[[1700000000, 1], [1.5, 2]]"))
            .expect_err("should fail");

        assert!(error.contains("sample 1"), "{error}");
        assert!(error.contains("integer unix seconds"), "{error}");
    }

    #[test]
    fn time_samples_convert_to_seconds() {
        let samples = ChartSamples::convert(
            ChartSeries::Time,
            raw("[[1700000000, 3], [1700086400, 5.5]]"),
        )
        .expect("should convert");

        assert_eq!(
            samples,
            ChartSamples::Time(vec![(1_700_000_000, 3.0), (1_700_086_400, 5.5)])
        );
    }

    #[test]
    fn weekday_samples_accept_names_and_abbreviations() {
        let samples = ChartSamples::convert(
            ChartSeries::Weekday,
            raw(r#"[["Monday", 1], ["tue", 2], ["WED", 3]]"#),
        )
        .expect("should convert");

        assert_eq!(
            samples,
            ChartSamples::Weekday(vec![
                (Weekday::Monday, 1.0),
                (Weekday::Tuesday, 2.0),
                (Weekday::Wednesday, 3.0),
            ])
        );
    }

    #[test]
    fn weekday_samples_reject_numbers_and_unknown_names() {
        let number =
            ChartSamples::convert(ChartSeries::Weekday, raw("[[1, 2]]")).expect_err("should fail");
        let name =
            ChartSamples::convert(ChartSeries::Weekday, raw(r#"[["mon", 1], ["someday", 2]]"#))
                .expect_err("should fail");

        assert!(number.contains("sample 0"), "{number}");
        assert!(number.contains("needs a day name"), "{number}");
        assert!(
            name.contains("sample 1: unknown weekday \"someday\""),
            "{name}"
        );
    }

    #[test]
    fn numeric_samples_reject_text() {
        let error = ChartSamples::convert(ChartSeries::Numeric, raw(r#"[["one", 1]]"#))
            .expect_err("should fail");

        assert!(error.contains("sample 0"), "{error}");
        assert!(
            error.contains("needs a number for x, got \"one\""),
            "{error}"
        );
    }

    #[test]
    fn numeric_samples_keep_their_values() {
        let samples = ChartSamples::convert(ChartSeries::Numeric, raw("[[0, 1], [2.5, 4]]"))
            .expect("should convert");

        assert_eq!(samples, ChartSamples::Numeric(vec![(0.0, 1.0), (2.5, 4.0)]));
    }
}

//! The chart item's scrubber configuration: a trigger string or an object
//! that also styles the highlight.

use super::token::ChartScrubTrigger;
use crate::feature::script::schema::badge::BadgeColor;
use serde::Deserialize;
use serde::de::value::{MapAccessDeserializer, StrDeserializer};
use serde::de::{Deserializer, Error as DeError, IntoDeserializer, MapAccess, Visitor};
use std::fmt;

/// How the pointer scrubber behaves and looks. The defaults match what
/// gpui-charts draws when no style is set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChartScrub {
    pub trigger: ChartScrubTrigger,
    pub value: bool,
    pub guide: bool,
    pub guide_color: Option<BadgeColor>,
    pub point: bool,
    pub point_color: Option<BadgeColor>,
    pub value_color: Option<BadgeColor>,
}

impl Default for ChartScrub {
    fn default() -> Self {
        Self::from_trigger(ChartScrubTrigger::Off)
    }
}

impl ChartScrub {
    /// The string shorthand: a trigger with every style left at its default.
    pub fn from_trigger(trigger: ChartScrubTrigger) -> Self {
        Self {
            trigger,
            value: true,
            guide: true,
            guide_color: None,
            point: true,
            point_color: None,
            value_color: None,
        }
    }
}

/// The object form as written. An object means the script wants a scrubber,
/// so an omitted trigger is hover rather than off.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScrubObject {
    #[serde(default = "hover")]
    trigger: ChartScrubTrigger,
    #[serde(default = "enabled")]
    value: bool,
    #[serde(default = "enabled")]
    guide: bool,
    #[serde(default)]
    guide_color: Option<BadgeColor>,
    #[serde(default = "enabled")]
    point: bool,
    #[serde(default)]
    point_color: Option<BadgeColor>,
    #[serde(default)]
    value_color: Option<BadgeColor>,
}

fn hover() -> ChartScrubTrigger {
    ChartScrubTrigger::Hover
}

fn enabled() -> bool {
    true
}

impl From<RawScrubObject> for ChartScrub {
    fn from(raw: RawScrubObject) -> Self {
        Self {
            trigger: raw.trigger,
            value: raw.value,
            guide: raw.guide,
            guide_color: raw.guide_color,
            point: raw.point,
            point_color: raw.point_color,
            value_color: raw.value_color,
        }
    }
}

impl<'de> Deserialize<'de> for ChartScrub {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ScrubVisitor)
    }
}

// A visitor rather than an untagged enum, which would swallow the field name
// and token in its errors.
struct ScrubVisitor;

impl<'de> Visitor<'de> for ScrubVisitor {
    type Value = ChartScrub;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a scrub trigger (off, hover, or press) or an object of scrub options")
    }

    fn visit_str<E: DeError>(self, text: &str) -> Result<ChartScrub, E> {
        let deserializer: StrDeserializer<E> = text.into_deserializer();
        ChartScrubTrigger::deserialize(deserializer).map(ChartScrub::from_trigger)
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<ChartScrub, A::Error> {
        RawScrubObject::deserialize(MapAccessDeserializer::new(map)).map(ChartScrub::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<ChartScrub, serde_json::Error> {
        serde_json::from_str(json)
    }

    fn error_of(json: &str) -> String {
        parse(json).expect_err("should fail").to_string()
    }

    #[test]
    fn the_default_is_off_with_every_style_at_its_default() {
        let scrub = ChartScrub::default();

        assert_eq!(scrub.trigger, ChartScrubTrigger::Off);
        assert!(scrub.value && scrub.guide && scrub.point);
        assert_eq!(scrub.guide_color, None);
        assert_eq!(scrub.point_color, None);
        assert_eq!(scrub.value_color, None);
    }

    #[test]
    fn the_string_shorthand_sets_only_the_trigger() {
        for (token, trigger) in [
            ("off", ChartScrubTrigger::Off),
            ("hover", ChartScrubTrigger::Hover),
            ("press", ChartScrubTrigger::Press),
        ] {
            let scrub = parse(&format!("\"{token}\"")).expect("should parse");
            assert_eq!(scrub, ChartScrub::from_trigger(trigger), "{token}");
        }
    }

    #[test]
    fn an_object_with_every_field_parses() {
        let scrub = parse(
            r##"{
                "trigger": "press", "value": false, "guide": false,
                "guide_color": "#112233", "point": false,
                "point_color": "success", "value_color": "danger"
            }"##,
        )
        .expect("should parse");

        assert_eq!(
            scrub,
            ChartScrub {
                trigger: ChartScrubTrigger::Press,
                value: false,
                guide: false,
                guide_color: Some(BadgeColor::Hex(0x112233)),
                point: false,
                point_color: Some(BadgeColor::Success),
                value_color: Some(BadgeColor::Danger),
            }
        );
    }

    #[test]
    fn an_empty_object_hovers_with_default_styles() {
        let scrub = parse("{}").expect("should parse");

        assert_eq!(scrub, ChartScrub::from_trigger(ChartScrubTrigger::Hover));
    }

    #[test]
    fn hiding_the_ring_leaves_the_guide_and_value_on() {
        let scrub = parse(r#"{"trigger": "hover", "point": false}"#).expect("should parse");

        assert!(!scrub.point);
        assert!(scrub.guide);
        assert!(scrub.value);
    }

    #[test]
    fn an_unknown_shorthand_token_is_rejected_with_the_offending_text() {
        let error = error_of("\"tap\"");

        assert!(error.contains("unknown scrub \"tap\""), "{error}");
    }

    #[test]
    fn an_unknown_trigger_in_an_object_is_rejected() {
        let error = error_of(r#"{"trigger": "tap"}"#);

        assert!(error.contains("unknown scrub \"tap\""), "{error}");
    }

    #[test]
    fn an_unknown_field_is_rejected_naming_it() {
        let error = error_of(r#"{"ring": false}"#);

        assert!(error.contains("unknown field `ring`"), "{error}");
    }

    #[test]
    fn a_bad_color_is_rejected_naming_the_color() {
        let error = error_of(r#"{"guide_color": "blue"}"#);

        assert!(error.contains("unknown color \"blue\""), "{error}");
    }

    #[test]
    fn a_non_boolean_flag_is_rejected() {
        assert!(parse(r#"{"point": "no"}"#).is_err());
    }

    #[test]
    fn a_number_is_rejected() {
        assert!(parse("1").is_err());
    }
}

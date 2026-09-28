//! The `chart` topic: drawing a data series as a graph card.

pub fn render() -> String {
    "CHART ITEMS\n\
     A \"chart\" draws a data series as a graph in a fixed-size card. It\n\
     may appear in a section's items or inside a group's items.\n\n\
     \x20 {\n\
     \x20   \"type\": \"chart\",\n\
     \x20   \"title\": \"...\",             // required; one truncated line\n\
     \x20   \"series\": \"time\",           // required; time, weekday, or numeric\n\
     \x20   \"plot\": [\"line\", \"points\"], // optional; line, bar, points\n\
     \x20   \"samples\": [[x, y], ...],   // required; may be empty\n\
     \x20   \"y_range\": [0, 10],         // optional; [low, high], else fitted\n\
     \x20   \"size\": \"medium\"            // optional; small, medium, or large\n\
     \x20 }\n\n\
     SERIES\n\
     Each sample is an [x, y] pair and y is always a number. What x is\n\
     depends on series: time takes integer unix seconds, labelled as UTC\n\
     dates; weekday takes a day name, full or three-letter, in any case\n\
     (\"Monday\", \"mon\"); numeric takes any number.\n\n\
     PLOTS\n\
     plot lists how the samples are drawn, painted in the order given, so\n\
     [\"line\", \"points\"] puts markers over the line. When omitted, time\n\
     and numeric draw a line and weekday draws bars.\n\n\
     SIZE\n\
     small is one tile wide, medium is two tiles wide, and large spans\n\
     the full width of the section. Each has a fixed height.\n\n\
     ERRORS\n\
     An unknown series, plot, or size token fails the script, as does a\n\
     malformed sample; the error names the sample, e.g. \"sample 2: ...\".\n\n\
     See also: --info tiles, --info groups, --info scripts, --info example\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_documents_every_chart_field() {
        let page = render();

        for field in [
            "\"type\"",
            "\"title\"",
            "\"series\"",
            "\"plot\"",
            "\"samples\"",
            "\"y_range\"",
            "\"size\"",
        ] {
            assert!(page.contains(field), "missing {field}");
        }
    }

    #[test]
    fn page_documents_every_series_kind() {
        let page = render();

        for series in ["time", "weekday", "numeric"] {
            assert!(page.contains(series), "missing {series}");
        }
    }

    #[test]
    fn page_documents_every_plot_kind() {
        let page = render();

        for plot in ["line", "bar", "points"] {
            assert!(page.contains(plot), "missing {plot}");
        }
    }

    #[test]
    fn page_documents_every_size_token() {
        let page = render();

        for size in ["small", "medium", "large"] {
            assert!(page.contains(size), "missing {size}");
        }
    }

    #[test]
    fn page_explains_weekday_names() {
        let page = render();

        assert!(page.contains("\"Monday\""));
        assert!(page.contains("\"mon\""));
    }
}

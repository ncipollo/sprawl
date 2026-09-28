//! The `groups` topic: clustering tiles and charts inside a labelled
//! container.

pub fn render() -> String {
    "GROUP ITEMS\n\
     A group wraps related tiles and charts in a bordered box that\n\
     spans the full width of the section, with its own wrapping grid\n\
     inside. Use it to label a cluster of items without splitting them\n\
     into sections.\n\n\
     \x20 {\n\
     \x20   \"type\": \"group\",\n\
     \x20   \"title\": \"...\",  // optional; shown above the items\n\
     \x20   \"items\": [...]   // tiles or charts; may be empty\n\
     \x20 }\n\n\
     Groups nest one level deep: every entry in a group's items must\n\
     be a tile or a chart. A group inside a group fails the script with\n\
     \"a group cannot contain another group\".\n\n\
     See also: --info tiles, --info chart, --info scripts, --info example\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_documents_every_group_field() {
        let page = render();

        for field in ["\"type\"", "\"title\"", "\"items\""] {
            assert!(page.contains(field), "missing {field}");
        }
    }

    #[test]
    fn page_explains_nesting_is_rejected() {
        assert!(render().contains("cannot contain another group"));
    }
}

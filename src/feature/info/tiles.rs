//! The `tiles` topic: the item schema scripts return.

pub fn render() -> String {
    "TILE ITEMS\n\
     Every entry in a section's items array is an object with a type\n\
     field. A \"tile\" is a fixed-width card in a wrapping grid; a\n\
     \"group\" is a labelled box of tiles (see --info groups).\n\n\
     \x20 {\n\
     \x20   \"type\": \"tile\",\n\
     \x20   \"title\": \"...\",       // required; wraps to two lines\n\
     \x20   \"subtitle\": \"...\",    // optional; one truncated line\n\
     \x20   \"badges\": [...],      // optional; colored-dot labels\n\
     \x20   \"url\": \"https://...\"  // optional; opened on click\n\
     \x20 }\n\n\
     BADGES\n\
     Each badge is { label, color }. color accepts the named tokens\n\
     success, warning, danger, and neutral, which follow the app\n\
     palette, or a raw hex value like \"#4ec9b0\".\n\n\
     See also: --info groups, --info scripts, --info example\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_documents_every_tile_field() {
        let page = render();

        for field in [
            "\"type\"",
            "\"title\"",
            "\"subtitle\"",
            "\"badges\"",
            "\"url\"",
        ] {
            assert!(page.contains(field), "missing {field}");
        }
    }

    #[test]
    fn page_documents_every_color_token_and_hex() {
        let page = render();

        for color in ["success", "warning", "danger", "neutral", "#4ec9b0"] {
            assert!(page.contains(color), "missing {color}");
        }
    }
}

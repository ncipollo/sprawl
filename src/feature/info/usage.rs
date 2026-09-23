//! The `usage` topic: invoking sprawl and where its config lives.

pub fn render() -> String {
    "USAGE\n\
     Running sprawl with no arguments opens the app.\n\n\
     CONFIG SCRIPTS\n\
     sprawl reads JavaScript files from ~/.sprawl/default. Each *.js\n\
     file is one section: an entry in the left sidebar whose items fill\n\
     the right pane when selected. Sections are ordered alphabetically\n\
     by file name.\n\n\
     On first launch, when ~/.sprawl does not exist, sprawl creates it\n\
     and writes two example scripts: my_prs.js and needs_my_review.js.\n\n\
     CACHING\n\
     A script's result is cached for 5 minutes. Reopening a section\n\
     within that window shows the cached items instantly; a stale\n\
     result stays on screen while the script re-runs in the background.\n\
     Until a script's first run finishes, its sidebar title is derived\n\
     from its file name (my_prs.js is shown as \"My Prs\").\n\n\
     See also: --info scripts, --info tiles, --info groups, --info example\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_documents_the_config_directory() {
        assert!(render().contains("~/.sprawl/default"));
    }

    #[test]
    fn page_documents_first_run_scaffolding() {
        let page = render();

        assert!(page.contains("my_prs.js"));
        assert!(page.contains("needs_my_review.js"));
    }

    #[test]
    fn page_documents_the_cache_window() {
        assert!(render().contains("5 minutes"));
    }
}

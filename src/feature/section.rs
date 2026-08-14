//! A section of the app: one config script, one sidebar entry, one pane of
//! items.

pub mod cache;
pub mod store;

use std::path::PathBuf;

/// One sidebar section, backed by a config script.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Section {
    /// The script that produces this section's content.
    pub path: PathBuf,
    /// The title shown until the script's own `title` is available.
    pub fallback_title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_with_the_same_path_and_title_are_equal() {
        let a = Section {
            path: PathBuf::from("/tmp/a.js"),
            fallback_title: "A".to_string(),
        };

        assert_eq!(a, a.clone());
    }
}

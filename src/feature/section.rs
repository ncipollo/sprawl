//! A section of the app: one sidebar entry, one pane of items. Where its
//! content comes from — a local config script or a sprawl server — is
//! hidden behind a [`source::SectionSource`].

pub mod cache;
pub mod source;
pub mod store;

use std::path::{Path, PathBuf};

/// What identifies a section and keys its cache entry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SectionKey {
    /// A section backed by a config script on this machine.
    Script(PathBuf),
    /// A section on a sprawl server, by the id the server gave it.
    Remote(String),
}

impl SectionKey {
    /// The script behind this section, when it is local.
    pub fn script_path(&self) -> Option<&Path> {
        match self {
            SectionKey::Script(path) => Some(path),
            SectionKey::Remote(_) => None,
        }
    }
}

/// One sidebar section.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Section {
    /// Identifies the section to its [`source::SectionSource`].
    pub key: SectionKey,
    /// The title shown until the section's own `title` is available.
    pub fallback_title: String,
}

impl Section {
    /// A section backed by the script at `path`.
    pub fn local(path: PathBuf, fallback_title: String) -> Self {
        Self {
            key: SectionKey::Script(path),
            fallback_title,
        }
    }

    /// A section served by a sprawl server under `id`.
    pub fn remote(id: String, fallback_title: String) -> Self {
        Self {
            key: SectionKey::Remote(id),
            fallback_title,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_with_the_same_key_and_title_are_equal() {
        let a = Section::local(PathBuf::from("/tmp/a.js"), "A".to_string());

        assert_eq!(a, a.clone());
    }

    #[test]
    fn only_local_keys_have_a_script_path() {
        let local = SectionKey::Script(PathBuf::from("/tmp/a.js"));
        let remote = SectionKey::Remote("a".to_string());

        assert_eq!(local.script_path(), Some(Path::new("/tmp/a.js")));
        assert_eq!(remote.script_path(), None);
    }

    #[test]
    fn a_local_and_a_remote_key_never_collide() {
        assert_ne!(
            SectionKey::Script(PathBuf::from("a")),
            SectionKey::Remote("a".to_string())
        );
    }
}

//! Locating and listing the user's config scripts in `~/.sprawl/default`.

pub mod examples;
pub mod scaffold;

use crate::feature::section::Section;
use std::io;
use std::path::{Path, PathBuf};

/// The user's sprawl config root, `~/.sprawl`.
pub fn config_root() -> Option<PathBuf> {
    std::env::home_dir().map(|home| home.join(".sprawl"))
}

/// The directory holding the default profile's section scripts.
pub fn default_dir(root: &Path) -> PathBuf {
    root.join("default")
}

/// The sections defined in `dir`: every `*.js` file, sorted by file name.
pub fn list_sections(dir: &Path) -> io::Result<Vec<Section>> {
    let mut paths: Vec<PathBuf> = dir
        .read_dir()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "js"))
        .collect();
    paths.sort();
    Ok(paths
        .into_iter()
        .map(|path| {
            let fallback_title = fallback_title(&path);
            Section {
                path,
                fallback_title,
            }
        })
        .collect())
}

/// A sidebar title derived from a script's file name, shown until the
/// script provides its own: `my_prs.js` becomes "My Prs".
pub fn fallback_title(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("");
    stem.split(['_', '-'])
        .filter(|word| !word.is_empty())
        .map(title_case)
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case(word: &str) -> String {
    let mut characters = word.chars();
    match characters.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(characters).collect(),
    }
}

/// The sections to show at startup: scaffolds the config directory with
/// example scripts on first run, then lists it. Problems are reported to
/// stderr and produce an empty section list rather than a crash.
pub fn startup_sections() -> Vec<Section> {
    let Some(root) = config_root() else {
        eprintln!("sprawl: could not determine the home directory");
        return Vec::new();
    };
    if let Err(error) = scaffold::ensure_scaffold(&root) {
        eprintln!("sprawl: could not create {}: {error}", root.display());
        return Vec::new();
    }
    let dir = default_dir(&root);
    match list_sections(&dir) {
        Ok(sections) => sections,
        Err(error) => {
            eprintln!("sprawl: could not read {}: {error}", dir.display());
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn list_sections_returns_js_files_sorted_by_name() {
        let dir = TempDir::new().expect("should create");
        fs::write(dir.path().join("b_second.js"), "").expect("should write");
        fs::write(dir.path().join("a_first.js"), "").expect("should write");

        let sections = list_sections(dir.path()).expect("should list");

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].fallback_title, "A First");
        assert_eq!(sections[1].fallback_title, "B Second");
    }

    #[test]
    fn list_sections_ignores_files_that_are_not_scripts() {
        let dir = TempDir::new().expect("should create");
        fs::write(dir.path().join("notes.txt"), "").expect("should write");
        fs::write(dir.path().join("section.js"), "").expect("should write");

        let sections = list_sections(dir.path()).expect("should list");

        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn list_sections_fails_when_the_directory_is_missing() {
        assert!(list_sections(Path::new("/definitely/not/a/dir")).is_err());
    }

    #[test]
    fn fallback_title_splits_underscores_and_hyphens_into_title_case() {
        assert_eq!(fallback_title(Path::new("my_prs.js")), "My Prs");
        assert_eq!(
            fallback_title(Path::new("needs-my-review.js")),
            "Needs My Review"
        );
    }

    #[test]
    fn default_dir_is_the_default_profile_inside_the_root() {
        assert_eq!(
            default_dir(Path::new("/home/u/.sprawl")),
            PathBuf::from("/home/u/.sprawl/default")
        );
    }
}

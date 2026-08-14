//! First-run scaffolding: creating `~/.sprawl/default` with the example
//! scripts.

use crate::feature::config::{default_dir, examples};
use std::fs;
use std::io;
use std::path::Path;

/// Creates `root` (`~/.sprawl`) with the default profile and example
/// scripts, unless it already exists. Returns whether anything was created.
pub fn ensure_scaffold(root: &Path) -> io::Result<bool> {
    if root.exists() {
        return Ok(false);
    }
    let dir = default_dir(root);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("my_prs.js"), examples::my_prs())?;
    fs::write(dir.join("needs_my_review.js"), examples::needs_my_review())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn a_missing_root_is_scaffolded_with_both_examples() {
        let parent = TempDir::new().expect("should create");
        let root = parent.path().join(".sprawl");

        let created = ensure_scaffold(&root).expect("should scaffold");

        assert!(created);
        let dir = default_dir(&root);
        assert!(dir.join("my_prs.js").exists());
        assert!(dir.join("needs_my_review.js").exists());
    }

    #[test]
    fn an_existing_root_is_left_untouched() {
        let parent = TempDir::new().expect("should create");
        let root = parent.path().join(".sprawl");
        fs::create_dir_all(&root).expect("should create");

        let created = ensure_scaffold(&root).expect("should not scaffold");

        assert!(!created);
        assert!(!default_dir(&root).exists());
    }

    #[test]
    fn the_scaffolded_scripts_hold_the_example_sources() {
        let parent = TempDir::new().expect("should create");
        let root = parent.path().join(".sprawl");
        ensure_scaffold(&root).expect("should scaffold");

        let written =
            fs::read_to_string(default_dir(&root).join("my_prs.js")).expect("should read");

        assert_eq!(written, examples::my_prs());
    }
}

//! Where rendered sections live. A section's id is its script's file stem;
//! its JSON file is `<output dir>/<id>.json`.

use std::path::{Path, PathBuf};

/// The directory rendered sections are written to, `~/.sprawl/output`.
pub fn output_dir(root: &Path) -> PathBuf {
    root.join("output")
}

/// The stable, path-safe id of the section backed by `script`.
pub fn section_id(script: &Path) -> Option<String> {
    script.file_stem()?.to_str().map(str::to_string)
}

/// The JSON file holding section `id`'s rendered output.
pub fn output_path(output_dir: &Path, id: &str) -> PathBuf {
    output_dir.join(format!("{id}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_is_the_script_file_stem() {
        assert_eq!(
            section_id(Path::new("/x/default/my_prs.js")),
            Some("my_prs".to_string())
        );
    }

    #[test]
    fn the_output_path_round_trips_the_id() {
        let path = output_path(Path::new("/out"), "my_prs");

        assert_eq!(path, PathBuf::from("/out/my_prs.json"));
    }

    #[test]
    fn the_output_dir_sits_under_the_config_root() {
        assert_eq!(
            output_dir(Path::new("/home/u/.sprawl")),
            PathBuf::from("/home/u/.sprawl/output")
        );
    }
}

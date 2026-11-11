//! Runs section scripts and persists the results as JSON.

use crate::feature::config;
use crate::feature::script::shell::ShellRunner;
use crate::feature::script::{self};
use crate::feature::section::Section;
use crate::feature::server::error::RenderError;
use crate::feature::server::paths;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// The result of rendering one section.
#[derive(Debug)]
pub struct RenderOutcome {
    pub id: String,
    pub result: Result<(), RenderError>,
}

/// Evaluates section scripts and writes their output. Cloning is cheap and
/// shares the write lock, so a renderer driven by a timer and another
/// driven by requests never write the same file at once.
#[derive(Clone)]
pub struct SectionRenderer {
    scripts_dir: PathBuf,
    output_dir: PathBuf,
    runner: Arc<dyn ShellRunner>,
    write_lock: Arc<Mutex<()>>,
}

impl SectionRenderer {
    pub fn new(scripts_dir: PathBuf, output_dir: PathBuf, runner: Arc<dyn ShellRunner>) -> Self {
        Self {
            scripts_dir,
            output_dir,
            runner,
            write_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn scripts_dir(&self) -> &Path {
        &self.scripts_dir
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Renders every script currently in the scripts directory.
    pub fn render_all(&self) -> io::Result<Vec<RenderOutcome>> {
        let sections = config::list_sections(&self.scripts_dir)?;
        Ok(sections
            .iter()
            .filter_map(|section| {
                let id = paths::id_of(section)?;
                let result = self.render_section(section);
                Some(RenderOutcome { id, result })
            })
            .collect())
    }

    /// Renders the section with script stem `id`.
    pub fn render_id(&self, id: &str) -> Result<(), RenderError> {
        let sections = config::list_sections(&self.scripts_dir)?;
        let section = sections
            .iter()
            .find(|section| paths::id_of(section).as_deref() == Some(id))
            .ok_or_else(|| RenderError::UnknownSection(id.to_string()))?;
        self.render_section(section)
    }

    /// Runs one script and replaces its JSON file. A failing script leaves
    /// the previous file untouched.
    pub fn render_section(&self, section: &Section) -> Result<(), RenderError> {
        let id = paths::id_of(section).unwrap_or_default();
        let path = section
            .key
            .script_path()
            .ok_or_else(|| RenderError::UnknownSection(id.clone()))?;
        let config = script::run_file(path, self.runner.clone())?;
        let json = serde_json::to_string_pretty(&config)?;
        let _guard = self.write_lock.lock().unwrap_or_else(|e| e.into_inner());
        fs::create_dir_all(&self.output_dir)?;
        write_atomically(&paths::output_path(&self.output_dir, &id), &json)?;
        Ok(())
    }
}

/// Writes to a sibling temp file then renames, so readers never see a
/// half-written section.
fn write_atomically(target: &Path, contents: &str) -> io::Result<()> {
    let temp = target.with_extension("json.tmp");
    fs::write(&temp, contents)?;
    fs::rename(&temp, target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::script::schema::SectionConfig;
    use crate::feature::script::shell::fake::FakeShell;
    use tempfile::TempDir;

    struct Fixture {
        _root: TempDir,
        scripts: PathBuf,
        output: PathBuf,
        renderer: SectionRenderer,
    }

    fn fixture() -> Fixture {
        let root = TempDir::new().expect("should create");
        let scripts = root.path().join("default");
        let output = root.path().join("output");
        fs::create_dir_all(&scripts).expect("should create");
        let renderer = SectionRenderer::new(
            scripts.clone(),
            output.clone(),
            Arc::new(FakeShell::outputting("")),
        );
        Fixture {
            _root: root,
            scripts,
            output,
            renderer,
        }
    }

    fn write_script(fixture: &Fixture, name: &str, title: &str) {
        let source =
            format!("({{ title: \"{title}\", items: [{{ type: \"tile\", title: \"t\" }}] }});");
        fs::write(fixture.scripts.join(name), source).expect("should write");
    }

    fn read_output(fixture: &Fixture, id: &str) -> SectionConfig {
        let text =
            fs::read_to_string(paths::output_path(&fixture.output, id)).expect("should read");
        serde_json::from_str(&text).expect("should parse")
    }

    #[test]
    fn rendering_writes_one_json_file_per_script() {
        let fixture = fixture();
        write_script(&fixture, "a.js", "A");
        write_script(&fixture, "b.js", "B");

        let outcomes = fixture.renderer.render_all().expect("should list");

        assert!(outcomes.iter().all(|outcome| outcome.result.is_ok()));
        assert_eq!(read_output(&fixture, "a").title, "A");
        assert_eq!(read_output(&fixture, "b").title, "B");
    }

    #[test]
    fn a_failing_script_keeps_the_last_good_file() {
        let fixture = fixture();
        write_script(&fixture, "a.js", "Good");
        fixture.renderer.render_all().expect("should list");
        fs::write(fixture.scripts.join("a.js"), "throw new Error('boom');").expect("should write");

        let outcomes = fixture.renderer.render_all().expect("should list");

        assert!(outcomes[0].result.is_err());
        assert_eq!(read_output(&fixture, "a").title, "Good");
    }

    #[test]
    fn a_failing_script_with_no_prior_result_writes_nothing() {
        let fixture = fixture();
        fs::write(fixture.scripts.join("a.js"), "nope(").expect("should write");

        fixture.renderer.render_all().expect("should list");

        assert!(!paths::output_path(&fixture.output, "a").exists());
    }

    #[test]
    fn render_id_renders_only_that_section() {
        let fixture = fixture();
        write_script(&fixture, "a.js", "A");
        write_script(&fixture, "b.js", "B");

        fixture.renderer.render_id("b").expect("should render");

        assert!(!paths::output_path(&fixture.output, "a").exists());
        assert_eq!(read_output(&fixture, "b").title, "B");
    }

    #[test]
    fn render_id_rejects_an_unknown_section() {
        let fixture = fixture();

        let result = fixture.renderer.render_id("missing");

        assert!(matches!(result, Err(RenderError::UnknownSection(_))));
    }

    #[test]
    fn a_missing_scripts_directory_is_an_error() {
        let renderer = SectionRenderer::new(
            PathBuf::from("/definitely/not/a/dir"),
            PathBuf::from("/tmp/unused"),
            Arc::new(FakeShell::outputting("")),
        );

        assert!(renderer.render_all().is_err());
    }
}

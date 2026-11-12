//! What the HTTP endpoints do, without any HTTP: list sections, read one
//! section's rendered layout, and force a re-render.

use crate::feature::config;
use crate::feature::script::schema::SectionConfig;
use crate::feature::server::error::ServiceError;
use crate::feature::server::paths;
use crate::feature::server::renderer::SectionRenderer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;

/// One entry of the section list: a stable id and the title to show.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SectionSummary {
    pub id: String,
    pub title: String,
}

/// Serves rendered sections. Cloning is cheap and shares the renderer's
/// write lock with the interval renderer.
#[derive(Clone)]
pub struct SectionService {
    renderer: SectionRenderer,
}

impl SectionService {
    pub fn new(renderer: SectionRenderer) -> Self {
        Self { renderer }
    }

    /// Every section with a script, sorted by id. The title comes from the
    /// last render, or from the script's file name before its first one.
    pub fn list(&self) -> Result<Vec<SectionSummary>, ServiceError> {
        let sections = config::list_sections(self.renderer.scripts_dir())?;
        Ok(sections
            .into_iter()
            .filter_map(|section| {
                let id = paths::id_of(&section)?;
                let title = self
                    .read_rendered(&id)
                    .map(|config| config.title)
                    .unwrap_or(section.fallback_title);
                Some(SectionSummary { id, title })
            })
            .collect())
    }

    /// The layout last rendered for `id`, rendering it first if the server
    /// has not got to it yet.
    pub fn layout(&self, id: &str) -> Result<SectionConfig, ServiceError> {
        self.ensure_known(id)?;
        match self.read_rendered(id) {
            Some(config) => Ok(config),
            None => self.refresh(id),
        }
    }

    /// Re-runs `id`'s script and returns the fresh layout. If the script
    /// fails the previous file is kept and the error is returned.
    pub fn refresh(&self, id: &str) -> Result<SectionConfig, ServiceError> {
        self.ensure_known(id)?;
        self.renderer.render_id(id)?;
        self.read_file(id)
    }

    fn ensure_known(&self, id: &str) -> Result<(), ServiceError> {
        let sections = config::list_sections(self.renderer.scripts_dir())?;
        let known = sections
            .iter()
            .any(|section| paths::id_of(section).as_deref() == Some(id));
        if known {
            Ok(())
        } else {
            Err(ServiceError::UnknownSection(id.to_string()))
        }
    }

    fn read_rendered(&self, id: &str) -> Option<SectionConfig> {
        self.read_file(id).ok()
    }

    fn read_file(&self, id: &str) -> Result<SectionConfig, ServiceError> {
        let path = paths::output_path(self.renderer.output_dir(), id);
        let text = fs::read_to_string(&path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => ServiceError::NotRendered(id.to_string()),
            _ => ServiceError::Io(error),
        })?;
        serde_json::from_str(&text).map_err(|error| ServiceError::Corrupt(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::script::shell::fake::FakeShell;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn service(root: &Path) -> SectionService {
        let scripts = root.join("default");
        fs::create_dir_all(&scripts).expect("should create");
        let renderer = SectionRenderer::new(
            scripts,
            root.join("output"),
            Arc::new(FakeShell::outputting("")),
        );
        SectionService::new(renderer)
    }

    fn write_script(root: &Path, name: &str, title: &str) {
        let source = format!("({{ title: \"{title}\", items: [] }});");
        fs::write(root.join("default").join(name), source).expect("should write");
    }

    #[test]
    fn the_list_falls_back_to_the_file_name_before_a_render() {
        let root = TempDir::new().expect("should create");
        let service = service(root.path());
        write_script(root.path(), "my_prs.js", "Pull Requests");

        let list = service.list().expect("should list");

        assert_eq!(
            list,
            vec![SectionSummary {
                id: "my_prs".to_string(),
                title: "My Prs".to_string()
            }]
        );
    }

    #[test]
    fn the_list_uses_the_rendered_title_once_available() {
        let root = TempDir::new().expect("should create");
        let service = service(root.path());
        write_script(root.path(), "my_prs.js", "Pull Requests");
        service.refresh("my_prs").expect("should refresh");

        let list = service.list().expect("should list");

        assert_eq!(list[0].title, "Pull Requests");
    }

    #[test]
    fn layout_renders_on_demand_when_nothing_is_on_disk() {
        let root = TempDir::new().expect("should create");
        let service = service(root.path());
        write_script(root.path(), "a.js", "A");

        let layout = service.layout("a").expect("should render");

        assert_eq!(layout.title, "A");
        assert!(root.path().join("output/a.json").exists());
    }

    #[test]
    fn layout_serves_the_file_without_rerunning_the_script() {
        let root = TempDir::new().expect("should create");
        let service = service(root.path());
        write_script(root.path(), "a.js", "Old");
        service.refresh("a").expect("should refresh");
        write_script(root.path(), "a.js", "New");

        let layout = service.layout("a").expect("should read");

        assert_eq!(layout.title, "Old");
    }

    #[test]
    fn refresh_reruns_the_script() {
        let root = TempDir::new().expect("should create");
        let service = service(root.path());
        write_script(root.path(), "a.js", "Old");
        service.refresh("a").expect("should refresh");
        write_script(root.path(), "a.js", "New");

        let layout = service.refresh("a").expect("should refresh");

        assert_eq!(layout.title, "New");
    }

    #[test]
    fn a_failed_refresh_keeps_serving_the_last_good_layout() {
        let root = TempDir::new().expect("should create");
        let service = service(root.path());
        write_script(root.path(), "a.js", "Good");
        service.refresh("a").expect("should refresh");
        fs::write(root.path().join("default/a.js"), "throw 1;").expect("should write");

        assert!(service.refresh("a").is_err());
        assert_eq!(service.layout("a").expect("should read").title, "Good");
    }

    #[test]
    fn unknown_and_path_like_ids_are_rejected() {
        let root = TempDir::new().expect("should create");
        let service = service(root.path());
        write_script(root.path(), "a.js", "A");

        for id in ["missing", "../a", "a/../a", ""] {
            assert!(
                matches!(service.layout(id), Err(ServiceError::UnknownSection(_))),
                "{id}"
            );
        }
    }
}

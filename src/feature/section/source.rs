//! Where section content comes from. The ui talks to a [`SectionSource`]
//! and never knows whether scripts run here or on a server.

use crate::feature::script::error::ScriptError;
use crate::feature::script::schema::SectionConfig;
use crate::feature::script::shell::SystemShell;
use crate::feature::script::{self};
use crate::feature::section::SectionKey;
use std::sync::Arc;

/// Produces section content. Called from background threads.
pub trait SectionSource: Send + Sync + 'static {
    /// The section's content: for a local source this runs its script; for
    /// a remote one it reads what the server last rendered.
    fn load(&self, key: &SectionKey) -> Result<SectionConfig, ScriptError>;

    /// The section's content, recomputed. A remote source asks the server to
    /// re-run the script and returns the fresh result.
    fn refresh(&self, key: &SectionKey) -> Result<SectionConfig, ScriptError>;

    /// Whether opening a section should recompute it rather than reuse a
    /// cached or server-rendered result. Local scripts follow the cache
    /// policy; remote sections refresh on selection.
    fn refreshes_on_select(&self) -> bool;
}

/// Runs config scripts on this machine.
pub struct LocalSource;

impl LocalSource {
    pub fn shared() -> Arc<dyn SectionSource> {
        Arc::new(LocalSource)
    }
}

impl SectionSource for LocalSource {
    fn load(&self, key: &SectionKey) -> Result<SectionConfig, ScriptError> {
        let path = key
            .script_path()
            .ok_or_else(|| ScriptError::Read("not a local section".to_string()))?;
        script::run_file(path, Arc::new(SystemShell))
    }

    fn refresh(&self, key: &SectionKey) -> Result<SectionConfig, ScriptError> {
        self.load(key)
    }

    fn refreshes_on_select(&self) -> bool {
        false
    }
}

/// The sections to show at startup from a local config directory.
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn a_local_source_runs_the_script_for_load_and_refresh() {
        let dir = TempDir::new().expect("should create");
        let path = dir.path().join("a.js");
        fs::write(&path, "({ title: 'A', items: [] });").expect("should write");
        let key = SectionKey::Script(path);

        assert_eq!(LocalSource.load(&key).expect("should run").title, "A");
        assert_eq!(LocalSource.refresh(&key).expect("should run").title, "A");
    }

    #[test]
    fn a_local_source_rejects_a_remote_key() {
        let result = LocalSource.load(&SectionKey::Remote("a".to_string()));

        assert!(result.is_err());
    }

    #[test]
    fn a_local_source_follows_the_cache_policy_on_select() {
        assert!(!LocalSource.refreshes_on_select());
    }
}

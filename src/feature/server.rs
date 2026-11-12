//! Headless server mode: runs every section script on an interval and
//! writes each rendered section to a JSON file on disk. The HTTP layer
//! serves these files and triggers re-renders through [`SectionRenderer`].

pub mod error;
pub mod paths;
pub mod renderer;

use crate::feature::config;
use crate::feature::script::shell::SystemShell;
use crate::feature::server::error::ServerError;
use crate::feature::server::renderer::SectionRenderer;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// The default time between renders of every section.
pub const DEFAULT_INTERVAL_SECS: u64 = 300;

/// Renders every section now, then again every `interval`, forever.
/// Returns only if the config directory cannot be located.
pub fn run(interval: Duration) -> Result<(), ServerError> {
    let root = config::config_root().ok_or(ServerError::NoHomeDirectory)?;
    config::scaffold::ensure_scaffold(&root).map_err(ServerError::Scaffold)?;
    let renderer = SectionRenderer::new(
        config::default_dir(&root),
        paths::output_dir(&root),
        Arc::new(SystemShell),
    );
    eprintln!(
        "sprawl: serving sections from {} into {} every {}s",
        renderer.scripts_dir().display(),
        renderer.output_dir().display(),
        interval.as_secs()
    );
    loop {
        report(&renderer);
        thread::sleep(interval);
    }
}

fn report(renderer: &SectionRenderer) {
    match renderer.render_all() {
        Ok(outcomes) => {
            for outcome in outcomes {
                if let Err(error) = outcome.result {
                    eprintln!("sprawl: section {} not updated: {error}", outcome.id);
                }
            }
        }
        Err(error) => eprintln!("sprawl: {error}"),
    }
}

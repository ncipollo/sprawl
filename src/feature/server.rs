//! Headless server mode: runs every section script on an interval and
//! writes each rendered section to a JSON file on disk. The HTTP layer
//! (`network::server`) serves these files through [`service::SectionService`].

pub mod error;
pub mod paths;
pub mod renderer;
pub mod service;

use crate::feature::config;
use crate::feature::script::shell::SystemShell;
use crate::feature::server::error::ServerError;
use crate::feature::server::renderer::SectionRenderer;
use crate::feature::server::service::SectionService;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// The default time between renders of every section.
pub const DEFAULT_INTERVAL_SECS: u64 = 300;

/// Starts rendering every section now and then every `interval` on a
/// background thread, and returns the service that reads and refreshes
/// what it produces.
pub fn start(interval: Duration) -> Result<SectionService, ServerError> {
    let root = config::config_root().ok_or(ServerError::NoHomeDirectory)?;
    config::scaffold::ensure_scaffold(&root).map_err(ServerError::Scaffold)?;
    let renderer = SectionRenderer::new(
        config::default_dir(&root),
        paths::output_dir(&root),
        Arc::new(SystemShell),
    );
    eprintln!(
        "sprawl: rendering {} into {} every {}s",
        renderer.scripts_dir().display(),
        renderer.output_dir().display(),
        interval.as_secs()
    );
    let background = renderer.clone();
    thread::spawn(move || {
        loop {
            report(&background);
            thread::sleep(interval);
        }
    });
    Ok(SectionService::new(renderer))
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

//! Errors from the headless server.

use crate::feature::script::error::ScriptError;
use std::io;
use thiserror::Error;

/// Why the server could not start.
#[derive(Debug, Error)]
pub enum ServerError {
    /// `~` could not be resolved.
    #[error("could not determine the home directory")]
    NoHomeDirectory,
    /// The config directory could not be created.
    #[error("could not create the config directory: {0}")]
    Scaffold(io::Error),
}

/// Why one section was not rendered.
#[derive(Debug, Error)]
pub enum RenderError {
    /// The script failed; the previous JSON file, if any, is kept.
    #[error(transparent)]
    Script(#[from] ScriptError),
    /// The result could not be written; the previous file is kept.
    #[error("could not write the rendered section: {0}")]
    Write(#[from] io::Error),
    /// The result could not be turned into JSON.
    #[error("could not encode the rendered section: {0}")]
    Encode(#[from] serde_json::Error),
    /// No script has this id.
    #[error("no section named {0:?}")]
    UnknownSection(String),
}

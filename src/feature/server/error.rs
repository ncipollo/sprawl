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

/// Why a request to the section service failed.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// No script has this id.
    #[error("no section named {0:?}")]
    UnknownSection(String),
    /// The section exists but has never rendered successfully.
    #[error("section {0:?} has not rendered yet")]
    NotRendered(String),
    /// Running the script failed.
    #[error(transparent)]
    Render(RenderError),
    /// The scripts or output directory could not be read.
    #[error("could not read sections: {0}")]
    Io(#[from] io::Error),
    /// A rendered file did not parse.
    #[error("a rendered section file is corrupt: {0}")]
    Corrupt(String),
}

impl From<RenderError> for ServiceError {
    fn from(error: RenderError) -> Self {
        match error {
            RenderError::UnknownSection(id) => ServiceError::UnknownSection(id),
            other => ServiceError::Render(other),
        }
    }
}

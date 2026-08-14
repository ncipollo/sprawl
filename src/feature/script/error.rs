//! Errors that can occur while running a section config script.

use thiserror::Error;

/// Something that went wrong reading, evaluating, or interpreting a
/// config script.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ScriptError {
    /// The script file could not be read.
    #[error("The script could not be read: {0}")]
    Read(String),
    /// The script failed to evaluate: a syntax error, an uncaught throw,
    /// or a failed `shell` call the script did not catch.
    #[error("The script failed: {0}")]
    Evaluate(String),
    /// The script's final value was not a JSON-representable object.
    #[error("The script did not end with a section object like ({{ title, items }});")]
    NotAnObject,
    /// The script's final value did not match the section schema.
    #[error("The script returned an unexpected shape: {0}")]
    Schema(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_an_object_explains_the_expected_completion_value() {
        let message = ScriptError::NotAnObject.to_string();

        assert!(message.contains("({ title, items });"));
    }

    #[test]
    fn evaluate_includes_the_underlying_message() {
        let message = ScriptError::Evaluate("boom".to_string()).to_string();

        assert!(message.contains("boom"));
    }
}

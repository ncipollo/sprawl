//! Evaluating section config scripts in a sandboxed JavaScript engine.
//!
//! A script's completion value — its final expression statement, written
//! `({ title, items });` — is the section object. The only capability
//! injected into the sandbox is [`bindings`]' `shell` function.

pub mod bindings;
pub mod error;
pub mod schema;
pub mod shell;

use crate::feature::script::error::ScriptError;
use crate::feature::script::schema::SectionConfig;
use crate::feature::script::shell::ShellRunner;
use boa_engine::{Context, Source};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::thread;

/// Evaluates `source` and interprets its completion value as a section.
pub fn evaluate(source: &str, runner: Arc<dyn ShellRunner>) -> Result<SectionConfig, ScriptError> {
    let mut context = Context::default();
    bindings::register_shell(&mut context, runner)?;
    let value = context
        .eval(Source::from_bytes(source))
        .map_err(|error| ScriptError::Evaluate(error.to_string()))?;
    let json = value
        .to_json(&mut context)
        .map_err(|error| ScriptError::Evaluate(error.to_string()))?
        .ok_or(ScriptError::NotAnObject)?;
    serde_json::from_value(json).map_err(|error| ScriptError::Schema(error.to_string()))
}

/// Reads and evaluates the script at `path` on its own thread. `boa`'s
/// recursive-descent parser needs a full-size stack; the GCD queue that
/// `gpui` runs background work on gives only 512KB, which an unoptimized
/// build overflows.
pub fn run_file(path: &Path, runner: Arc<dyn ShellRunner>) -> Result<SectionConfig, ScriptError> {
    let path = path.to_path_buf();
    thread::spawn(move || {
        let source =
            fs::read_to_string(&path).map_err(|error| ScriptError::Read(error.to_string()))?;
        evaluate(&source, runner)
    })
    .join()
    .unwrap_or_else(|_| Err(ScriptError::Evaluate("the script crashed".to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::script::schema::SectionItem;
    use crate::feature::script::shell::fake::FakeShell;

    fn fake() -> Arc<FakeShell> {
        Arc::new(FakeShell::outputting(""))
    }

    #[test]
    fn a_script_returning_a_section_object_parses() {
        let source = r#"
            const items = [{ type: "tile", title: "hello" }];
            ({ title: "Test", items });
        "#;

        let config = evaluate(source, fake()).expect("should parse");

        assert_eq!(config.title, "Test");
        let SectionItem::Tile(tile) = &config.items[0];
        assert_eq!(tile.title, "hello");
    }

    #[test]
    fn a_script_can_build_its_section_from_shell_output() {
        let shell = Arc::new(FakeShell::outputting(r#"{"names": ["a", "b"]}"#));
        let source = r#"
            const parsed = JSON.parse(shell("list-things"));
            ({
                title: "Things",
                items: parsed.names.map((name) => ({ type: "tile", title: name })),
            });
        "#;

        let config = evaluate(source, shell).expect("should parse");

        assert_eq!(config.items.len(), 2);
    }

    #[test]
    fn a_script_without_a_final_object_is_not_a_section() {
        let error = evaluate("const x = 1;", fake()).expect_err("should fail");

        assert_eq!(error, ScriptError::NotAnObject);
    }

    #[test]
    fn a_syntax_error_fails_evaluation() {
        let error = evaluate("this is not javascript", fake()).expect_err("should fail");

        assert!(matches!(error, ScriptError::Evaluate(_)));
    }

    #[test]
    fn an_uncaught_throw_fails_evaluation_with_its_message() {
        let error = evaluate(r#"throw new Error("kaboom");"#, fake()).expect_err("should fail");

        let ScriptError::Evaluate(message) = error else {
            panic!("expected an evaluation error");
        };
        assert!(message.contains("kaboom"));
    }

    #[test]
    fn an_uncaught_shell_failure_fails_evaluation() {
        let shell = Arc::new(FakeShell::failing());

        let error = evaluate(r#"shell("boom");"#, shell).expect_err("should fail");

        assert!(matches!(error, ScriptError::Evaluate(_)));
    }

    #[test]
    fn a_valid_object_with_the_wrong_shape_is_a_schema_error() {
        let error = evaluate(r#"({ title: "T", items: [{ type: "mystery" }] });"#, fake())
            .expect_err("should fail");

        assert!(matches!(error, ScriptError::Schema(_)));
    }

    /// Globals that would give a script network or host access. Every
    /// one must be absent: `shell` is the sandbox's only door out.
    const FORBIDDEN_GLOBALS: &[&str] = &[
        "fetch",
        "XMLHttpRequest",
        "WebSocket",
        "EventSource",
        "Request",
        "Response",
        "Headers",
        "URL",
        "Worker",
        "navigator",
        "window",
        "self",
        "global",
        "require",
        "process",
        "Deno",
        "Bun",
        "setTimeout",
        "setInterval",
    ];

    #[test]
    fn the_sandbox_has_no_network_or_host_globals() {
        let checks = FORBIDDEN_GLOBALS
            .iter()
            .map(|name| format!("{{ type: \"tile\", title: typeof {name} }}"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!("({{ title: \"probe\", items: [{checks}] }});");

        let config = evaluate(&source, fake()).expect("should parse");

        for (name, item) in FORBIDDEN_GLOBALS.iter().zip(&config.items) {
            let SectionItem::Tile(tile) = item;
            assert_eq!(tile.title, "undefined", "{name} is reachable from scripts");
        }
    }

    #[test]
    fn the_only_global_scripts_can_enumerate_is_shell() {
        let source = r#"({ title: "probe", items: [{ type: "tile", title: Object.getOwnPropertyNames(globalThis).filter((n) => !(n in Object.getPrototypeOf(globalThis) ?? {})).sort().join(",") }] });"#;

        let config = evaluate(source, fake()).expect("should parse");

        let SectionItem::Tile(tile) = &config.items[0];
        assert!(
            !tile
                .title
                .split(',')
                .any(|name| FORBIDDEN_GLOBALS.contains(&name)),
            "forbidden global present: {}",
            tile.title
        );
        assert!(tile.title.split(',').any(|name| name == "shell"));
    }

    #[test]
    fn run_file_reports_a_missing_file_as_a_read_error() {
        let error =
            run_file(Path::new("/definitely/not/a/file.js"), fake()).expect_err("should fail");

        assert!(matches!(error, ScriptError::Read(_)));
    }

    #[test]
    fn run_file_does_not_depend_on_the_callers_stack() {
        let dir = tempfile::TempDir::new().expect("should create a temp dir");
        let path = dir.path().join("section.js");
        fs::write(&path, r#"({ title: "Test", items: [] });"#).expect("should write");

        let result = thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(move || run_file(&path, fake()))
            .expect("should spawn")
            .join()
            .expect("the caller thread should not crash");

        assert!(result.is_ok());
    }
}

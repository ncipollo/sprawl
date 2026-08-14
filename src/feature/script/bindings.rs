//! The `shell` global injected into script contexts. It is the only door
//! out of the JavaScript sandbox: boa's default context has no fetch, file
//! system, network, timers, or module loader.

use crate::feature::script::error::ScriptError;
use crate::feature::script::shell::{ShellInvocation, ShellRunner};
use boa_engine::{Context, JsNativeError, JsResult, JsString, JsValue, NativeFunction};
use boa_gc::{Finalize, Trace};
use std::sync::Arc;

/// The runner smuggled into the native `shell` function. The garbage
/// collector never needs to trace an `Arc` to plain Rust data.
#[derive(Trace, Finalize)]
struct ShellCapture {
    #[unsafe_ignore_trace]
    runner: Arc<dyn ShellRunner>,
}

/// Registers the global `shell` function on `context`, backed by `runner`.
///
/// From JavaScript: `shell(command)` runs `command` through `/bin/sh -c`;
/// `shell(program, args)` runs `program` directly with an array of string
/// arguments. Both return stdout as a string and throw an `Error` when the
/// command fails.
pub fn register_shell(
    context: &mut Context,
    runner: Arc<dyn ShellRunner>,
) -> Result<(), ScriptError> {
    context
        .register_global_callable(
            JsString::from("shell"),
            1,
            NativeFunction::from_copy_closure_with_captures(shell_native, ShellCapture { runner }),
        )
        .map_err(|error| ScriptError::Evaluate(error.to_string()))
}

fn shell_native(
    _this: &JsValue,
    args: &[JsValue],
    capture: &ShellCapture,
    context: &mut Context,
) -> JsResult<JsValue> {
    let invocation = parse_invocation(args, context)?;
    match capture.runner.run(&invocation) {
        Ok(stdout) => Ok(JsValue::from(JsString::from(stdout))),
        Err(error) => Err(JsNativeError::error()
            .with_message(error.to_string())
            .into()),
    }
}

/// Interprets the JavaScript arguments as a [`ShellInvocation`].
fn parse_invocation(args: &[JsValue], context: &mut Context) -> JsResult<ShellInvocation> {
    let first = args.first().filter(|value| !value.is_undefined());
    let Some(first) = first else {
        return Err(JsNativeError::typ()
            .with_message("shell() requires a command string")
            .into());
    };
    let command = first.to_string(context)?.to_std_string_lossy();
    match args.get(1).filter(|value| !value.is_undefined()) {
        None => Ok(ShellInvocation::Command(command)),
        Some(second) => Ok(ShellInvocation::Program {
            program: command,
            args: args_from_value(second, context)?,
        }),
    }
}

/// Converts the second `shell` argument into an argument vector.
fn args_from_value(value: &JsValue, context: &mut Context) -> JsResult<Vec<String>> {
    let type_error =
        || JsNativeError::typ().with_message("shell() arguments must be an array of strings");
    let json = value.to_json(context)?.ok_or_else(type_error)?;
    let serde_json::Value::Array(elements) = json else {
        return Err(type_error().into());
    };
    elements
        .into_iter()
        .map(|element| match element {
            serde_json::Value::String(text) => Ok(text),
            _ => Err(type_error().into()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::script::shell::fake::FakeShell;
    use boa_engine::Source;

    fn eval_with(shell: Arc<FakeShell>, source: &str) -> JsResult<JsValue> {
        let mut context = Context::default();
        register_shell(&mut context, shell).expect("registering shell should work");
        context.eval(Source::from_bytes(source))
    }

    #[test]
    fn shell_returns_the_commands_stdout() {
        let shell = Arc::new(FakeShell::outputting("hello\n"));

        let value = eval_with(shell, r#"shell("echo hello")"#).expect("should run");

        assert_eq!(
            value.to_json(&mut Context::default()).unwrap(),
            Some(serde_json::json!("hello\n"))
        );
    }

    #[test]
    fn the_command_form_records_a_shell_invocation() {
        let shell = Arc::new(FakeShell::outputting(""));

        eval_with(shell.clone(), r#"shell("echo hi")"#).expect("should run");

        assert_eq!(
            shell.invocations(),
            vec![ShellInvocation::Command("echo hi".to_string())]
        );
    }

    #[test]
    fn the_argv_form_records_a_program_invocation() {
        let shell = Arc::new(FakeShell::outputting(""));

        eval_with(shell.clone(), r#"shell("gh", ["api", "graphql"])"#).expect("should run");

        assert_eq!(
            shell.invocations(),
            vec![ShellInvocation::Program {
                program: "gh".to_string(),
                args: vec!["api".to_string(), "graphql".to_string()],
            }]
        );
    }

    #[test]
    fn a_failing_command_throws_into_javascript() {
        let shell = Arc::new(FakeShell::failing());

        let error = eval_with(shell, r#"shell("boom")"#).expect_err("should throw");

        assert!(error.to_string().contains("command failed"));
    }

    #[test]
    fn a_script_can_catch_a_shell_failure() {
        let shell = Arc::new(FakeShell::failing());

        let value = eval_with(shell, r#"try { shell("boom") } catch (e) { "caught" }"#)
            .expect("the catch should handle it");

        let mut context = Context::default();
        assert_eq!(
            value.to_json(&mut context).unwrap(),
            Some(serde_json::json!("caught"))
        );
    }

    #[test]
    fn calling_shell_without_arguments_is_a_type_error() {
        let shell = Arc::new(FakeShell::outputting(""));

        let error = eval_with(shell, "shell()").expect_err("should throw");

        assert!(error.to_string().contains("requires a command string"));
    }

    #[test]
    fn non_string_argv_elements_are_a_type_error() {
        let shell = Arc::new(FakeShell::outputting(""));

        let error = eval_with(shell, r#"shell("gh", [1, 2])"#).expect_err("should throw");

        assert!(error.to_string().contains("array of strings"));
    }
}

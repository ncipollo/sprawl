//! Errors that can occur while fetching pull requests from GitHub.

use thiserror::Error;

/// Something that went wrong fetching or parsing pull requests.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum GithubError {
    /// The `gh` binary is not on `PATH`.
    #[error("The `gh` command line tool was not found. Install it from https://cli.github.com.")]
    GhNotFound,
    /// `gh` ran, but reported that the user is not signed in.
    #[error("`gh` is not signed in. Run `gh auth login` in a terminal, then try again.")]
    NotAuthenticated,
    /// `gh` could not be spawned, for a reason other than being missing.
    #[error("`gh` could not be run: {0}")]
    Spawn(String),
    /// `gh` ran and exited with a failure status.
    #[error("`gh` failed: {0}")]
    CommandFailed(String),
    /// `gh`'s output was not valid UTF-8.
    #[error("`gh` returned output that was not valid UTF-8.")]
    NonUtf8Output,
    /// `gh`'s output was not the JSON shape we expected.
    #[error("The response from `gh` could not be read: {0}")]
    MalformedJson(String),
    /// The GitHub API itself reported an error.
    #[error("GitHub returned an error: {0}")]
    Api(String),
}

/// Turns a failing `gh` invocation's stderr into the most specific error we
/// can recognise.
pub fn classify_failure(stderr: &str) -> GithubError {
    let message = stderr.trim();
    if is_auth_failure(message) {
        return GithubError::NotAuthenticated;
    }
    GithubError::CommandFailed(first_line(message))
}

/// Whether `stderr` looks like `gh` is not signed in.
fn is_auth_failure(stderr: &str) -> bool {
    let lowered = stderr.to_lowercase();
    [
        "gh auth login",
        "not logged in",
        "bad credentials",
        "authentication",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

/// The first non-empty line of `message`, or `message` itself when it has
/// no line breaks (or is empty).
fn first_line(message: &str) -> String {
    message
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(message)
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_failure_detects_a_missing_login() {
        let error = classify_failure("error: not logged in to any GitHub hosts");

        assert_eq!(error, GithubError::NotAuthenticated);
    }

    #[test]
    fn classify_failure_detects_bad_credentials() {
        let error = classify_failure("HTTP 401: Bad credentials");

        assert_eq!(error, GithubError::NotAuthenticated);
    }

    #[test]
    fn classify_failure_falls_back_to_the_first_line_of_the_message() {
        let error = classify_failure("some other error\nwith extra detail");

        assert_eq!(
            error,
            GithubError::CommandFailed("some other error".to_string())
        );
    }

    #[test]
    fn classify_failure_skips_leading_blank_lines() {
        let error = classify_failure("\n\n  actual error message  \nmore detail");

        assert_eq!(
            error,
            GithubError::CommandFailed("actual error message".to_string())
        );
    }

    #[test]
    fn gh_not_found_tells_the_user_where_to_install_it() {
        let message = GithubError::GhNotFound.to_string();

        assert!(message.contains("cli.github.com"));
    }
}

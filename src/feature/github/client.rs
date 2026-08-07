//! Running the `gh` command line tool. This is the one impurity in the
//! `github` module, kept behind a trait so everything above it — argument
//! building, parsing, mapping, sorting — stays unit testable.

use crate::feature::github::error::{GithubError, classify_failure};
use crate::feature::github::parse;
use crate::feature::github::pull_request::PullRequest;
use crate::feature::github::query::PullRequestQuery;
use std::io::ErrorKind;
use std::process::Command;

/// Runs `gh` with the given arguments and returns its stdout.
pub trait GhRunner {
    fn run(&self, args: &[String]) -> Result<String, GithubError>;
}

/// Runs the real `gh` binary found on `PATH`.
pub struct GhCli;

impl GhRunner for GhCli {
    fn run(&self, args: &[String]) -> Result<String, GithubError> {
        let output = Command::new("gh").args(args).output().map_err(|error| {
            if error.kind() == ErrorKind::NotFound {
                GithubError::GhNotFound
            } else {
                GithubError::Spawn(error.to_string())
            }
        })?;

        if !output.status.success() {
            return Err(classify_failure(&String::from_utf8_lossy(&output.stderr)));
        }
        String::from_utf8(output.stdout).map_err(|_| GithubError::NonUtf8Output)
    }
}

/// Fetches the open pull requests for `query`, newest first.
pub fn fetch_pull_requests(
    runner: &dyn GhRunner,
    query: PullRequestQuery,
) -> Result<Vec<PullRequest>, GithubError> {
    let body = runner.run(&query.gh_args())?;
    parse::parse_pull_requests(&body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FakeRunner {
        args: RefCell<Vec<String>>,
        result: Result<String, GithubError>,
    }

    impl FakeRunner {
        fn returning(result: Result<String, GithubError>) -> Self {
            Self {
                args: RefCell::new(Vec::new()),
                result,
            }
        }
    }

    impl GhRunner for FakeRunner {
        fn run(&self, args: &[String]) -> Result<String, GithubError> {
            *self.args.borrow_mut() = args.to_vec();
            self.result.clone()
        }
    }

    const EMPTY_RESPONSE: &str = r#"{"data":{"search":{"issueCount":0,"nodes":[]}}}"#;

    #[test]
    fn fetch_pull_requests_parses_the_runner_output() {
        let runner = FakeRunner::returning(Ok(EMPTY_RESPONSE.to_string()));

        let pull_requests =
            fetch_pull_requests(&runner, PullRequestQuery::Authored).expect("should succeed");

        assert!(pull_requests.is_empty());
    }

    #[test]
    fn fetch_pull_requests_passes_the_query_arguments_to_gh() {
        let runner = FakeRunner::returning(Ok(EMPTY_RESPONSE.to_string()));

        fetch_pull_requests(&runner, PullRequestQuery::ReviewRequested).expect("should succeed");

        assert_eq!(
            *runner.args.borrow(),
            PullRequestQuery::ReviewRequested.gh_args()
        );
    }

    #[test]
    fn fetch_pull_requests_propagates_a_runner_failure() {
        let runner = FakeRunner::returning(Err(GithubError::NotAuthenticated));

        let error =
            fetch_pull_requests(&runner, PullRequestQuery::Authored).expect_err("should fail");

        assert_eq!(error, GithubError::NotAuthenticated);
    }

    #[test]
    fn fetch_pull_requests_propagates_a_parse_failure() {
        let runner = FakeRunner::returning(Ok("not json".to_string()));

        let error =
            fetch_pull_requests(&runner, PullRequestQuery::Authored).expect_err("should fail");

        assert!(matches!(error, GithubError::MalformedJson(_)));
    }
}

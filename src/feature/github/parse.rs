//! Turning a `gh api graphql` response body into domain pull requests.

use crate::feature::github::error::GithubError;
use crate::feature::github::pull_request::{
    ChecksStatus, PullRequest, ReviewStatus, sort_newest_first,
};
use crate::feature::github::response::{GraphQlError, GraphQlResponse, PullRequestNode};

/// Turns a `gh api graphql` response body into pull requests, newest first.
pub fn parse_pull_requests(body: &str) -> Result<Vec<PullRequest>, GithubError> {
    let response: GraphQlResponse = serde_json::from_str(body)
        .map_err(|error| GithubError::MalformedJson(error.to_string()))?;

    if !response.errors.is_empty() {
        return Err(GithubError::Api(join_messages(&response.errors)));
    }
    let data = response
        .data
        .ok_or_else(|| GithubError::MalformedJson("response had no data".to_string()))?;

    let mut pull_requests: Vec<PullRequest> =
        data.search.nodes.into_iter().filter_map(map_node).collect();
    sort_newest_first(&mut pull_requests);
    Ok(pull_requests)
}

/// Search returns `ISSUE` nodes; anything that isn't a pull request comes
/// back as an object missing the pull-request-only fields, so nodes missing
/// a required field are skipped rather than erroring.
fn map_node(node: PullRequestNode) -> Option<PullRequest> {
    // Computed before the `?`s below move fields out of `node`.
    let checks = ChecksStatus::from_api(node.checks_state());
    Some(PullRequest {
        number: node.number?,
        title: node.title?,
        url: node.url?,
        created_at: node.created_at?,
        repository: node
            .repository
            .and_then(|repository| repository.name_with_owner)
            .unwrap_or_default(),
        author: node.author.and_then(|author| author.login),
        is_draft: node.is_draft,
        comment_count: node.total_comments_count,
        review: ReviewStatus::from_api(node.review_decision.as_deref()),
        checks,
    })
}

fn join_messages(errors: &[GraphQlError]) -> String {
    errors
        .iter()
        .map(|error| error.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_RESPONSE: &str = r#"{"data":{"search":{"issueCount":2,"nodes":[
      {"number":19948,"title":"Show template deserialization errors","url":"https://github.com/WhoopInc/android/pull/19948",
       "createdAt":"2026-08-07T17:42:31Z","isDraft":false,"totalCommentsCount":3,"reviewDecision":"REVIEW_REQUIRED",
       "repository":{"nameWithOwner":"WhoopInc/android"},"author":{"login":"esparkman-whoop"},
       "commits":{"nodes":[{"commit":{"statusCheckRollup":{"state":"SUCCESS"}}}]}},
      {"number":12,"title":"Older change","url":"https://github.com/ncipollo/sprawl/pull/12",
       "createdAt":"2026-08-01T09:00:00Z","isDraft":true,"totalCommentsCount":0,"reviewDecision":null,
       "repository":{"nameWithOwner":"ncipollo/sprawl"},"author":null,
       "commits":{"nodes":[]}}
    ]}}}"#;

    const EMPTY_RESPONSE: &str = r#"{"data":{"search":{"issueCount":0,"nodes":[]}}}"#;
    const ERRORS_RESPONSE: &str =
        r#"{"data":null,"errors":[{"message":"Field 'foo' doesn't exist"}]}"#;
    const NON_PR_NODE_RESPONSE: &str = r#"{"data":{"search":{"issueCount":1,"nodes":[{}]}}}"#;
    const UNKNOWN_FIELD_RESPONSE: &str =
        r#"{"data":{"search":{"issueCount":0,"nodes":[]},"somethingNew":true}}"#;

    #[test]
    fn parse_pull_requests_maps_every_field_of_a_full_response() {
        let pull_requests = parse_pull_requests(FULL_RESPONSE).expect("should parse");
        let newest = &pull_requests[0];

        assert_eq!(newest.number, 19948);
        assert_eq!(newest.title, "Show template deserialization errors");
        assert_eq!(newest.url, "https://github.com/WhoopInc/android/pull/19948");
        assert_eq!(newest.repository, "WhoopInc/android");
        assert_eq!(newest.author, Some("esparkman-whoop".to_string()));
        assert_eq!(newest.created_at, "2026-08-07T17:42:31Z");
        assert!(!newest.is_draft);
        assert_eq!(newest.comment_count, 3);
        assert_eq!(newest.review, ReviewStatus::ReviewRequired);
        assert_eq!(newest.checks, ChecksStatus::Passing);
    }

    #[test]
    fn parse_pull_requests_returns_the_newest_pull_request_first() {
        let pull_requests = parse_pull_requests(FULL_RESPONSE).expect("should parse");

        assert_eq!(pull_requests[0].number, 19948);
        assert_eq!(pull_requests[1].number, 12);
    }

    #[test]
    fn parse_pull_requests_treats_a_null_review_decision_as_no_review() {
        let pull_requests = parse_pull_requests(FULL_RESPONSE).expect("should parse");
        let older = pull_requests
            .iter()
            .find(|pr| pr.number == 12)
            .expect("present");

        assert_eq!(older.review, ReviewStatus::None);
    }

    #[test]
    fn parse_pull_requests_treats_missing_commits_as_unknown_checks() {
        let pull_requests = parse_pull_requests(FULL_RESPONSE).expect("should parse");
        let older = pull_requests
            .iter()
            .find(|pr| pr.number == 12)
            .expect("present");

        assert_eq!(older.checks, ChecksStatus::Unknown);
    }

    #[test]
    fn parse_pull_requests_keeps_a_pull_request_with_a_deleted_author() {
        let pull_requests = parse_pull_requests(FULL_RESPONSE).expect("should parse");
        let older = pull_requests
            .iter()
            .find(|pr| pr.number == 12)
            .expect("present");

        assert_eq!(older.author, None);
    }

    #[test]
    fn parse_pull_requests_skips_nodes_that_are_not_pull_requests() {
        let pull_requests = parse_pull_requests(NON_PR_NODE_RESPONSE).expect("should parse");

        assert!(pull_requests.is_empty());
    }

    #[test]
    fn parse_pull_requests_returns_no_pull_requests_for_an_empty_search() {
        let pull_requests = parse_pull_requests(EMPTY_RESPONSE).expect("should parse");

        assert!(pull_requests.is_empty());
    }

    #[test]
    fn parse_pull_requests_reports_graphql_errors() {
        let error = parse_pull_requests(ERRORS_RESPONSE).expect_err("should fail");

        assert_eq!(
            error,
            GithubError::Api("Field 'foo' doesn't exist".to_string())
        );
    }

    #[test]
    fn parse_pull_requests_reports_malformed_json() {
        let error = parse_pull_requests("not json").expect_err("should fail");

        assert!(matches!(error, GithubError::MalformedJson(_)));
    }

    #[test]
    fn parse_pull_requests_ignores_fields_it_does_not_know_about() {
        let pull_requests = parse_pull_requests(UNKNOWN_FIELD_RESPONSE).expect("should parse");

        assert!(pull_requests.is_empty());
    }
}

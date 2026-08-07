//! Building the `gh api graphql` invocation for a pull request search.

/// The GraphQL document sent to `gh api graphql`. Verified against gh
/// 2.97.0: `search` with `type: ISSUE` returns pull requests (and other
/// issue-shaped nodes, which the caller filters out) matching the search
/// syntax in `q`.
const SEARCH_DOCUMENT: &str = r#"query($q: String!, $limit: Int!) {
  search(query: $q, type: ISSUE, first: $limit) {
    issueCount
    nodes {
      ... on PullRequest {
        number
        title
        url
        createdAt
        isDraft
        totalCommentsCount
        reviewDecision
        repository { nameWithOwner }
        author { login }
        commits(last: 1) { nodes { commit { statusCheckRollup { state } } } }
      }
    }
  }
}"#;

/// How many pull requests to ask GitHub for, per query.
pub const DEFAULT_LIMIT: u32 = 30;

/// One of the cross-repository pull request searches the app runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PullRequestQuery {
    /// Open pull requests authored by the signed-in user.
    Authored,
    /// Open pull requests awaiting the signed-in user's review.
    ReviewRequested,
}

impl PullRequestQuery {
    /// The GitHub search syntax filter. Deliberately unscoped by
    /// organisation: it matches every repository the signed-in user can see.
    pub fn search_filter(self) -> &'static str {
        match self {
            Self::Authored => "is:open is:pr author:@me archived:false",
            Self::ReviewRequested => "is:open is:pr review-requested:@me archived:false",
        }
    }

    /// The arguments to pass to `gh`, using [`DEFAULT_LIMIT`].
    pub fn gh_args(self) -> Vec<String> {
        self.gh_args_with_limit(DEFAULT_LIMIT)
    }

    /// The arguments to pass to `gh`, asking for at most `limit` results.
    pub fn gh_args_with_limit(self, limit: u32) -> Vec<String> {
        vec![
            "api".to_string(),
            "graphql".to_string(),
            // -F gives `limit` an Int, which `$limit: Int!` requires.
            "-F".to_string(),
            format!("q={}", self.search_filter()),
            "-F".to_string(),
            format!("limit={limit}"),
            // -f keeps the document a raw string.
            "-f".to_string(),
            format!("query={SEARCH_DOCUMENT}"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_searches_open_pull_requests_by_the_current_user() {
        assert_eq!(
            PullRequestQuery::Authored.search_filter(),
            "is:open is:pr author:@me archived:false"
        );
    }

    #[test]
    fn review_requested_searches_open_pull_requests_awaiting_the_current_user() {
        assert_eq!(
            PullRequestQuery::ReviewRequested.search_filter(),
            "is:open is:pr review-requested:@me archived:false"
        );
    }

    #[test]
    fn gh_args_builds_the_verified_graphql_invocation() {
        let args = PullRequestQuery::Authored.gh_args();

        assert_eq!(args[0], "api");
        assert_eq!(args[1], "graphql");
        assert_eq!(args[2], "-F");
        assert_eq!(args[3], "q=is:open is:pr author:@me archived:false");
        assert_eq!(args[4], "-F");
        assert_eq!(args[5], "limit=30");
        assert_eq!(args[6], "-f");
        assert!(args[7].starts_with("query=query($q: String!"));
        assert_eq!(args.len(), 8);
    }

    #[test]
    fn gh_args_with_limit_uses_the_requested_limit() {
        let args = PullRequestQuery::ReviewRequested.gh_args_with_limit(5);

        assert_eq!(args[5], "limit=5");
    }
}

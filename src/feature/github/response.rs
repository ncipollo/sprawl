//! The wire shape of `gh api graphql`'s response. Kept separate from the
//! domain model in `pull_request` because every field here can be `null`,
//! and the domain expresses that nullability semantically instead.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GraphQlResponse {
    pub data: Option<ResponseData>,
    #[serde(default)]
    pub errors: Vec<GraphQlError>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQlError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ResponseData {
    pub search: SearchResults,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    #[serde(default)]
    pub issue_count: u32,
    #[serde(default)]
    pub nodes: Vec<PullRequestNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestNode {
    pub number: Option<u64>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub created_at: Option<String>,
    #[serde(default)]
    pub is_draft: bool,
    #[serde(default)]
    pub total_comments_count: u32,
    pub review_decision: Option<String>,
    pub repository: Option<RepositoryNode>,
    pub author: Option<AuthorNode>,
    pub commits: Option<CommitConnection>,
}

impl PullRequestNode {
    /// The rollup state of the newest commit, if the pull request has one
    /// with checks. Keeps the four-level
    /// `commits.nodes[0].commit.statusCheckRollup.state` walk out of the
    /// mapping code.
    pub fn checks_state(&self) -> Option<&str> {
        self.commits
            .as_ref()?
            .nodes
            .first()?
            .commit
            .as_ref()?
            .status_check_rollup
            .as_ref()?
            .state
            .as_deref()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryNode {
    pub name_with_owner: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthorNode {
    pub login: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommitConnection {
    #[serde(default)]
    pub nodes: Vec<CommitNode>,
}

#[derive(Debug, Deserialize)]
pub struct CommitNode {
    pub commit: Option<Commit>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub status_check_rollup: Option<StatusCheckRollup>,
}

#[derive(Debug, Deserialize)]
pub struct StatusCheckRollup {
    pub state: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_state_reads_the_rollup_of_the_newest_commit() {
        let node = PullRequestNode {
            number: None,
            title: None,
            url: None,
            created_at: None,
            is_draft: false,
            total_comments_count: 0,
            review_decision: None,
            repository: None,
            author: None,
            commits: Some(CommitConnection {
                nodes: vec![CommitNode {
                    commit: Some(Commit {
                        status_check_rollup: Some(StatusCheckRollup {
                            state: Some("SUCCESS".to_string()),
                        }),
                    }),
                }],
            }),
        };

        assert_eq!(node.checks_state(), Some("SUCCESS"));
    }

    #[test]
    fn checks_state_is_none_when_there_are_no_commits() {
        let node = PullRequestNode {
            number: None,
            title: None,
            url: None,
            created_at: None,
            is_draft: false,
            total_comments_count: 0,
            review_decision: None,
            repository: None,
            author: None,
            commits: Some(CommitConnection { nodes: vec![] }),
        };

        assert_eq!(node.checks_state(), None);
    }
}

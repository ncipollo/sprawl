//! The domain model for a pull request, independent of how it was fetched.

/// An open pull request, as shown in the content pane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub url: String,
    /// The repository the pull request belongs to, e.g. `"WhoopInc/android"`.
    pub repository: String,
    /// `None` when the author's account has been deleted.
    pub author: Option<String>,
    /// The raw `createdAt` value from GitHub: a fixed-width UTC RFC 3339
    /// timestamp, e.g. `"2026-08-07T17:42:31Z"`.
    pub created_at: String,
    pub is_draft: bool,
    pub comment_count: u32,
    pub review: ReviewStatus,
    pub checks: ChecksStatus,
}

/// The state of review on a pull request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewStatus {
    Approved,
    ChangesRequested,
    ReviewRequired,
    /// No review is required, or none has been requested.
    None,
}

impl ReviewStatus {
    /// Maps GitHub's `reviewDecision`, which is `null` when no review is
    /// required.
    pub fn from_api(decision: Option<&str>) -> Self {
        match decision {
            Some("APPROVED") => Self::Approved,
            Some("CHANGES_REQUESTED") => Self::ChangesRequested,
            Some("REVIEW_REQUIRED") => Self::ReviewRequired,
            _ => Self::None,
        }
    }

    /// A short, human-readable label for this status.
    pub fn label(self) -> &'static str {
        match self {
            Self::Approved => "Approved",
            Self::ChangesRequested => "Changes requested",
            Self::ReviewRequired => "Review required",
            Self::None => "No review",
        }
    }
}

/// The state of CI checks on a pull request's latest commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChecksStatus {
    Passing,
    Failing,
    Pending,
    /// The pull request has no checks configured.
    Unknown,
}

impl ChecksStatus {
    /// Maps `commits.nodes[0].commit.statusCheckRollup.state`, which is
    /// `null` when the pull request has no checks.
    pub fn from_api(state: Option<&str>) -> Self {
        match state {
            Some("SUCCESS") => Self::Passing,
            Some("FAILURE" | "ERROR") => Self::Failing,
            Some("PENDING" | "EXPECTED") => Self::Pending,
            _ => Self::Unknown,
        }
    }

    /// A short, human-readable label for this status.
    pub fn label(self) -> &'static str {
        match self {
            Self::Passing => "Checks passing",
            Self::Failing => "Checks failing",
            Self::Pending => "Checks running",
            Self::Unknown => "No checks",
        }
    }
}

/// Sorts `pull_requests` newest first.
///
/// GitHub's `createdAt` is a fixed-width UTC RFC 3339 timestamp: the same
/// offset (`Z`), zero-padded fields, and no fractional seconds. For that
/// shape a lexicographic comparison is also a chronological one, so no date
/// library is needed here. The sort is stable, so pull requests created in
/// the same second keep the order GitHub returned them in.
pub fn sort_newest_first(pull_requests: &mut [PullRequest]) {
    pull_requests.sort_by(|a, b| b.created_at.cmp(&a.created_at));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pull_request(number: u64, created_at: &str) -> PullRequest {
        PullRequest {
            number,
            title: format!("pr {number}"),
            url: format!("https://github.com/o/r/pull/{number}"),
            repository: "o/r".to_string(),
            author: Some("someone".to_string()),
            created_at: created_at.to_string(),
            is_draft: false,
            comment_count: 0,
            review: ReviewStatus::None,
            checks: ChecksStatus::Unknown,
        }
    }

    #[test]
    fn review_status_maps_every_documented_decision() {
        assert_eq!(
            ReviewStatus::from_api(Some("APPROVED")),
            ReviewStatus::Approved
        );
        assert_eq!(
            ReviewStatus::from_api(Some("CHANGES_REQUESTED")),
            ReviewStatus::ChangesRequested
        );
        assert_eq!(
            ReviewStatus::from_api(Some("REVIEW_REQUIRED")),
            ReviewStatus::ReviewRequired
        );
    }

    #[test]
    fn review_status_is_none_when_github_returns_null() {
        assert_eq!(ReviewStatus::from_api(None), ReviewStatus::None);
    }

    #[test]
    fn checks_status_maps_success_failure_error_pending_and_expected() {
        assert_eq!(
            ChecksStatus::from_api(Some("SUCCESS")),
            ChecksStatus::Passing
        );
        assert_eq!(
            ChecksStatus::from_api(Some("FAILURE")),
            ChecksStatus::Failing
        );
        assert_eq!(ChecksStatus::from_api(Some("ERROR")), ChecksStatus::Failing);
        assert_eq!(
            ChecksStatus::from_api(Some("PENDING")),
            ChecksStatus::Pending
        );
        assert_eq!(
            ChecksStatus::from_api(Some("EXPECTED")),
            ChecksStatus::Pending
        );
    }

    #[test]
    fn checks_status_is_unknown_when_there_is_no_rollup() {
        assert_eq!(ChecksStatus::from_api(None), ChecksStatus::Unknown);
    }

    #[test]
    fn label_returns_the_expected_display_string() {
        assert_eq!(ReviewStatus::Approved.label(), "Approved");
        assert_eq!(ReviewStatus::ChangesRequested.label(), "Changes requested");
        assert_eq!(ReviewStatus::ReviewRequired.label(), "Review required");
        assert_eq!(ReviewStatus::None.label(), "No review");
        assert_eq!(ChecksStatus::Passing.label(), "Checks passing");
        assert_eq!(ChecksStatus::Failing.label(), "Checks failing");
        assert_eq!(ChecksStatus::Pending.label(), "Checks running");
        assert_eq!(ChecksStatus::Unknown.label(), "No checks");
    }

    #[test]
    fn sort_newest_first_puts_the_newest_pull_request_first() {
        let mut pull_requests = vec![
            pull_request(1, "2026-08-01T09:00:00Z"),
            pull_request(2, "2026-08-07T17:42:31Z"),
        ];

        sort_newest_first(&mut pull_requests);

        assert_eq!(pull_requests[0].number, 2);
        assert_eq!(pull_requests[1].number, 1);
    }

    #[test]
    fn sort_newest_first_orders_correctly_across_a_year_boundary() {
        let mut pull_requests = vec![
            pull_request(1, "2025-12-31T23:59:59Z"),
            pull_request(2, "2026-01-01T00:00:00Z"),
        ];

        sort_newest_first(&mut pull_requests);

        assert_eq!(pull_requests[0].number, 2);
        assert_eq!(pull_requests[1].number, 1);
    }

    #[test]
    fn sort_newest_first_keeps_github_order_for_identical_timestamps() {
        let mut pull_requests = vec![
            pull_request(1, "2026-08-07T17:42:31Z"),
            pull_request(2, "2026-08-07T17:42:31Z"),
        ];

        sort_newest_first(&mut pull_requests);

        assert_eq!(pull_requests[0].number, 1);
        assert_eq!(pull_requests[1].number, 2);
    }
}

//! The hard-coded list of navigation sections shown in the sidebar.

use crate::feature::github::query::PullRequestQuery;

/// A navigable section of the app, listed in the sidebar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    MyPrs,
    NeedsMyReview,
    TicketsAssignedToMe,
}

impl Section {
    /// All sections, in the order they should appear in the sidebar.
    pub fn all() -> Vec<Section> {
        vec![
            Section::MyPrs,
            Section::NeedsMyReview,
            Section::TicketsAssignedToMe,
        ]
    }

    /// The display title for this section.
    pub fn title(self) -> &'static str {
        match self {
            Section::MyPrs => "My PRs",
            Section::NeedsMyReview => "Needs My Review",
            Section::TicketsAssignedToMe => "Tickets Assigned to Me",
        }
    }

    /// The pull request search backing this section, if it shows pull
    /// requests.
    pub fn pull_request_query(self) -> Option<PullRequestQuery> {
        match self {
            Section::MyPrs => Some(PullRequestQuery::Authored),
            Section::NeedsMyReview => Some(PullRequestQuery::ReviewRequested),
            Section::TicketsAssignedToMe => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_sections_in_sidebar_order() {
        assert_eq!(
            Section::all(),
            vec![
                Section::MyPrs,
                Section::NeedsMyReview,
                Section::TicketsAssignedToMe
            ]
        );
    }

    #[test]
    fn title_returns_the_expected_display_string() {
        assert_eq!(Section::MyPrs.title(), "My PRs");
        assert_eq!(Section::NeedsMyReview.title(), "Needs My Review");
        assert_eq!(
            Section::TicketsAssignedToMe.title(),
            "Tickets Assigned to Me"
        );
    }

    #[test]
    fn pull_request_query_maps_the_pull_request_sections() {
        assert_eq!(
            Section::MyPrs.pull_request_query(),
            Some(PullRequestQuery::Authored)
        );
        assert_eq!(
            Section::NeedsMyReview.pull_request_query(),
            Some(PullRequestQuery::ReviewRequested)
        );
    }

    #[test]
    fn pull_request_query_is_none_for_the_tickets_section() {
        assert_eq!(Section::TicketsAssignedToMe.pull_request_query(), None);
    }
}

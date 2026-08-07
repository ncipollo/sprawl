//! Pure mapping from pull request status to a tile badge's colour and
//! label. Kept separate from the view so it's testable without gpui.

use crate::feature::github::pull_request::{ChecksStatus, ReviewStatus};
use crate::ui::colors;
use crate::ui::components::tile::TileBadge;

/// A badge showing whether the pull request has been reviewed.
pub fn review_badge(status: ReviewStatus) -> TileBadge {
    let color = match status {
        ReviewStatus::Approved => colors::SUCCESS,
        ReviewStatus::ChangesRequested => colors::DANGER,
        ReviewStatus::ReviewRequired => colors::WARNING,
        ReviewStatus::None => colors::NEUTRAL,
    };
    TileBadge::new(color, status.label())
}

/// A badge showing the state of CI checks.
pub fn checks_badge(status: ChecksStatus) -> TileBadge {
    let color = match status {
        ChecksStatus::Passing => colors::SUCCESS,
        ChecksStatus::Failing => colors::DANGER,
        ChecksStatus::Pending => colors::WARNING,
        ChecksStatus::Unknown => colors::NEUTRAL,
    };
    TileBadge::new(color, status.label())
}

/// A badge showing the comment count, singular or plural.
pub fn comment_badge(comment_count: u32) -> TileBadge {
    let label = if comment_count == 1 {
        "1 comment".to_string()
    } else {
        format!("{comment_count} comments")
    };
    TileBadge::new(colors::NEUTRAL, label)
}

/// A badge marking the pull request as a draft.
pub fn draft_badge() -> TileBadge {
    TileBadge::new(colors::NEUTRAL, "Draft")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_badge_is_green_when_approved() {
        let badge = review_badge(ReviewStatus::Approved);

        assert_eq!(badge.color(), colors::SUCCESS);
        assert_eq!(badge.label(), "Approved");
    }

    #[test]
    fn review_badge_is_red_when_changes_are_requested() {
        let badge = review_badge(ReviewStatus::ChangesRequested);

        assert_eq!(badge.color(), colors::DANGER);
        assert_eq!(badge.label(), "Changes requested");
    }

    #[test]
    fn review_badge_is_amber_when_a_review_is_required() {
        let badge = review_badge(ReviewStatus::ReviewRequired);

        assert_eq!(badge.color(), colors::WARNING);
        assert_eq!(badge.label(), "Review required");
    }

    #[test]
    fn review_badge_reports_that_no_review_is_needed() {
        let badge = review_badge(ReviewStatus::None);

        assert_eq!(badge.color(), colors::NEUTRAL);
        assert_eq!(badge.label(), "No review");
    }

    #[test]
    fn checks_badge_maps_each_status_to_a_colour_and_label() {
        assert_eq!(checks_badge(ChecksStatus::Passing).color(), colors::SUCCESS);
        assert_eq!(checks_badge(ChecksStatus::Failing).color(), colors::DANGER);
        assert_eq!(checks_badge(ChecksStatus::Pending).color(), colors::WARNING);
        assert_eq!(checks_badge(ChecksStatus::Unknown).color(), colors::NEUTRAL);
        assert_eq!(
            checks_badge(ChecksStatus::Passing).label(),
            "Checks passing"
        );
    }

    #[test]
    fn comment_badge_uses_the_singular_for_one_comment() {
        assert_eq!(comment_badge(1).label(), "1 comment");
    }

    #[test]
    fn comment_badge_uses_the_plural_for_no_comments() {
        assert_eq!(comment_badge(0).label(), "0 comments");
    }
}

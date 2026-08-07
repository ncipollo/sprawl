//! The hard-coded list of navigation sections shown in the sidebar.

/// A navigable section of the app, listed in the sidebar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    MyPrs,
    OtherPrs,
    TicketsAssignedToMe,
}

impl Section {
    /// All sections, in the order they should appear in the sidebar.
    pub fn all() -> Vec<Section> {
        vec![
            Section::MyPrs,
            Section::OtherPrs,
            Section::TicketsAssignedToMe,
        ]
    }

    /// The display title for this section.
    pub fn title(self) -> &'static str {
        match self {
            Section::MyPrs => "My PRs",
            Section::OtherPrs => "Other PRs",
            Section::TicketsAssignedToMe => "Tickets Assigned to Me",
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
                Section::OtherPrs,
                Section::TicketsAssignedToMe
            ]
        );
    }

    #[test]
    fn title_returns_the_expected_display_string() {
        assert_eq!(Section::MyPrs.title(), "My PRs");
        assert_eq!(Section::OtherPrs.title(), "Other PRs");
        assert_eq!(
            Section::TicketsAssignedToMe.title(),
            "Tickets Assigned to Me"
        );
    }
}

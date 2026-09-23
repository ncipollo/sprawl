//! The example scripts scaffolded on first run. They are the single source
//! of truth: written to `~/.sprawl/default`, embedded in `--info example`,
//! and evaluated by the real engine in tests.

/// The shared shape of the two GitHub pull request scripts. `__TITLE__` and
/// `__FILTER__` are substituted per script.
const PULL_REQUEST_TEMPLATE: &str = r#"// A sprawl section: open GitHub pull requests, via the `gh` CLI.
// The script's final expression is the section object.

const FILTER = "__FILTER__";
const QUERY = `query($q: String!, $limit: Int!) {
  search(query: $q, type: ISSUE, first: $limit) {
    nodes {
      ... on PullRequest {
        number title url createdAt isDraft totalCommentsCount reviewDecision
        repository { nameWithOwner }
        commits(last: 1) { nodes { commit { statusCheckRollup { state } } } }
      }
    }
  }
}`;

function reviewBadge(decision) {
  switch (decision) {
    case "APPROVED": return { label: "Approved", color: "success" };
    case "CHANGES_REQUESTED": return { label: "Changes requested", color: "danger" };
    case "REVIEW_REQUIRED": return { label: "Review required", color: "warning" };
    default: return { label: "No review", color: "neutral" };
  }
}

function checksBadge(pr) {
  const commit = pr.commits.nodes[0];
  const rollup = commit && commit.commit.statusCheckRollup;
  const state = rollup && rollup.state;
  switch (state) {
    case "SUCCESS": return { label: "Checks passing", color: "success" };
    case "FAILURE":
    case "ERROR": return { label: "Checks failing", color: "danger" };
    case "PENDING":
    case "EXPECTED": return { label: "Checks running", color: "warning" };
    default: return { label: "No checks", color: "neutral" };
  }
}

function badges(pr) {
  const count = pr.totalCommentsCount;
  const list = [
    { label: count === 1 ? "1 comment" : `${count} comments`, color: "neutral" },
    reviewBadge(pr.reviewDecision),
    checksBadge(pr),
  ];
  if (pr.isDraft) list.push({ label: "Draft", color: "neutral" });
  return list;
}

function tile(pr) {
  return {
    type: "tile",
    title: pr.title,
    subtitle: `${pr.repository.nameWithOwner} #${pr.number}`,
    badges: badges(pr),
    url: pr.url,
  };
}

const response = JSON.parse(shell("gh", [
  "api", "graphql",
  "-F", `q=${FILTER}`,
  "-F", "limit=30",
  "-f", `query=${QUERY}`,
]));
if (response.errors && response.errors.length > 0) {
  throw new Error(response.errors.map((e) => e.message).join("; "));
}
const items = response.data.search.nodes
  .filter((node) => node.title !== undefined)
  .sort((a, b) => b.createdAt.localeCompare(a.createdAt))
  .map(tile);

({ title: "__TITLE__", items });
"#;

/// The `my_prs.js` example: open pull requests authored by the signed-in
/// user.
pub fn my_prs() -> String {
    render("My PRs", "is:open is:pr author:@me archived:false")
}

/// The `needs_my_review.js` example: open pull requests awaiting the
/// signed-in user's review.
pub fn needs_my_review() -> String {
    render(
        "Needs My Review",
        "is:open is:pr review-requested:@me archived:false",
    )
}

fn render(title: &str, filter: &str) -> String {
    PULL_REQUEST_TEMPLATE
        .replace("__TITLE__", title)
        .replace("__FILTER__", filter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::script;
    use crate::feature::script::schema::{BadgeColor, SectionConfig, SectionItem};
    use crate::feature::script::shell::ShellInvocation;
    use crate::feature::script::shell::fake::FakeShell;
    use std::sync::Arc;

    /// A realistic `gh api graphql` response: a full node, a draft node
    /// with no checks, and an empty non-pull-request node.
    const GH_RESPONSE: &str = r#"{"data":{"search":{"issueCount":2,"nodes":[
      {"number":12,"title":"Older change","url":"https://github.com/ncipollo/sprawl/pull/12",
       "createdAt":"2026-08-01T09:00:00Z","isDraft":true,"totalCommentsCount":1,"reviewDecision":null,
       "repository":{"nameWithOwner":"ncipollo/sprawl"},
       "commits":{"nodes":[]}},
      {"number":19948,"title":"Show template deserialization errors","url":"https://github.com/WhoopInc/android/pull/19948",
       "createdAt":"2026-08-07T17:42:31Z","isDraft":false,"totalCommentsCount":3,"reviewDecision":"APPROVED",
       "repository":{"nameWithOwner":"WhoopInc/android"},
       "commits":{"nodes":[{"commit":{"statusCheckRollup":{"state":"FAILURE"}}}]}},
      {}
    ]}}}"#;

    fn run(source: &str, shell: &Arc<FakeShell>) -> SectionConfig {
        script::evaluate(source, shell.clone()).expect("the example should run")
    }

    #[test]
    fn my_prs_searches_for_authored_pull_requests() {
        let shell = Arc::new(FakeShell::outputting(GH_RESPONSE));

        let config = run(&my_prs(), &shell);

        assert_eq!(config.title, "My PRs");
        let ShellInvocation::Program { program, args } = &shell.invocations()[0] else {
            panic!("the example should use the argv form");
        };
        assert_eq!(program, "gh");
        assert!(args.contains(&"q=is:open is:pr author:@me archived:false".to_string()));
    }

    #[test]
    fn needs_my_review_searches_for_review_requests() {
        let shell = Arc::new(FakeShell::outputting(GH_RESPONSE));

        let config = run(&needs_my_review(), &shell);

        assert_eq!(config.title, "Needs My Review");
        let ShellInvocation::Program { args, .. } = &shell.invocations()[0] else {
            panic!("the example should use the argv form");
        };
        assert!(args.contains(&"q=is:open is:pr review-requested:@me archived:false".to_string()));
    }

    #[test]
    fn the_examples_map_pull_requests_to_tiles_newest_first() {
        let shell = Arc::new(FakeShell::outputting(GH_RESPONSE));

        let config = run(&my_prs(), &shell);

        assert_eq!(config.items.len(), 2);
        let SectionItem::Tile(newest) = &config.items[0] else {
            panic!("expected a tile")
        };
        assert_eq!(newest.title, "Show template deserialization errors");
        assert_eq!(newest.subtitle, "WhoopInc/android #19948");
        assert_eq!(
            newest.url.as_deref(),
            Some("https://github.com/WhoopInc/android/pull/19948")
        );
    }

    #[test]
    fn the_examples_reproduce_the_badge_mapping() {
        let shell = Arc::new(FakeShell::outputting(GH_RESPONSE));

        let config = run(&my_prs(), &shell);

        let SectionItem::Tile(newest) = &config.items[0] else {
            panic!("expected a tile")
        };
        let labels: Vec<(&str, BadgeColor)> = newest
            .badges
            .iter()
            .map(|badge| (badge.label.as_str(), badge.color))
            .collect();
        assert_eq!(
            labels,
            vec![
                ("3 comments", BadgeColor::Neutral),
                ("Approved", BadgeColor::Success),
                ("Checks failing", BadgeColor::Danger),
            ]
        );
    }

    #[test]
    fn a_draft_with_no_checks_gets_the_draft_and_no_checks_badges() {
        let shell = Arc::new(FakeShell::outputting(GH_RESPONSE));

        let config = run(&my_prs(), &shell);

        let SectionItem::Tile(draft) = &config.items[1] else {
            panic!("expected a tile")
        };
        let labels: Vec<&str> = draft.badges.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(labels, vec!["1 comment", "No review", "No checks", "Draft"]);
    }

    #[test]
    fn a_graphql_error_makes_the_example_throw() {
        let shell = Arc::new(FakeShell::outputting(
            r#"{"data":null,"errors":[{"message":"Field 'foo' doesn't exist"}]}"#,
        ));

        let error = script::evaluate(&my_prs(), shell).expect_err("should fail");

        assert!(error.to_string().contains("Field 'foo' doesn't exist"));
    }

    #[test]
    fn an_empty_search_produces_an_empty_section() {
        let shell = Arc::new(FakeShell::outputting(
            r#"{"data":{"search":{"issueCount":0,"nodes":[]}}}"#,
        ));

        let config = run(&my_prs(), &shell);

        assert!(config.items.is_empty());
    }
}

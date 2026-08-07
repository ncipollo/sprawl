//! The stale-while-revalidate policy for pull request fetches: request
//! de-duplication, error retention, and what the content pane should show.
//! Everything here is gpui-free and driven by an injected [`Clock`], so the
//! whole policy is unit tested without booting an `App`.

use crate::feature::clock::{Clock, SystemClock};
use crate::feature::github::cache::{CacheState, DEFAULT_TTL, PullRequestCache};
use crate::feature::github::error::GithubError;
use crate::feature::github::pull_request::PullRequest;
use crate::feature::github::query::PullRequestQuery;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// What the caller should do after a query becomes visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchDecision {
    /// Nothing to do: the cached result is fresh, or a fetch is already
    /// running.
    Idle,
    /// Fetch; there is nothing cached to show in the meantime.
    Fetch(PullRequestQuery),
    /// Fetch in the background and keep showing the cached result.
    Refresh(PullRequestQuery),
}

/// What the content pane should show for a query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayState {
    Loading,
    Failed,
    Empty,
    Populated,
}

/// Owns the cached pull requests for every query, which fetches are running,
/// and the last error for each query.
pub struct PullRequestStore {
    cache: PullRequestCache,
    loading: HashSet<PullRequestQuery>,
    errors: HashMap<PullRequestQuery, GithubError>,
    clock: Box<dyn Clock>,
}

impl PullRequestStore {
    pub fn new() -> Self {
        Self::with_ttl_and_clock(DEFAULT_TTL, Box::new(SystemClock))
    }

    pub fn with_ttl_and_clock(ttl: Duration, clock: Box<dyn Clock>) -> Self {
        Self {
            cache: PullRequestCache::with_ttl(ttl),
            loading: HashSet::new(),
            errors: HashMap::new(),
            clock,
        }
    }

    /// Records that `query` has become visible and reports what to do. A
    /// stale result is refreshed in the background while it stays on
    /// screen.
    pub fn visit(&mut self, query: PullRequestQuery) -> FetchDecision {
        if self.loading.contains(&query) {
            return FetchDecision::Idle;
        }
        let decision = match self.cache.state(query, self.clock.now()) {
            CacheState::Fresh => return FetchDecision::Idle,
            CacheState::Stale => FetchDecision::Refresh(query),
            CacheState::Missing => FetchDecision::Fetch(query),
        };
        self.loading.insert(query);
        decision
    }

    /// Records the outcome of a fetch started by [`Self::visit`]. A failed
    /// refresh keeps any cached result, so a network blip does not empty
    /// the pane.
    pub fn finish_fetch(
        &mut self,
        query: PullRequestQuery,
        result: Result<Vec<PullRequest>, GithubError>,
    ) {
        self.loading.remove(&query);
        match result {
            Ok(pull_requests) => {
                self.cache.store(query, pull_requests, self.clock.now());
                self.errors.remove(&query);
            }
            Err(error) => {
                self.errors.insert(query, error);
            }
        }
    }

    /// The cached pull requests for `query`, or an empty slice.
    pub fn pull_requests(&self, query: PullRequestQuery) -> &[PullRequest] {
        self.cache.pull_requests(query)
    }

    /// The last error recorded for `query`, if any.
    pub fn error(&self, query: PullRequestQuery) -> Option<&GithubError> {
        self.errors.get(&query)
    }

    /// Whether a cached result is on screen while a newer one is fetched.
    pub fn is_refreshing(&self, query: PullRequestQuery) -> bool {
        self.loading.contains(&query) && self.cache.contains(query)
    }

    /// What the content pane should show for `query`. A cached result
    /// always wins over `Loading`/`Failed` — that is what makes this
    /// stale-while-revalidate.
    pub fn display_state(&self, query: PullRequestQuery) -> DisplayState {
        if self.cache.contains(query) {
            if self.cache.pull_requests(query).is_empty() {
                return DisplayState::Empty;
            }
            return DisplayState::Populated;
        }
        if self.errors.contains_key(&query) && !self.loading.contains(&query) {
            return DisplayState::Failed;
        }
        DisplayState::Loading
    }
}

impl Default for PullRequestStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::clock::fake::FixedClock;

    fn store_with_clock(clock: FixedClock) -> PullRequestStore {
        PullRequestStore::with_ttl_and_clock(Duration::from_secs(15 * 60), Box::new(clock))
    }

    #[test]
    fn visit_fetches_when_nothing_is_cached() {
        let mut store = store_with_clock(FixedClock::new());

        let decision = store.visit(PullRequestQuery::Authored);

        assert_eq!(decision, FetchDecision::Fetch(PullRequestQuery::Authored));
    }

    #[test]
    fn visit_is_idle_while_a_fetch_is_already_running() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(PullRequestQuery::Authored);

        let decision = store.visit(PullRequestQuery::Authored);

        assert_eq!(decision, FetchDecision::Idle);
    }

    #[test]
    fn visit_is_idle_while_the_cached_result_is_fresh() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(PullRequestQuery::Authored);
        store.finish_fetch(PullRequestQuery::Authored, Ok(Vec::new()));

        let decision = store.visit(PullRequestQuery::Authored);

        assert_eq!(decision, FetchDecision::Idle);
    }

    #[test]
    fn visit_refreshes_once_the_time_to_live_has_elapsed() {
        let clock = FixedClock::new();
        let mut store = store_with_clock(clock.clone());
        store.visit(PullRequestQuery::Authored);
        store.finish_fetch(PullRequestQuery::Authored, Ok(Vec::new()));

        clock.advance(Duration::from_secs(16 * 60));
        let decision = store.visit(PullRequestQuery::Authored);

        assert_eq!(decision, FetchDecision::Refresh(PullRequestQuery::Authored));
    }

    #[test]
    fn finish_fetch_stores_the_result_and_stops_loading() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(PullRequestQuery::Authored);

        store.finish_fetch(PullRequestQuery::Authored, Ok(Vec::new()));

        assert_eq!(store.visit(PullRequestQuery::Authored), FetchDecision::Idle);
    }

    #[test]
    fn finish_fetch_keeps_the_cached_result_when_a_refresh_fails() {
        let clock = FixedClock::new();
        let mut store = store_with_clock(clock.clone());
        store.visit(PullRequestQuery::Authored);
        store.finish_fetch(PullRequestQuery::Authored, Ok(vec![placeholder()]));
        clock.advance(Duration::from_secs(16 * 60));
        store.visit(PullRequestQuery::Authored);

        store.finish_fetch(
            PullRequestQuery::Authored,
            Err(GithubError::NotAuthenticated),
        );

        assert_eq!(
            store.display_state(PullRequestQuery::Authored),
            DisplayState::Populated
        );
        assert_eq!(store.pull_requests(PullRequestQuery::Authored).len(), 1);
    }

    #[test]
    fn finish_fetch_clears_an_error_recorded_by_an_earlier_attempt() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(PullRequestQuery::Authored);
        store.finish_fetch(
            PullRequestQuery::Authored,
            Err(GithubError::NotAuthenticated),
        );

        store.finish_fetch(PullRequestQuery::Authored, Ok(Vec::new()));

        assert_eq!(store.error(PullRequestQuery::Authored), None);
    }

    #[test]
    fn display_state_is_loading_before_the_first_result_arrives() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(PullRequestQuery::Authored);

        assert_eq!(
            store.display_state(PullRequestQuery::Authored),
            DisplayState::Loading
        );
    }

    #[test]
    fn display_state_is_failed_when_the_first_fetch_fails() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(PullRequestQuery::Authored);

        store.finish_fetch(
            PullRequestQuery::Authored,
            Err(GithubError::NotAuthenticated),
        );

        assert_eq!(
            store.display_state(PullRequestQuery::Authored),
            DisplayState::Failed
        );
    }

    #[test]
    fn display_state_is_empty_when_github_returns_no_pull_requests() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(PullRequestQuery::Authored);

        store.finish_fetch(PullRequestQuery::Authored, Ok(Vec::new()));

        assert_eq!(
            store.display_state(PullRequestQuery::Authored),
            DisplayState::Empty
        );
    }

    #[test]
    fn display_state_is_populated_while_a_stale_result_is_refreshed() {
        let clock = FixedClock::new();
        let mut store = store_with_clock(clock.clone());
        store.visit(PullRequestQuery::Authored);
        store.finish_fetch(PullRequestQuery::Authored, Ok(vec![placeholder()]));
        clock.advance(Duration::from_secs(16 * 60));

        store.visit(PullRequestQuery::Authored);

        assert_eq!(
            store.display_state(PullRequestQuery::Authored),
            DisplayState::Populated
        );
        assert!(store.is_refreshing(PullRequestQuery::Authored));
    }

    #[test]
    fn is_refreshing_is_false_when_there_is_nothing_cached_to_show() {
        let mut store = store_with_clock(FixedClock::new());

        store.visit(PullRequestQuery::Authored);

        assert!(!store.is_refreshing(PullRequestQuery::Authored));
    }

    fn placeholder() -> PullRequest {
        use crate::feature::github::pull_request::{ChecksStatus, ReviewStatus};

        PullRequest {
            number: 1,
            title: "title".to_string(),
            url: "https://github.com/o/r/pull/1".to_string(),
            repository: "o/r".to_string(),
            author: None,
            created_at: "2026-08-07T17:42:31Z".to_string(),
            is_draft: false,
            comment_count: 0,
            review: ReviewStatus::None,
            checks: ChecksStatus::Unknown,
        }
    }
}

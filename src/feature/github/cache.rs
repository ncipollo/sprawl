//! A per-query cache of fetched pull requests with a time to live. Pure: it
//! takes the current time as an argument, so it has no clock of its own to
//! fake.

use crate::feature::github::pull_request::PullRequest;
use crate::feature::github::query::PullRequestQuery;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How long a fetched result is considered fresh.
pub const DEFAULT_TTL: Duration = Duration::from_secs(15 * 60);

/// How usable a cached result is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheState {
    /// Nothing has been fetched for this query yet.
    Missing,
    /// Something was fetched, but longer than the time to live ago.
    Stale,
    /// Fetched within the time to live.
    Fresh,
}

struct Entry {
    pull_requests: Vec<PullRequest>,
    fetched_at: Instant,
}

/// Pull requests fetched per query, with a time to live.
pub struct PullRequestCache {
    ttl: Duration,
    entries: HashMap<PullRequestQuery, Entry>,
}

impl PullRequestCache {
    pub fn new() -> Self {
        Self::with_ttl(DEFAULT_TTL)
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: HashMap::new(),
        }
    }

    /// How usable the cached result for `query` is, as of `now`.
    pub fn state(&self, query: PullRequestQuery, now: Instant) -> CacheState {
        match self.entries.get(&query) {
            None => CacheState::Missing,
            Some(entry) if now.duration_since(entry.fetched_at) < self.ttl => CacheState::Fresh,
            Some(_) => CacheState::Stale,
        }
    }

    /// Whether anything has been fetched for `query`, fresh or not.
    pub fn contains(&self, query: PullRequestQuery) -> bool {
        self.entries.contains_key(&query)
    }

    /// The cached pull requests, or an empty slice if nothing is cached.
    pub fn pull_requests(&self, query: PullRequestQuery) -> &[PullRequest] {
        self.entries
            .get(&query)
            .map_or(&[], |entry| &entry.pull_requests)
    }

    /// Replaces the cached result for `query`, fetched at `now`.
    pub fn store(
        &mut self,
        query: PullRequestQuery,
        pull_requests: Vec<PullRequest>,
        now: Instant,
    ) {
        self.entries.insert(
            query,
            Entry {
                pull_requests,
                fetched_at: now,
            },
        );
    }
}

impl Default for PullRequestCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_is_missing_before_anything_is_stored() {
        let cache = PullRequestCache::new();

        assert_eq!(
            cache.state(PullRequestQuery::Authored, Instant::now()),
            CacheState::Missing
        );
    }

    #[test]
    fn state_is_fresh_immediately_after_storing() {
        let mut cache = PullRequestCache::new();
        let now = Instant::now();

        cache.store(PullRequestQuery::Authored, Vec::new(), now);

        assert_eq!(
            cache.state(PullRequestQuery::Authored, now),
            CacheState::Fresh
        );
    }

    #[test]
    fn state_is_stale_once_the_time_to_live_has_elapsed() {
        let mut cache = PullRequestCache::with_ttl(Duration::from_secs(60));
        let now = Instant::now();
        cache.store(PullRequestQuery::Authored, Vec::new(), now);

        let state = cache.state(PullRequestQuery::Authored, now + Duration::from_secs(60));

        assert_eq!(state, CacheState::Stale);
    }

    #[test]
    fn state_is_fresh_one_moment_before_the_time_to_live_elapses() {
        let mut cache = PullRequestCache::with_ttl(Duration::from_secs(60));
        let now = Instant::now();
        cache.store(PullRequestQuery::Authored, Vec::new(), now);

        let state = cache.state(
            PullRequestQuery::Authored,
            now + Duration::from_millis(59_999),
        );

        assert_eq!(state, CacheState::Fresh);
    }

    #[test]
    fn pull_requests_returns_an_empty_slice_when_nothing_is_cached() {
        let cache = PullRequestCache::new();

        assert!(cache.pull_requests(PullRequestQuery::Authored).is_empty());
    }

    #[test]
    fn store_replaces_the_previous_result_for_a_query() {
        let mut cache = PullRequestCache::new();
        let now = Instant::now();
        cache.store(PullRequestQuery::Authored, vec![placeholder(1)], now);

        cache.store(PullRequestQuery::Authored, vec![placeholder(2)], now);

        assert_eq!(cache.pull_requests(PullRequestQuery::Authored)[0].number, 2);
    }

    #[test]
    fn each_query_is_cached_independently() {
        let mut cache = PullRequestCache::new();
        let now = Instant::now();
        cache.store(PullRequestQuery::Authored, vec![placeholder(1)], now);

        assert!(
            cache
                .pull_requests(PullRequestQuery::ReviewRequested)
                .is_empty()
        );
        assert_eq!(cache.pull_requests(PullRequestQuery::Authored)[0].number, 1);
    }

    fn placeholder(number: u64) -> PullRequest {
        use crate::feature::github::pull_request::{ChecksStatus, ReviewStatus};

        PullRequest {
            number,
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

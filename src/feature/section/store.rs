//! The stale-while-revalidate policy for section scripts: run
//! de-duplication, error retention, and what the content pane should show.
//! Everything here is gpui-free and driven by an injected [`Clock`], so the
//! whole policy is unit tested without booting an `App`.

use crate::feature::clock::{Clock, SystemClock};
use crate::feature::script::error::ScriptError;
use crate::feature::script::schema::SectionConfig;
use crate::feature::section::cache::{CacheState, DEFAULT_TTL, SectionCache};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What the caller should do after a section becomes visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchDecision {
    /// Nothing to do: the cached result is fresh, or the script is already
    /// running.
    Idle,
    /// Run the script; there is nothing cached to show in the meantime.
    Fetch,
    /// Run the script in the background and keep showing the cached result.
    Refresh,
}

/// What the content pane should show for a section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayState {
    Loading,
    Failed,
    Empty,
    Populated,
}

/// Owns the cached result for every section script, which scripts are
/// running, and the last error for each script.
pub struct SectionStore {
    cache: SectionCache,
    loading: HashSet<PathBuf>,
    errors: HashMap<PathBuf, ScriptError>,
    clock: Box<dyn Clock>,
}

impl SectionStore {
    pub fn new() -> Self {
        Self::with_ttl_and_clock(DEFAULT_TTL, Box::new(SystemClock))
    }

    pub fn with_ttl_and_clock(ttl: Duration, clock: Box<dyn Clock>) -> Self {
        Self {
            cache: SectionCache::with_ttl(ttl),
            loading: HashSet::new(),
            errors: HashMap::new(),
            clock,
        }
    }

    /// Records that the script at `path` has become visible and reports
    /// what to do. A stale result is refreshed in the background while it
    /// stays on screen.
    pub fn visit(&mut self, path: &Path) -> FetchDecision {
        if self.loading.contains(path) {
            return FetchDecision::Idle;
        }
        let decision = match self.cache.state(path, self.clock.now()) {
            CacheState::Fresh => return FetchDecision::Idle,
            CacheState::Stale => FetchDecision::Refresh,
            CacheState::Missing => FetchDecision::Fetch,
        };
        self.start_loading(path, decision)
    }

    /// Forces the script at `path` to re-run regardless of cache freshness.
    /// A run already in flight is left alone.
    pub fn force_refresh(&mut self, path: &Path) -> FetchDecision {
        if self.loading.contains(path) {
            return FetchDecision::Idle;
        }
        let decision = if self.cache.contains(path) {
            FetchDecision::Refresh
        } else {
            FetchDecision::Fetch
        };
        self.start_loading(path, decision)
    }

    fn start_loading(&mut self, path: &Path, decision: FetchDecision) -> FetchDecision {
        self.loading.insert(path.to_path_buf());
        decision
    }

    /// Records the outcome of a run started by [`Self::visit`]. A failed
    /// refresh keeps any cached result, so a script blip does not empty
    /// the pane.
    pub fn finish_fetch(&mut self, path: &Path, result: Result<SectionConfig, ScriptError>) {
        self.loading.remove(path);
        match result {
            Ok(config) => {
                self.cache
                    .store(path.to_path_buf(), config, self.clock.now());
                self.errors.remove(path);
            }
            Err(error) => {
                self.errors.insert(path.to_path_buf(), error);
            }
        }
    }

    /// The cached section for the script at `path`, if any.
    pub fn config(&self, path: &Path) -> Option<&SectionConfig> {
        self.cache.config(path)
    }

    /// The script-provided title, if the script has ever succeeded. It
    /// survives a failed refresh because the cached result is retained.
    pub fn title(&self, path: &Path) -> Option<&str> {
        self.config(path).map(|config| config.title.as_str())
    }

    /// The last error recorded for the script at `path`, if any.
    pub fn error(&self, path: &Path) -> Option<&ScriptError> {
        self.errors.get(path)
    }

    /// Whether a cached result is on screen while a newer one is computed.
    pub fn is_refreshing(&self, path: &Path) -> bool {
        self.loading.contains(path) && self.cache.contains(path)
    }

    /// What the content pane should show for the script at `path`. A cached
    /// result always wins over `Loading`/`Failed` — that is what makes this
    /// stale-while-revalidate.
    pub fn display_state(&self, path: &Path) -> DisplayState {
        if self.cache.contains(path) {
            if self
                .cache
                .config(path)
                .is_none_or(|config| config.items.is_empty())
            {
                return DisplayState::Empty;
            }
            return DisplayState::Populated;
        }
        if self.errors.contains_key(path) && !self.loading.contains(path) {
            return DisplayState::Failed;
        }
        DisplayState::Loading
    }
}

impl Default for SectionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::clock::fake::FixedClock;
    use crate::feature::script::schema::{SectionItem, TileItem};

    fn store_with_clock(clock: FixedClock) -> SectionStore {
        SectionStore::with_ttl_and_clock(Duration::from_secs(15 * 60), Box::new(clock))
    }

    fn path() -> PathBuf {
        PathBuf::from("/tmp/section.js")
    }

    fn empty_config() -> SectionConfig {
        SectionConfig {
            title: "Test".to_string(),
            items: Vec::new(),
        }
    }

    fn populated_config() -> SectionConfig {
        SectionConfig {
            title: "Test".to_string(),
            items: vec![SectionItem::Tile(TileItem {
                title: "tile".to_string(),
                subtitle: String::new(),
                badges: Vec::new(),
                url: None,
            })],
        }
    }

    fn error() -> ScriptError {
        ScriptError::Evaluate("kaboom".to_string())
    }

    #[test]
    fn visit_fetches_when_nothing_is_cached() {
        let mut store = store_with_clock(FixedClock::new());

        let decision = store.visit(&path());

        assert_eq!(decision, FetchDecision::Fetch);
    }

    #[test]
    fn visit_is_idle_while_a_run_is_already_going() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());

        let decision = store.visit(&path());

        assert_eq!(decision, FetchDecision::Idle);
    }

    #[test]
    fn visit_is_idle_while_the_cached_result_is_fresh() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());
        store.finish_fetch(&path(), Ok(empty_config()));

        let decision = store.visit(&path());

        assert_eq!(decision, FetchDecision::Idle);
    }

    #[test]
    fn visit_refreshes_once_the_time_to_live_has_elapsed() {
        let clock = FixedClock::new();
        let mut store = store_with_clock(clock.clone());
        store.visit(&path());
        store.finish_fetch(&path(), Ok(empty_config()));

        clock.advance(Duration::from_secs(16 * 60));
        let decision = store.visit(&path());

        assert_eq!(decision, FetchDecision::Refresh);
    }

    #[test]
    fn finish_fetch_stores_the_result_and_stops_loading() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());

        store.finish_fetch(&path(), Ok(empty_config()));

        assert_eq!(store.visit(&path()), FetchDecision::Idle);
    }

    #[test]
    fn finish_fetch_keeps_the_cached_result_when_a_refresh_fails() {
        let clock = FixedClock::new();
        let mut store = store_with_clock(clock.clone());
        store.visit(&path());
        store.finish_fetch(&path(), Ok(populated_config()));
        clock.advance(Duration::from_secs(16 * 60));
        store.visit(&path());

        store.finish_fetch(&path(), Err(error()));

        assert_eq!(store.display_state(&path()), DisplayState::Populated);
        assert!(store.config(&path()).is_some());
    }

    #[test]
    fn the_title_survives_a_failed_refresh() {
        let clock = FixedClock::new();
        let mut store = store_with_clock(clock.clone());
        store.visit(&path());
        store.finish_fetch(&path(), Ok(populated_config()));
        clock.advance(Duration::from_secs(16 * 60));
        store.visit(&path());

        store.finish_fetch(&path(), Err(error()));

        assert_eq!(store.title(&path()), Some("Test"));
    }

    #[test]
    fn finish_fetch_clears_an_error_recorded_by_an_earlier_attempt() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());
        store.finish_fetch(&path(), Err(error()));

        store.finish_fetch(&path(), Ok(empty_config()));

        assert_eq!(store.error(&path()), None);
    }

    #[test]
    fn display_state_is_loading_before_the_first_result_arrives() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());

        assert_eq!(store.display_state(&path()), DisplayState::Loading);
    }

    #[test]
    fn display_state_is_failed_when_the_first_run_fails() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());

        store.finish_fetch(&path(), Err(error()));

        assert_eq!(store.display_state(&path()), DisplayState::Failed);
    }

    #[test]
    fn display_state_is_empty_when_the_script_returns_no_items() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());

        store.finish_fetch(&path(), Ok(empty_config()));

        assert_eq!(store.display_state(&path()), DisplayState::Empty);
    }

    #[test]
    fn display_state_is_populated_while_a_stale_result_is_refreshed() {
        let clock = FixedClock::new();
        let mut store = store_with_clock(clock.clone());
        store.visit(&path());
        store.finish_fetch(&path(), Ok(populated_config()));
        clock.advance(Duration::from_secs(16 * 60));

        store.visit(&path());

        assert_eq!(store.display_state(&path()), DisplayState::Populated);
        assert!(store.is_refreshing(&path()));
    }

    #[test]
    fn is_refreshing_is_false_when_there_is_nothing_cached_to_show() {
        let mut store = store_with_clock(FixedClock::new());

        store.visit(&path());

        assert!(!store.is_refreshing(&path()));
    }

    #[test]
    fn force_refresh_fetches_when_nothing_is_cached() {
        let mut store = store_with_clock(FixedClock::new());

        let decision = store.force_refresh(&path());

        assert_eq!(decision, FetchDecision::Fetch);
    }

    #[test]
    fn force_refresh_refreshes_a_fresh_cached_result() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());
        store.finish_fetch(&path(), Ok(populated_config()));

        let decision = store.force_refresh(&path());

        assert_eq!(decision, FetchDecision::Refresh);
    }

    #[test]
    fn force_refresh_refreshes_a_stale_cached_result() {
        let clock = FixedClock::new();
        let mut store = store_with_clock(clock.clone());
        store.visit(&path());
        store.finish_fetch(&path(), Ok(populated_config()));
        clock.advance(Duration::from_secs(16 * 60));

        let decision = store.force_refresh(&path());

        assert_eq!(decision, FetchDecision::Refresh);
    }

    #[test]
    fn force_refresh_is_idle_while_a_run_is_already_going() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());

        let decision = store.force_refresh(&path());

        assert_eq!(decision, FetchDecision::Idle);
    }

    #[test]
    fn force_refresh_keeps_the_cached_result_visible_while_it_runs() {
        let mut store = store_with_clock(FixedClock::new());
        store.visit(&path());
        store.finish_fetch(&path(), Ok(populated_config()));

        store.force_refresh(&path());

        assert!(store.is_refreshing(&path()));
        assert_eq!(store.display_state(&path()), DisplayState::Populated);
    }
}

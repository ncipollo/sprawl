//! A per-script cache of section results with a time to live. Pure: it
//! takes the current time as an argument, so it has no clock of its own to
//! fake.

use crate::feature::script::schema::SectionConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How long a script result is considered fresh.
pub const DEFAULT_TTL: Duration = Duration::from_secs(5 * 60);

/// How usable a cached result is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheState {
    /// The script has not produced a result yet.
    Missing,
    /// The script ran, but longer than the time to live ago.
    Stale,
    /// The script ran within the time to live.
    Fresh,
}

struct Entry {
    config: SectionConfig,
    fetched_at: Instant,
}

/// Section results per script, with a time to live.
pub struct SectionCache {
    ttl: Duration,
    entries: HashMap<PathBuf, Entry>,
}

impl SectionCache {
    pub fn new() -> Self {
        Self::with_ttl(DEFAULT_TTL)
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: HashMap::new(),
        }
    }

    /// How usable the cached result for the script at `path` is, as of `now`.
    pub fn state(&self, path: &Path, now: Instant) -> CacheState {
        match self.entries.get(path) {
            None => CacheState::Missing,
            Some(entry) if now.duration_since(entry.fetched_at) < self.ttl => CacheState::Fresh,
            Some(_) => CacheState::Stale,
        }
    }

    /// Whether the script has produced any result, fresh or not.
    pub fn contains(&self, path: &Path) -> bool {
        self.entries.contains_key(path)
    }

    /// The cached section, if the script has produced one.
    pub fn config(&self, path: &Path) -> Option<&SectionConfig> {
        self.entries.get(path).map(|entry| &entry.config)
    }

    /// Replaces the cached result for the script at `path`, run at `now`.
    pub fn store(&mut self, path: PathBuf, config: SectionConfig, now: Instant) {
        self.entries.insert(
            path,
            Entry {
                config,
                fetched_at: now,
            },
        );
    }
}

impl Default for SectionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path() -> PathBuf {
        PathBuf::from("/tmp/section.js")
    }

    fn config(title: &str) -> SectionConfig {
        SectionConfig {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    #[test]
    fn state_is_missing_before_anything_is_stored() {
        let cache = SectionCache::new();

        assert_eq!(cache.state(&path(), Instant::now()), CacheState::Missing);
    }

    #[test]
    fn state_is_fresh_immediately_after_storing() {
        let mut cache = SectionCache::new();
        let now = Instant::now();

        cache.store(path(), config("T"), now);

        assert_eq!(cache.state(&path(), now), CacheState::Fresh);
    }

    #[test]
    fn state_is_stale_once_the_time_to_live_has_elapsed() {
        let mut cache = SectionCache::with_ttl(Duration::from_secs(60));
        let now = Instant::now();
        cache.store(path(), config("T"), now);

        let state = cache.state(&path(), now + Duration::from_secs(60));

        assert_eq!(state, CacheState::Stale);
    }

    #[test]
    fn state_is_fresh_one_moment_before_the_time_to_live_elapses() {
        let mut cache = SectionCache::with_ttl(Duration::from_secs(60));
        let now = Instant::now();
        cache.store(path(), config("T"), now);

        let state = cache.state(&path(), now + Duration::from_millis(59_999));

        assert_eq!(state, CacheState::Fresh);
    }

    #[test]
    fn config_is_none_when_nothing_is_cached() {
        let cache = SectionCache::new();

        assert_eq!(cache.config(&path()), None);
    }

    #[test]
    fn store_replaces_the_previous_result_for_a_script() {
        let mut cache = SectionCache::new();
        let now = Instant::now();
        cache.store(path(), config("first"), now);

        cache.store(path(), config("second"), now);

        assert_eq!(
            cache.config(&path()).map(|c| c.title.as_str()),
            Some("second")
        );
    }

    #[test]
    fn each_script_is_cached_independently() {
        let mut cache = SectionCache::new();
        let now = Instant::now();
        cache.store(path(), config("T"), now);

        assert!(cache.config(Path::new("/tmp/other.js")).is_none());
        assert!(cache.config(&path()).is_some());
    }
}

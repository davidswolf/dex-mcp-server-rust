//! Time-based cache with TTL (Time To Live) support.
//!
//! This module provides a thread-safe cache that automatically expires entries
//! after a specified duration.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// A cache entry with a timestamp.
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
}

/// A thread-safe cache with time-based expiration.
///
/// Entries are automatically expired after the configured TTL (Time To Live) duration.
/// The cache is thread-safe and can be cloned cheaply (uses Arc internally).
///
/// # Memory Efficiency with Arc
///
/// For large values, consider wrapping them in `Arc` to avoid cloning:
/// ```ignore
/// let cache = TimedCache::<String, Arc<LargeStruct>>::new(60);
/// cache.insert("key".to_string(), Arc::new(large_value));
/// let value: Option<Arc<LargeStruct>> = cache.get(&"key".to_string());
/// ```
#[derive(Clone)]
pub struct TimedCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    cache: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}

impl<K, V> TimedCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new TimedCache with the specified TTL in seconds.
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Insert a value into the cache.
    ///
    /// If a value with the same key already exists, it will be replaced.
    pub fn insert(&self, key: K, value: V) {
        let entry = CacheEntry {
            value,
            inserted_at: Instant::now(),
        };

        if let Ok(mut cache) = self.cache.write() {
            cache.insert(key, entry);
        }
    }

    /// Get a value from the cache if it exists and hasn't expired.
    ///
    /// Returns `None` if:
    /// - The key doesn't exist
    /// - The entry has expired (older than TTL)
    pub fn get(&self, key: &K) -> Option<V> {
        let now = Instant::now();

        if let Ok(cache) = self.cache.read() {
            if let Some(entry) = cache.get(key) {
                if now.duration_since(entry.inserted_at) < self.ttl {
                    return Some(entry.value.clone());
                }
            }
        }

        None
    }

    /// Check if a key exists in the cache and hasn't expired.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Remove a specific key from the cache.
    pub fn remove(&self, key: &K) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(key);
        }
    }

    /// Clear all entries from the cache.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Remove all expired entries from the cache.
    ///
    /// This is useful for freeing up memory, but it's not required as
    /// expired entries are automatically ignored by `get()`.
    pub fn cleanup_expired(&self) {
        let now = Instant::now();

        if let Ok(mut cache) = self.cache.write() {
            cache.retain(|_, entry| now.duration_since(entry.inserted_at) < self.ttl);
        }
    }

    /// Get the number of entries in the cache (including expired ones).
    pub fn len(&self) -> usize {
        if let Ok(cache) = self.cache.read() {
            cache.len()
        } else {
            0
        }
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the TTL duration for this cache.
    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

impl<K, V> std::fmt::Debug for TimedCache<K, V>
where
    K: Eq + Hash + Clone + std::fmt::Debug,
    V: Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimedCache")
            .field("ttl", &self.ttl)
            .field("entries", &self.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_insert_and_get() {
        let cache = TimedCache::new(60);
        cache.insert("key1", "value1");

        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key2"), None);
    }

    #[test]
    fn test_ttl_expiration() {
        let cache = TimedCache::new(1); // 1 second TTL
        cache.insert("key1", "value1");

        // Should exist immediately
        assert_eq!(cache.get(&"key1"), Some("value1"));

        // Wait for expiration
        thread::sleep(Duration::from_millis(1100));

        // Should be expired
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn test_contains_key() {
        let cache = TimedCache::new(60);
        cache.insert("key1", "value1");

        assert!(cache.contains_key(&"key1"));
        assert!(!cache.contains_key(&"key2"));
    }

    #[test]
    fn test_remove() {
        let cache = TimedCache::new(60);
        cache.insert("key1", "value1");
        cache.insert("key2", "value2");

        assert_eq!(cache.len(), 2);

        cache.remove(&"key1");

        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.get(&"key2"), Some("value2"));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_clear() {
        let cache = TimedCache::new(60);
        cache.insert("key1", "value1");
        cache.insert("key2", "value2");

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cleanup_expired() {
        let cache = TimedCache::new(1); // 1 second TTL
        cache.insert("key1", "value1");
        cache.insert("key2", "value2");

        assert_eq!(cache.len(), 2);

        // Wait for expiration
        thread::sleep(Duration::from_millis(1100));

        // Still 2 entries (expired but not cleaned up)
        assert_eq!(cache.len(), 2);

        // Cleanup expired entries
        cache.cleanup_expired();

        // Should be empty now
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_update_value() {
        let cache = TimedCache::new(60);
        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));

        // Update the value
        cache.insert("key1", "value2");
        assert_eq!(cache.get(&"key1"), Some("value2"));
    }

    #[test]
    fn test_clone_cache() {
        let cache1 = TimedCache::new(60);
        cache1.insert("key1", "value1");

        // Clone shares the same underlying cache
        let cache2 = cache1.clone();
        assert_eq!(cache2.get(&"key1"), Some("value1"));

        // Updates are visible in both
        cache2.insert("key2", "value2");
        assert_eq!(cache1.get(&"key2"), Some("value2"));
    }

    #[test]
    fn test_concurrent_access() {
        let cache = TimedCache::new(60);
        let cache_clone = cache.clone();

        let handle = thread::spawn(move || {
            for i in 0..100 {
                cache_clone.insert(format!("key{}", i), format!("value{}", i));
            }
        });

        for i in 100..200 {
            cache.insert(format!("key{}", i), format!("value{}", i));
        }

        handle.join().unwrap();

        // Should have 200 entries
        assert_eq!(cache.len(), 200);
    }

    #[test]
    fn test_debug_format() {
        let cache = TimedCache::new(60);
        cache.insert("key1", "value1");

        let debug_str = format!("{:?}", cache);
        assert!(debug_str.contains("TimedCache"));
        assert!(debug_str.contains("ttl"));
    }
}

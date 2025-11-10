//! Production-grade metrics tracking.
//!
//! This module provides comprehensive metrics for monitoring the health
//! and performance of the Dex MCP server in production.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Global metrics tracker for the application.
#[derive(Debug, Clone)]
pub struct MetricsTracker {
    http_requests_total: Arc<AtomicU64>,
    http_errors_total: Arc<AtomicU64>,
    cache_hits_total: Arc<AtomicU64>,
    cache_misses_total: Arc<AtomicU64>,
    search_queries_total: Arc<AtomicU64>,
}

impl MetricsTracker {
    /// Create a new metrics tracker.
    pub fn new() -> Self {
        Self {
            http_requests_total: Arc::new(AtomicU64::new(0)),
            http_errors_total: Arc::new(AtomicU64::new(0)),
            cache_hits_total: Arc::new(AtomicU64::new(0)),
            cache_misses_total: Arc::new(AtomicU64::new(0)),
            search_queries_total: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Track an HTTP request.
    pub fn track_http_request(&self, method: &str, duration_ms: u128, success: bool) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);

        if !success {
            self.http_errors_total.fetch_add(1, Ordering::Relaxed);
        }

        tracing::debug!(
            method = %method,
            duration_ms = duration_ms,
            success = success,
            "HTTP request completed"
        );
    }

    /// Track a cache access.
    pub fn track_cache_access(&self, cache_type: &str, hit: bool) {
        if hit {
            self.cache_hits_total.fetch_add(1, Ordering::Relaxed);
            tracing::trace!(cache_type = %cache_type, "Cache hit");
        } else {
            self.cache_misses_total.fetch_add(1, Ordering::Relaxed);
            tracing::trace!(cache_type = %cache_type, "Cache miss");
        }
    }

    /// Track a search query.
    pub fn track_search_query(&self, duration_ms: u128, result_count: usize) {
        self.search_queries_total.fetch_add(1, Ordering::Relaxed);

        tracing::info!(
            duration_ms = duration_ms,
            result_count = result_count,
            "Search query completed"
        );
    }

    /// Get the total number of HTTP requests.
    pub fn http_requests_total(&self) -> u64 {
        self.http_requests_total.load(Ordering::Relaxed)
    }

    /// Get the total number of HTTP errors.
    pub fn http_errors_total(&self) -> u64 {
        self.http_errors_total.load(Ordering::Relaxed)
    }

    /// Get the total number of cache hits.
    pub fn cache_hits_total(&self) -> u64 {
        self.cache_hits_total.load(Ordering::Relaxed)
    }

    /// Get the total number of cache misses.
    pub fn cache_misses_total(&self) -> u64 {
        self.cache_misses_total.load(Ordering::Relaxed)
    }

    /// Get the total number of search queries.
    pub fn search_queries_total(&self) -> u64 {
        self.search_queries_total.load(Ordering::Relaxed)
    }

    /// Get the cache hit rate (0.0 to 1.0).
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits_total() as f64;
        let total = (self.cache_hits_total() + self.cache_misses_total()) as f64;

        if total == 0.0 {
            0.0
        } else {
            hits / total
        }
    }

    /// Get the HTTP error rate (0.0 to 1.0).
    pub fn http_error_rate(&self) -> f64 {
        let errors = self.http_errors_total() as f64;
        let total = self.http_requests_total() as f64;

        if total == 0.0 {
            0.0
        } else {
            errors / total
        }
    }

    /// Print a summary of all metrics.
    pub fn summary(&self) -> String {
        format!(
            "Metrics Summary:\n\
             HTTP Requests: {}\n\
             HTTP Errors: {} ({:.2}% error rate)\n\
             Cache Hits: {}\n\
             Cache Misses: {}\n\
             Cache Hit Rate: {:.2}%\n\
             Search Queries: {}",
            self.http_requests_total(),
            self.http_errors_total(),
            self.http_error_rate() * 100.0,
            self.cache_hits_total(),
            self.cache_misses_total(),
            self.cache_hit_rate() * 100.0,
            self.search_queries_total(),
        )
    }
}

impl Default for MetricsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A timer for tracking operation duration.
pub struct Timer {
    start: Instant,
    operation: String,
}

impl Timer {
    /// Start a new timer for the given operation.
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.into(),
        }
    }

    /// Finish the timer and return the elapsed time in milliseconds.
    pub fn finish(self) -> u128 {
        let duration = self.start.elapsed();
        let duration_ms = duration.as_millis();

        tracing::debug!(
            operation = %self.operation,
            duration_ms = duration_ms,
            "Operation completed"
        );

        duration_ms
    }

    /// Finish the timer with a specific status.
    pub fn finish_with_status(self, success: bool) -> u128 {
        let duration = self.start.elapsed();
        let duration_ms = duration.as_millis();

        if success {
            tracing::debug!(
                operation = %self.operation,
                duration_ms = duration_ms,
                "Operation succeeded"
            );
        } else {
            tracing::warn!(
                operation = %self.operation,
                duration_ms = duration_ms,
                "Operation failed"
            );
        }

        duration_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_tracker_creation() {
        let tracker = MetricsTracker::new();
        assert_eq!(tracker.http_requests_total(), 0);
        assert_eq!(tracker.http_errors_total(), 0);
        assert_eq!(tracker.cache_hits_total(), 0);
        assert_eq!(tracker.cache_misses_total(), 0);
        assert_eq!(tracker.search_queries_total(), 0);
    }

    #[test]
    fn test_track_http_request() {
        let tracker = MetricsTracker::new();

        tracker.track_http_request("get_contact", 100, true);
        assert_eq!(tracker.http_requests_total(), 1);
        assert_eq!(tracker.http_errors_total(), 0);

        tracker.track_http_request("get_contact", 200, false);
        assert_eq!(tracker.http_requests_total(), 2);
        assert_eq!(tracker.http_errors_total(), 1);
    }

    #[test]
    fn test_track_cache_access() {
        let tracker = MetricsTracker::new();

        tracker.track_cache_access("contact_cache", true);
        assert_eq!(tracker.cache_hits_total(), 1);
        assert_eq!(tracker.cache_misses_total(), 0);

        tracker.track_cache_access("contact_cache", false);
        assert_eq!(tracker.cache_hits_total(), 1);
        assert_eq!(tracker.cache_misses_total(), 1);
    }

    #[test]
    fn test_track_search_query() {
        let tracker = MetricsTracker::new();

        tracker.track_search_query(150, 5);
        assert_eq!(tracker.search_queries_total(), 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        let tracker = MetricsTracker::new();

        tracker.track_cache_access("test", true);
        tracker.track_cache_access("test", true);
        tracker.track_cache_access("test", false);

        let hit_rate = tracker.cache_hit_rate();
        assert!((hit_rate - 0.6667).abs() < 0.001);
    }

    #[test]
    fn test_http_error_rate() {
        let tracker = MetricsTracker::new();

        tracker.track_http_request("test", 100, true);
        tracker.track_http_request("test", 100, true);
        tracker.track_http_request("test", 100, false);

        let error_rate = tracker.http_error_rate();
        assert!((error_rate - 0.3333).abs() < 0.001);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = timer.finish();
        assert!(duration >= 10);
    }

    #[test]
    fn test_summary() {
        let tracker = MetricsTracker::new();
        tracker.track_http_request("test", 100, true);
        tracker.track_cache_access("test", true);

        let summary = tracker.summary();
        assert!(summary.contains("HTTP Requests: 1"));
        assert!(summary.contains("Cache Hits: 1"));
    }
}

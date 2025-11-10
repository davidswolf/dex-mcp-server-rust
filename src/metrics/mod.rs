//! Basic metrics instrumentation for tracking performance.
//!
//! Provides counters and duration tracking for HTTP requests and API operations.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Metrics collector for tracking API performance.
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Total number of HTTP requests made
    http_requests_total: Arc<AtomicU64>,

    /// Total number of HTTP errors
    http_errors_total: Arc<AtomicU64>,

    /// Total duration of all HTTP requests in milliseconds
    http_duration_total_ms: Arc<AtomicU64>,

    /// Number of contacts fetched
    contacts_fetched_total: Arc<AtomicU64>,

    /// Number of notes fetched
    notes_fetched_total: Arc<AtomicU64>,

    /// Number of reminders fetched
    reminders_fetched_total: Arc<AtomicU64>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self {
            http_requests_total: Arc::new(AtomicU64::new(0)),
            http_errors_total: Arc::new(AtomicU64::new(0)),
            http_duration_total_ms: Arc::new(AtomicU64::new(0)),
            contacts_fetched_total: Arc::new(AtomicU64::new(0)),
            notes_fetched_total: Arc::new(AtomicU64::new(0)),
            reminders_fetched_total: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record an HTTP request with duration.
    pub fn record_http_request(&self, duration: Duration) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);
        self.http_duration_total_ms
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }

    /// Record an HTTP error.
    pub fn record_http_error(&self) {
        self.http_errors_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record contacts fetched.
    pub fn record_contacts_fetched(&self, count: usize) {
        self.contacts_fetched_total
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    /// Record notes fetched.
    pub fn record_notes_fetched(&self, count: usize) {
        self.notes_fetched_total
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    /// Record reminders fetched.
    pub fn record_reminders_fetched(&self, count: usize) {
        self.reminders_fetched_total
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    /// Get total HTTP requests.
    pub fn http_requests_total(&self) -> u64 {
        self.http_requests_total.load(Ordering::Relaxed)
    }

    /// Get total HTTP errors.
    pub fn http_errors_total(&self) -> u64 {
        self.http_errors_total.load(Ordering::Relaxed)
    }

    /// Get total HTTP duration in milliseconds.
    pub fn http_duration_total_ms(&self) -> u64 {
        self.http_duration_total_ms.load(Ordering::Relaxed)
    }

    /// Get average HTTP request duration in milliseconds.
    pub fn http_duration_avg_ms(&self) -> f64 {
        let total = self.http_duration_total_ms.load(Ordering::Relaxed);
        let count = self.http_requests_total.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            total as f64 / count as f64
        }
    }

    /// Get total contacts fetched.
    pub fn contacts_fetched_total(&self) -> u64 {
        self.contacts_fetched_total.load(Ordering::Relaxed)
    }

    /// Get total notes fetched.
    pub fn notes_fetched_total(&self) -> u64 {
        self.notes_fetched_total.load(Ordering::Relaxed)
    }

    /// Get total reminders fetched.
    pub fn reminders_fetched_total(&self) -> u64 {
        self.reminders_fetched_total.load(Ordering::Relaxed)
    }

    /// Reset all metrics to zero.
    pub fn reset(&self) {
        self.http_requests_total.store(0, Ordering::Relaxed);
        self.http_errors_total.store(0, Ordering::Relaxed);
        self.http_duration_total_ms.store(0, Ordering::Relaxed);
        self.contacts_fetched_total.store(0, Ordering::Relaxed);
        self.notes_fetched_total.store(0, Ordering::Relaxed);
        self.reminders_fetched_total.store(0, Ordering::Relaxed);
    }

    /// Get a summary of all metrics.
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            http_requests_total: self.http_requests_total(),
            http_errors_total: self.http_errors_total(),
            http_duration_total_ms: self.http_duration_total_ms(),
            http_duration_avg_ms: self.http_duration_avg_ms(),
            contacts_fetched_total: self.contacts_fetched_total(),
            notes_fetched_total: self.notes_fetched_total(),
            reminders_fetched_total: self.reminders_fetched_total(),
        }
    }
}

/// A snapshot of metrics values.
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub http_requests_total: u64,
    pub http_errors_total: u64,
    pub http_duration_total_ms: u64,
    pub http_duration_avg_ms: f64,
    pub contacts_fetched_total: u64,
    pub notes_fetched_total: u64,
    pub reminders_fetched_total: u64,
}

/// Helper for timing HTTP requests.
pub struct HttpTimer {
    start: Instant,
    metrics: Metrics,
}

impl HttpTimer {
    /// Start timing an HTTP request.
    pub fn new(metrics: Metrics) -> Self {
        Self {
            start: Instant::now(),
            metrics,
        }
    }

    /// Complete the timing and record the duration.
    pub fn complete(self) {
        let duration = self.start.elapsed();
        self.metrics.record_http_request(duration);
    }

    /// Complete the timing and record as an error.
    pub fn complete_with_error(self) {
        let duration = self.start.elapsed();
        self.metrics.record_http_request(duration);
        self.metrics.record_http_error();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert_eq!(metrics.http_requests_total(), 0);
        assert_eq!(metrics.http_errors_total(), 0);
        assert_eq!(metrics.http_duration_total_ms(), 0);
    }

    #[test]
    fn test_record_http_request() {
        let metrics = Metrics::new();
        metrics.record_http_request(Duration::from_millis(100));
        assert_eq!(metrics.http_requests_total(), 1);
        assert_eq!(metrics.http_duration_total_ms(), 100);
        assert_eq!(metrics.http_duration_avg_ms(), 100.0);
    }

    #[test]
    fn test_record_http_error() {
        let metrics = Metrics::new();
        metrics.record_http_error();
        assert_eq!(metrics.http_errors_total(), 1);
    }

    #[test]
    fn test_record_contacts_fetched() {
        let metrics = Metrics::new();
        metrics.record_contacts_fetched(5);
        assert_eq!(metrics.contacts_fetched_total(), 5);
    }

    #[test]
    fn test_average_duration() {
        let metrics = Metrics::new();
        metrics.record_http_request(Duration::from_millis(100));
        metrics.record_http_request(Duration::from_millis(200));
        assert_eq!(metrics.http_requests_total(), 2);
        assert_eq!(metrics.http_duration_total_ms(), 300);
        assert_eq!(metrics.http_duration_avg_ms(), 150.0);
    }

    #[test]
    fn test_reset() {
        let metrics = Metrics::new();
        metrics.record_http_request(Duration::from_millis(100));
        metrics.record_http_error();
        metrics.record_contacts_fetched(5);

        assert_eq!(metrics.http_requests_total(), 1);
        assert_eq!(metrics.http_errors_total(), 1);
        assert_eq!(metrics.contacts_fetched_total(), 5);

        metrics.reset();

        assert_eq!(metrics.http_requests_total(), 0);
        assert_eq!(metrics.http_errors_total(), 0);
        assert_eq!(metrics.contacts_fetched_total(), 0);
    }

    #[test]
    fn test_summary() {
        let metrics = Metrics::new();
        metrics.record_http_request(Duration::from_millis(100));
        metrics.record_http_error();
        metrics.record_contacts_fetched(3);

        let summary = metrics.summary();
        assert_eq!(summary.http_requests_total, 1);
        assert_eq!(summary.http_errors_total, 1);
        assert_eq!(summary.http_duration_total_ms, 100);
        assert_eq!(summary.http_duration_avg_ms, 100.0);
        assert_eq!(summary.contacts_fetched_total, 3);
    }

    #[test]
    fn test_http_timer() {
        let metrics = Metrics::new();
        let timer = HttpTimer::new(metrics.clone());
        thread::sleep(Duration::from_millis(10));
        timer.complete();

        assert_eq!(metrics.http_requests_total(), 1);
        assert!(metrics.http_duration_total_ms() >= 10);
    }

    #[test]
    fn test_http_timer_with_error() {
        let metrics = Metrics::new();
        let timer = HttpTimer::new(metrics.clone());
        timer.complete_with_error();

        assert_eq!(metrics.http_requests_total(), 1);
        assert_eq!(metrics.http_errors_total(), 1);
    }

    #[test]
    fn test_concurrent_access() {
        let metrics = Metrics::new();
        let metrics1 = metrics.clone();
        let metrics2 = metrics.clone();

        let handle1 = thread::spawn(move || {
            for _ in 0..100 {
                metrics1.record_http_request(Duration::from_millis(1));
            }
        });

        let handle2 = thread::spawn(move || {
            for _ in 0..100 {
                metrics2.record_http_request(Duration::from_millis(1));
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        assert_eq!(metrics.http_requests_total(), 200);
    }
}

//! Observability module for monitoring and metrics.
//!
//! This module provides production-grade observability features including
//! metrics tracking, structured logging, and performance monitoring.

pub mod metrics;

pub use metrics::{MetricsTracker, Timer};

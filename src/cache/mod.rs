//! Caching utilities for the Dex MCP Server.
//!
//! This module provides a generic time-based cache implementation with TTL support.

pub mod timed_cache;

pub use timed_cache::TimedCache;

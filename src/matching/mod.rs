//! Fuzzy matching utilities for contact discovery.
//!
//! This module provides fuzzy and exact matching for contacts based on
//! names, emails, phone numbers, and social media URLs.

pub mod fuzzy_matcher;

pub use fuzzy_matcher::{ContactMatcher, ContactQuery, MatchResult, MatchType};

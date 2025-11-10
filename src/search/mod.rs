//! Full-text search utilities for contacts, notes, and reminders.
//!
//! This module provides fuzzy full-text search across all contact-related data,
//! with snippet generation and result aggregation.

pub mod full_text_index;

pub use full_text_index::{
    FullTextSearchIndex, MatchContext, SearchResult, SearchableDocument, SearchableField,
};

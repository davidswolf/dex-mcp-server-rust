//! MCP tools for interacting with Dex Personal CRM.
//!
//! This module provides four categories of tools:
//! - **Discovery**: Find and retrieve contacts
//! - **History**: Access relationship history (notes, reminders, timeline)
//! - **Enrichment**: Update contacts and add notes/reminders
//! - **Search**: Full-text search with caching

pub mod discovery;
pub mod enrichment;
pub mod history;
pub mod search;

pub use discovery::{ContactDiscoveryTools, FindContactParams, FindContactResponse};
pub use enrichment::{
    ContactEnrichmentTools, CreateNoteParams, CreateReminderParams, EnrichContactParams,
};
pub use history::{
    ContactHistoryResponse, HistoryFilterParams, RelationshipHistoryTools, TimelineEntry,
};
pub use search::{SearchParams, SearchResponse, SearchTools};

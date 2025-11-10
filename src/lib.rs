//! Dex MCP Server - A Rust implementation of the Model Context Protocol server for Dex Personal CRM.
//!
//! This library provides a production-quality MCP server that enables AI assistants
//! to interact with the Dex Personal CRM system for contact discovery, relationship
//! history tracking, and contact enrichment.
//!
//! # Architecture
//!
//! - **models**: Data structures for contacts, notes, and reminders
//! - **error**: Custom error types for precise error handling
//! - **config**: Configuration management from environment variables
//! - **client**: HTTP client for the Dex API (to be implemented)
//! - **tools**: MCP tool implementations (to be implemented)
//! - **matching**: Fuzzy matching and search utilities (to be implemented)
//! - **cache**: Caching implementations (to be implemented)
//! - **server**: MCP protocol server (to be implemented)

// Re-export commonly used types
pub mod cache;
pub mod client;
pub mod config;
pub mod domain;
pub mod error;
pub mod matching;
pub mod metrics;
pub mod models;
pub mod observability;
pub mod repositories;
pub mod search;
pub mod server;
pub mod services;
pub mod tools;

pub use cache::TimedCache;
pub use client::DexClient;
pub use config::Config;
pub use error::{ConfigError, DexApiError, MatchingError, SearchError};
pub use matching::{ContactMatcher, MatchResult};
pub use metrics::{HttpTimer, Metrics, MetricsSummary};
pub use models::{Contact, Note, Reminder, SocialProfile};
pub use search::{FullTextSearchIndex, MatchContext, SearchResult, SearchableDocument};
pub use server::DexMcpServer;
pub use tools::{
    ContactDiscoveryTools, ContactEnrichmentTools, ContactHistoryResponse, CreateNoteParams,
    CreateReminderParams, EnrichContactParams, FindContactParams, FindContactResponse,
    HistoryFilterParams, RelationshipHistoryTools, SearchParams, SearchResponse, SearchTools,
    TimelineEntry,
};

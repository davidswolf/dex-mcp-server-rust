//! Search tools for full-text search with caching.
//!
//! Provides efficient full-text search over contacts, notes, and reminders
//! using a cached search index.

use crate::cache::TimedCache;
use crate::error::DexApiResult;
use crate::models::Contact;
use crate::repositories::{ContactRepository, NoteRepository, ReminderRepository};
use crate::search::{FullTextSearchIndex, SearchResult};
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cached search data including index and contacts.
#[derive(Clone)]
struct SearchCache {
    index: Arc<FullTextSearchIndex>,
    contacts: Arc<Vec<Contact>>,
}

/// Search tools for performing full-text searches.
#[derive(Clone)]
pub struct SearchTools {
    contact_repo: Arc<dyn ContactRepository>,
    note_repo: Arc<dyn NoteRepository>,
    reminder_repo: Arc<dyn ReminderRepository>,
    /// Cached search index and contacts
    cache: Arc<RwLock<TimedCache<String, SearchCache>>>,
    cache_ttl_secs: u64,
}

/// Parameters for full-text search.
#[derive(Debug, Clone)]
pub struct SearchParams {
    /// Search query string
    pub query: String,

    /// Maximum number of results to return (default: 10)
    pub max_results: Option<usize>,

    /// Minimum confidence threshold (0-100, default: 50)
    pub min_confidence: Option<u8>,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_results: Some(10),
            min_confidence: Some(50),
        }
    }
}

/// Response from search with cache metadata.
#[derive(Debug, Clone)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,

    /// Whether the results came from cache
    pub from_cache: bool,

    /// Number of documents in the index
    pub index_size: usize,
}

impl SearchTools {
    /// Create new search tools.
    ///
    /// # Arguments
    /// * `contact_repo` - ContactRepository for contact data access
    /// * `note_repo` - NoteRepository for note data access
    /// * `reminder_repo` - ReminderRepository for reminder data access
    /// * `cache_ttl_secs` - Cache time-to-live in seconds
    pub fn new(
        contact_repo: Arc<dyn ContactRepository>,
        note_repo: Arc<dyn NoteRepository>,
        reminder_repo: Arc<dyn ReminderRepository>,
        cache_ttl_secs: u64,
    ) -> Self {
        Self {
            contact_repo,
            note_repo,
            reminder_repo,
            cache: Arc::new(RwLock::new(TimedCache::new(cache_ttl_secs))),
            cache_ttl_secs,
        }
    }

    /// Perform a full-text search.
    ///
    /// This method uses a cached search index for performance. The index is
    /// built on first search and cached for `cache_ttl_secs`.
    ///
    /// # Arguments
    /// * `params` - Search parameters
    ///
    /// # Returns
    /// Search results with cache metadata
    pub async fn search_full_text(&self, params: SearchParams) -> DexApiResult<SearchResponse> {
        let max_results = params.max_results.unwrap_or(10);
        let min_confidence = params.min_confidence.unwrap_or(50);

        let (search_cache, from_cache) = self.get_or_build_cache().await?;

        // Perform search on the index
        let results = search_cache
            .index
            .search(&search_cache.contacts, &params.query, max_results, min_confidence);
        let index_size = search_cache.index.document_count();

        Ok(SearchResponse {
            results,
            from_cache,
            index_size,
        })
    }

    /// Get the cached search data or build new.
    async fn get_or_build_cache(&self) -> DexApiResult<(SearchCache, bool)> {
        let cache_key = "search_data".to_string();

        // Try to get from cache
        {
            let cache = self.cache.read().await;
            if let Some(cached_data) = cache.get(&cache_key) {
                tracing::debug!("Using cached search index");
                return Ok((cached_data, true));
            }
        }

        // Build new index and fetch contacts
        tracing::info!("Building search index");
        let start = std::time::Instant::now();

        let contacts = self.fetch_all_contacts().await?;

        tracing::info!(
            "Fetching notes and reminders for {} contacts in parallel",
            contacts.len()
        );

        // Create owned contact data for parallel processing
        let contact_data: Vec<_> = contacts
            .iter()
            .map(|c| (c.clone(), c.id.clone()))
            .collect();

        // Fetch notes and reminders in parallel with bounded concurrency
        let results = stream::iter(contact_data)
            .map(|(contact, contact_id)| {
                let note_repo = self.note_repo.clone();
                let reminder_repo = self.reminder_repo.clone();

                async move {
                    // Fetch both notes and reminders concurrently for each contact
                    let (notes_result, reminders_result) = tokio::join!(
                        note_repo.get_for_contact(&contact_id, 100, 0),
                        reminder_repo.get_for_contact(&contact_id, 100, 0),
                    );

                    // Don't fail entire build if one contact fails
                    let notes = notes_result.unwrap_or_else(|e| {
                        tracing::warn!(
                            "Failed to fetch notes for contact {}: {}",
                            contact_id,
                            e
                        );
                        Vec::new()
                    });

                    let reminders = reminders_result.unwrap_or_else(|e| {
                        tracing::warn!(
                            "Failed to fetch reminders for contact {}: {}",
                            contact_id,
                            e
                        );
                        Vec::new()
                    });

                    (contact, notes, reminders)
                }
            })
            .buffer_unordered(20) // Max 20 concurrent contact fetches
            .collect::<Vec<_>>()
            .await;

        // Build index from results
        let mut index = FullTextSearchIndex::new();
        for (contact, notes, reminders) in results {
            index.index_contact(&contact, &notes, &reminders);
        }

        let duration = start.elapsed();
        tracing::info!(
            "Search index built in {}ms ({} contacts indexed)",
            duration.as_millis(),
            contacts.len()
        );

        let search_cache = SearchCache {
            index: Arc::new(index),
            contacts: Arc::new(contacts),
        };

        // Cache the search data
        {
            let cache = self.cache.write().await;
            cache.insert(cache_key, search_cache.clone());
        }

        Ok((search_cache, false))
    }

    /// Fetch all contacts with pagination.
    async fn fetch_all_contacts(&self) -> DexApiResult<Vec<Contact>> {
        const PAGE_SIZE: usize = 100;
        let mut all_contacts = Vec::new();
        let mut offset = 0;

        loop {
            let contacts = self.contact_repo.list(PAGE_SIZE, offset).await?;

            let count = contacts.len();
            all_contacts.extend(contacts);

            if count < PAGE_SIZE {
                break;
            }
            offset += PAGE_SIZE;
        }

        Ok(all_contacts)
    }

    /// Invalidate the search index cache.
    ///
    /// This should be called after any contact modifications to ensure
    /// fresh data is indexed on the next search.
    pub async fn invalidate_cache(&self) {
        let cache = self.cache.write().await;
        cache.remove(&"search_data".to_string());
        tracing::debug!("Search index cache invalidated");
    }

    /// Get the current cache TTL in seconds.
    pub fn cache_ttl_secs(&self) -> u64 {
        self.cache_ttl_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{AsyncDexClient, AsyncDexClientImpl, DexClient};
    use crate::config::Config;
    use crate::repositories::{DexContactRepository, DexNoteRepository, DexReminderRepository};

    #[test]
    fn test_search_params_default() {
        let params = SearchParams::default();
        assert_eq!(params.max_results, Some(10));
        assert_eq!(params.min_confidence, Some(50));
        assert_eq!(params.query, "");
    }

    #[test]
    fn test_search_tools_creation() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo = Arc::new(DexContactRepository::new(client.clone()));
        let note_repo = Arc::new(DexNoteRepository::new(client.clone()));
        let reminder_repo = Arc::new(DexReminderRepository::new(client));
        let cache_ttl_secs = 300;

        let tools = SearchTools::new(contact_repo, note_repo, reminder_repo, cache_ttl_secs);
        assert_eq!(tools.cache_ttl_secs(), 300);
    }

    #[tokio::test]
    async fn test_invalidate_cache() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo = Arc::new(DexContactRepository::new(client.clone()));
        let note_repo = Arc::new(DexNoteRepository::new(client.clone()));
        let reminder_repo = Arc::new(DexReminderRepository::new(client));
        let cache_ttl_secs = 300;

        let tools = SearchTools::new(contact_repo, note_repo, reminder_repo, cache_ttl_secs);

        // Insert something into cache
        {
            let cache = tools.cache.write().await;
            cache.insert(
                "search_data".to_string(),
                SearchCache {
                    index: Arc::new(FullTextSearchIndex::new()),
                    contacts: Arc::new(vec![]),
                },
            );
        }

        // Verify it exists
        {
            let cache = tools.cache.read().await;
            assert!(cache.contains_key(&"search_data".to_string()));
        }

        // Invalidate
        tools.invalidate_cache().await;

        // Verify it's gone
        {
            let cache = tools.cache.read().await;
            assert!(!cache.contains_key(&"search_data".to_string()));
        }
    }

    // Note: More comprehensive tests would require mocking the AsyncDexClient
    // Integration tests in tests/ directory would use mockito for full testing
}

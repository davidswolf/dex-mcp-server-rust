//! Contact discovery tools.
//!
//! Provides intelligent contact search with fuzzy matching, exact matching,
//! and result ranking. Includes caching for performance.

use crate::cache::TimedCache;
use crate::error::DexApiResult;
use crate::matching::{ContactMatcher, ContactQuery, MatchResult};
use crate::models::Contact;
use crate::repositories::ContactRepository;
use std::sync::Arc;

/// Contact discovery tools for finding and retrieving contacts.
pub struct ContactDiscoveryTools {
    contact_repo: Arc<dyn ContactRepository>,
    contact_cache: Arc<TimedCache<String, Vec<Contact>>>,
    matcher: ContactMatcher,
    cache_ttl_secs: u64,
}

/// Parameters for finding a contact.
#[derive(Debug, Clone)]
pub struct FindContactParams {
    /// Contact name to search for (fuzzy matching)
    pub name: Option<String>,

    /// Email address (exact matching)
    pub email: Option<String>,

    /// Phone number (normalized matching)
    pub phone: Option<String>,

    /// Social media URL (normalized matching)
    pub social_url: Option<String>,

    /// Company name (fuzzy matching boost)
    pub company: Option<String>,

    /// Maximum number of results to return (default: 5)
    pub max_results: Option<usize>,

    /// Minimum confidence threshold (0-100, default: 30)
    pub min_confidence: Option<u8>,
}

impl Default for FindContactParams {
    fn default() -> Self {
        Self {
            name: None,
            email: None,
            phone: None,
            social_url: None,
            company: None,
            max_results: Some(5),
            min_confidence: Some(30),
        }
    }
}

/// Response from find_contact with matches and confidence scores.
#[derive(Debug, Clone)]
pub struct FindContactResponse {
    /// Matched contacts with confidence scores
    pub matches: Vec<MatchResult>,

    /// Whether the results came from cache
    pub from_cache: bool,
}

impl ContactDiscoveryTools {
    /// Create new contact discovery tools.
    ///
    /// # Arguments
    /// * `contact_repo` - ContactRepository for data access
    /// * `cache_ttl_secs` - Cache time-to-live in seconds
    pub fn new(contact_repo: Arc<dyn ContactRepository>, cache_ttl_secs: u64) -> Self {
        Self {
            contact_repo,
            contact_cache: Arc::new(TimedCache::new(cache_ttl_secs)),
            matcher: ContactMatcher::new(),
            cache_ttl_secs,
        }
    }

    /// Find contacts using intelligent matching.
    ///
    /// This method searches for contacts using:
    /// - Fuzzy name matching (handles typos, variations)
    /// - Exact email matching
    /// - Normalized phone matching
    /// - Social URL matching
    /// - Company-based confidence boosting
    ///
    /// Results are ranked by confidence score and cached for performance.
    ///
    /// # Arguments
    /// * `params` - Search parameters
    ///
    /// # Returns
    /// A list of matching contacts with confidence scores
    pub async fn find_contact(&mut self, params: FindContactParams) -> DexApiResult<FindContactResponse> {
        // Get all contacts (with caching)
        let contacts = self.get_cached_contacts().await?;

        // If email is provided and we have no cache, try direct email search first
        let from_cache = self.contact_cache.contains_key(&"all_contacts".to_string());
        if !from_cache && params.email.is_some() {
            let results = self
                .contact_repo
                .search_by_email(params.email.as_ref().unwrap(), 10, 0).await?;
            if !results.is_empty() {
                // Found by email, return as high-confidence matches
                return Ok(FindContactResponse {
                    matches: results
                        .into_iter()
                        .map(|contact| MatchResult {
                            contact,
                            confidence: 100,
                            match_type: crate::matching::MatchType::ExactEmail,
                        })
                        .collect(),
                    from_cache: false,
                });
            }
        }

        // Perform fuzzy matching
        let max_results = params.max_results.unwrap_or(5);
        let min_confidence = params.min_confidence.unwrap_or(30);

        // Build query for the matcher
        let query = ContactQuery {
            name: params.name,
            email: params.email,
            phone: params.phone,
            company: params.company,
            social_url: params.social_url,
        };

        let matches = self
            .matcher
            .find_matches(&query, &contacts, max_results, min_confidence);

        Ok(FindContactResponse {
            matches,
            from_cache,
        })
    }

    /// Get detailed information about a specific contact.
    ///
    /// # Arguments
    /// * `contact_id` - The ID of the contact to retrieve
    ///
    /// # Returns
    /// The full contact details
    pub async fn get_contact_details(&self, contact_id: &str) -> DexApiResult<Contact> {
        self.contact_repo.get(contact_id).await
    }

    /// Get all contacts from cache or API.
    ///
    /// This method maintains a cache of all contacts to improve performance
    /// of repeated searches.
    async fn get_cached_contacts(&self) -> DexApiResult<Vec<Contact>> {
        let cache_key = "all_contacts".to_string();

        // Check cache first
        if let Some(contacts) = self.contact_cache.get(&cache_key) {
            return Ok(contacts);
        }

        // Cache miss - fetch from repository
        // Fetch in pages of 100 until we have all contacts
        let mut all_contacts = Vec::new();
        let mut offset = 0;
        const PAGE_SIZE: usize = 100;

        loop {
            let contacts = self.contact_repo.list(PAGE_SIZE, offset).await?;
            let count = contacts.len();
            all_contacts.extend(contacts);

            if count < PAGE_SIZE {
                // Last page
                break;
            }

            offset += PAGE_SIZE;
        }

        // Store in cache
        self.contact_cache.insert(cache_key, all_contacts.clone());

        Ok(all_contacts)
    }

    /// Invalidate the contact cache.
    ///
    /// This should be called after any contact modifications to ensure
    /// fresh data is fetched on the next search.
    pub fn invalidate_cache(&self) {
        self.contact_cache.remove(&"all_contacts".to_string());
    }

    /// Get the current cache TTL in seconds.
    pub fn cache_ttl_secs(&self) -> u64 {
        self.cache_ttl_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::client::{AsyncDexClient, AsyncDexClientImpl, DexClient};
    use crate::repositories::DexContactRepository;

    #[allow(dead_code)]
    fn create_test_contact(id: &str, name: &str, email: Option<&str>) -> Contact {
        let mut contact = Contact::new(id.to_string(), name.to_string());
        contact.email = email.map(String::from);
        contact
    }

    #[test]
    fn test_find_contact_params_default() {
        let params = FindContactParams::default();
        assert_eq!(params.max_results, Some(5));
        assert_eq!(params.min_confidence, Some(30));
        assert!(params.name.is_none());
        assert!(params.email.is_none());
    }

    #[test]
    fn test_contact_discovery_tools_creation() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo = Arc::new(DexContactRepository::new(client));
        let cache_ttl_secs = 300;

        let tools = ContactDiscoveryTools::new(contact_repo, cache_ttl_secs);
        assert_eq!(tools.cache_ttl_secs(), 300);
    }

    #[test]
    fn test_invalidate_cache() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo = Arc::new(DexContactRepository::new(client));
        let cache_ttl_secs = 300;

        let tools = ContactDiscoveryTools::new(contact_repo, cache_ttl_secs);

        // Insert something into cache
        tools
            .contact_cache
            .insert("all_contacts".to_string(), vec![]);
        assert!(tools
            .contact_cache
            .contains_key(&"all_contacts".to_string()));

        // Invalidate
        tools.invalidate_cache();
        assert!(!tools
            .contact_cache
            .contains_key(&"all_contacts".to_string()));
    }

    // Note: More comprehensive tests would require mocking the DexClient
    // These integration tests should be in tests/ directory with mockito
}

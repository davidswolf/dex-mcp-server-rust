//! Contact service layer.
//!
//! Business logic for contact discovery, search, and enrichment.

use crate::error::DexApiResult;
use crate::models::{Contact, SocialProfile};
use crate::tools::{
    ContactDiscoveryTools, ContactEnrichmentTools, EnrichContactParams, FindContactParams,
    FindContactResponse, SearchTools,
};
use crate::tools::search::{SearchParams, SearchResponse};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Parameters for enriching a contact.
#[derive(Debug, Clone, Default)]
pub struct ContactEnrichParams {
    pub contact_id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub social_profiles: Option<Vec<String>>,
}

/// Contact service trait for business operations.
#[async_trait]
pub trait ContactService: Send + Sync {
    /// Search contacts using full-text search with ranking.
    async fn search_full_text(
        &self,
        query: String,
        max_results: Option<usize>,
        min_confidence: Option<u8>,
    ) -> DexApiResult<SearchResponse>;

    /// Find contacts using intelligent matching (fuzzy name, exact email/phone, etc.).
    async fn find_contact(
        &self,
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        social_url: Option<String>,
        company: Option<String>,
    ) -> DexApiResult<FindContactResponse>;

    /// Get complete details for a specific contact.
    async fn get_contact_details(&self, contact_id: &str) -> DexApiResult<Contact>;

    /// Enrich a contact with new information.
    ///
    /// This performs intelligent merging of data and invalidates caches.
    async fn enrich_contact(&self, params: ContactEnrichParams) -> DexApiResult<Contact>;

    /// Invalidate the discovery cache.
    ///
    /// Should be called after any contact modification.
    async fn invalidate_cache(&self);
}

/// Default implementation of ContactService.
pub struct ContactServiceImpl {
    discovery_tools: Arc<RwLock<ContactDiscoveryTools>>,
    enrichment_tools: Arc<ContactEnrichmentTools>,
    search_tools: SearchTools,
}

/// Validation helper functions.
impl ContactServiceImpl {
    /// Validate search query.
    fn validate_search_query(query: &str) -> Result<(), String> {
        if query.trim().is_empty() {
            return Err("Search query cannot be empty".to_string());
        }
        if query.len() > 500 {
            return Err("Search query too long (max 500 characters)".to_string());
        }
        Ok(())
    }

    /// Validate email format.
    fn validate_email(email: &str) -> Result<(), String> {
        if !email.contains('@') || email.len() < 3 {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }

    /// Validate contact ID format.
    fn validate_contact_id(contact_id: &str) -> Result<(), String> {
        if contact_id.trim().is_empty() {
            return Err("Contact ID cannot be empty".to_string());
        }
        if contact_id.len() > 100 {
            return Err("Contact ID too long".to_string());
        }
        Ok(())
    }
}

impl ContactServiceImpl {
    /// Create a new contact service.
    pub fn new(
        discovery_tools: Arc<RwLock<ContactDiscoveryTools>>,
        enrichment_tools: Arc<ContactEnrichmentTools>,
        search_tools: SearchTools,
    ) -> Self {
        Self {
            discovery_tools,
            enrichment_tools,
            search_tools,
        }
    }
}

#[async_trait]
impl ContactService for ContactServiceImpl {
    async fn search_full_text(
        &self,
        query: String,
        max_results: Option<usize>,
        min_confidence: Option<u8>,
    ) -> DexApiResult<SearchResponse> {
        // Validate query
        Self::validate_search_query(&query).map_err(|e| {
            crate::error::DexApiError::InvalidRequest(e)
        })?;

        let search_params = SearchParams {
            query,
            max_results,
            min_confidence,
        };

        self.search_tools.search_full_text(search_params).await
    }

    async fn find_contact(
        &self,
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        social_url: Option<String>,
        company: Option<String>,
    ) -> DexApiResult<FindContactResponse> {
        // Validate email if provided
        if let Some(ref email_val) = email {
            Self::validate_email(email_val).map_err(|e| {
                crate::error::DexApiError::InvalidRequest(e)
            })?;
        }

        let find_params = FindContactParams {
            name,
            email,
            phone,
            social_url,
            company,
            max_results: Some(5),
            min_confidence: Some(30),
        };

        let mut discovery = self.discovery_tools.write().await;
        discovery.find_contact(find_params).await
    }

    async fn get_contact_details(&self, contact_id: &str) -> DexApiResult<Contact> {
        // Validate contact ID
        Self::validate_contact_id(contact_id).map_err(|e| {
            crate::error::DexApiError::InvalidRequest(e)
        })?;

        let discovery = self.discovery_tools.read().await;
        discovery.get_contact_details(contact_id).await
    }

    async fn enrich_contact(&self, params: ContactEnrichParams) -> DexApiResult<Contact> {
        // Validate contact ID
        Self::validate_contact_id(&params.contact_id).map_err(|e| {
            crate::error::DexApiError::InvalidRequest(e)
        })?;

        // Validate email if provided
        if let Some(ref email_val) = params.email {
            Self::validate_email(email_val).map_err(|e| {
                crate::error::DexApiError::InvalidRequest(e)
            })?;
        }

        // Convert social_profiles from strings to SocialProfile objects
        let social_profiles = params.social_profiles.map(|profiles| {
            profiles
                .into_iter()
                .map(|url| SocialProfile {
                    profile_type: "unknown".to_string(), // Will be inferred from URL
                    url,
                    username: None,
                })
                .collect()
        });

        let enrich_params = EnrichContactParams {
            contact_id: params.contact_id.clone(),
            first_name: None,
            last_name: None,
            email: params.email,
            phone: params.phone,
            company: params.company,
            title: params.title,
            website: None,
            location: None,
            birthday: None,
            notes: params.notes,
            additional_emails: None,
            additional_phones: None,
            tags: params.tags,
            social_profiles,
        };

        let updated_contact = self
            .enrichment_tools
            .enrich_contact(enrich_params)
            .await?;

        // Invalidate discovery cache after enrichment
        self.invalidate_cache().await;

        Ok(updated_contact)
    }

    async fn invalidate_cache(&self) {
        let discovery = self.discovery_tools.write().await;
        discovery.invalidate_cache();

        // Also invalidate search cache
        self.search_tools.invalidate_cache().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{AsyncDexClient, AsyncDexClientImpl, DexClient};
    use crate::config::Config;
    use crate::repositories::{
        ContactRepository, DexContactRepository, DexNoteRepository, DexReminderRepository,
        NoteRepository, ReminderRepository,
    };

    #[test]
    fn test_contact_service_creation() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo =
            Arc::new(DexContactRepository::new(client.clone())) as Arc<dyn ContactRepository>;
        let note_repo =
            Arc::new(DexNoteRepository::new(client.clone())) as Arc<dyn NoteRepository>;
        let reminder_repo =
            Arc::new(DexReminderRepository::new(client.clone())) as Arc<dyn ReminderRepository>;

        let discovery_tools = Arc::new(RwLock::new(ContactDiscoveryTools::new(
            contact_repo.clone(),
            300,
        )));
        let enrichment_tools = Arc::new(ContactEnrichmentTools::new(
            contact_repo.clone(),
            note_repo.clone(),
            reminder_repo.clone(),
        ));
        let search_tools = SearchTools::new(contact_repo, note_repo, reminder_repo, 300);

        let _service = ContactServiceImpl::new(discovery_tools, enrichment_tools, search_tools);
        // Just verify it constructs without panic
    }
}

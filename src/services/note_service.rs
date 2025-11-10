//! Note service layer.
//!
//! Business logic for note management and retrieval.

use crate::error::DexApiResult;
use crate::models::Note;
use crate::tools::{
    ContactEnrichmentTools, CreateNoteParams, HistoryFilterParams, RelationshipHistoryTools,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Note service trait for business operations.
#[async_trait]
pub trait NoteService: Send + Sync {
    /// Get all notes for a contact with optional filtering.
    async fn get_contact_notes(
        &self,
        contact_id: &str,
        date_from: Option<String>,
        limit: Option<usize>,
    ) -> DexApiResult<Vec<Note>>;

    /// Create a new note for a contact.
    async fn create_note(
        &self,
        contact_id: String,
        content: String,
        tags: Option<Vec<String>>,
    ) -> DexApiResult<Note>;
}

/// Default implementation of NoteService.
pub struct NoteServiceImpl {
    history_tools: Arc<RelationshipHistoryTools>,
    enrichment_tools: Arc<ContactEnrichmentTools>,
}

/// Validation helper functions.
impl NoteServiceImpl {
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

    /// Validate note content.
    fn validate_note_content(content: &str) -> Result<(), String> {
        if content.trim().is_empty() {
            return Err("Note content cannot be empty".to_string());
        }
        if content.len() > 10000 {
            return Err("Note content too long (max 10000 characters)".to_string());
        }
        Ok(())
    }
}

impl NoteServiceImpl {
    /// Create a new note service.
    pub fn new(
        history_tools: Arc<RelationshipHistoryTools>,
        enrichment_tools: Arc<ContactEnrichmentTools>,
    ) -> Self {
        Self {
            history_tools,
            enrichment_tools,
        }
    }
}

#[async_trait]
impl NoteService for NoteServiceImpl {
    async fn get_contact_notes(
        &self,
        contact_id: &str,
        date_from: Option<String>,
        limit: Option<usize>,
    ) -> DexApiResult<Vec<Note>> {
        // Validate contact ID
        Self::validate_contact_id(contact_id).map_err(crate::error::DexApiError::InvalidRequest)?;

        let filter = HistoryFilterParams {
            start_date: date_from,
            end_date: None,
            entry_types: None,
            limit,
        };

        self.history_tools
            .get_contact_notes(contact_id, Some(filter))
            .await
    }

    async fn create_note(
        &self,
        contact_id: String,
        content: String,
        tags: Option<Vec<String>>,
    ) -> DexApiResult<Note> {
        // Validate contact ID
        Self::validate_contact_id(&contact_id)
            .map_err(crate::error::DexApiError::InvalidRequest)?;

        // Validate note content
        Self::validate_note_content(&content).map_err(crate::error::DexApiError::InvalidRequest)?;

        let note_params = CreateNoteParams {
            contact_id,
            content,
            tags,
            source: Some("mcp".to_string()),
        };

        self.enrichment_tools.add_contact_note(note_params).await
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
    fn test_note_service_creation() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo =
            Arc::new(DexContactRepository::new(client.clone())) as Arc<dyn ContactRepository>;
        let note_repo = Arc::new(DexNoteRepository::new(client.clone())) as Arc<dyn NoteRepository>;
        let reminder_repo =
            Arc::new(DexReminderRepository::new(client.clone())) as Arc<dyn ReminderRepository>;

        let history_tools = Arc::new(RelationshipHistoryTools::new(
            contact_repo.clone(),
            note_repo.clone(),
            reminder_repo.clone(),
        ));
        let enrichment_tools = Arc::new(ContactEnrichmentTools::new(
            contact_repo,
            note_repo,
            reminder_repo,
        ));

        let _service = NoteServiceImpl::new(history_tools, enrichment_tools);
        // Just verify it constructs without panic
    }
}

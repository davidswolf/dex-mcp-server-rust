//! History service layer.
//!
//! Business logic for contact relationship history and timelines.

use crate::error::DexApiResult;
use crate::tools::{ContactHistoryResponse, HistoryFilterParams, RelationshipHistoryTools};
use async_trait::async_trait;
use std::sync::Arc;

/// History service trait for business operations.
#[async_trait]
pub trait HistoryService: Send + Sync {
    /// Get the complete relationship timeline for a contact.
    ///
    /// Returns a chronologically sorted timeline of notes and reminders.
    async fn get_contact_history(
        &self,
        contact_id: &str,
        date_from: Option<String>,
        date_to: Option<String>,
        include_notes: bool,
        include_reminders: bool,
    ) -> DexApiResult<ContactHistoryResponse>;
}

/// Default implementation of HistoryService.
pub struct HistoryServiceImpl {
    history_tools: Arc<RelationshipHistoryTools>,
}

/// Validation helper functions.
impl HistoryServiceImpl {
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

impl HistoryServiceImpl {
    /// Create a new history service.
    pub fn new(history_tools: Arc<RelationshipHistoryTools>) -> Self {
        Self { history_tools }
    }
}

#[async_trait]
impl HistoryService for HistoryServiceImpl {
    async fn get_contact_history(
        &self,
        contact_id: &str,
        date_from: Option<String>,
        date_to: Option<String>,
        include_notes: bool,
        include_reminders: bool,
    ) -> DexApiResult<ContactHistoryResponse> {
        // Validate contact ID
        Self::validate_contact_id(contact_id)
            .map_err(crate::error::DexApiError::InvalidRequest)?;

        let filter = HistoryFilterParams {
            start_date: date_from,
            end_date: date_to,
            entry_types: {
                let mut types = Vec::new();
                if include_notes {
                    types.push("note".to_string());
                }
                if include_reminders {
                    types.push("reminder".to_string());
                }
                if types.is_empty() {
                    None
                } else {
                    Some(types)
                }
            },
            limit: None,
        };

        self.history_tools
            .get_contact_history(contact_id, Some(filter))
            .await
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
    fn test_history_service_creation() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo =
            Arc::new(DexContactRepository::new(client.clone())) as Arc<dyn ContactRepository>;
        let note_repo = Arc::new(DexNoteRepository::new(client.clone())) as Arc<dyn NoteRepository>;
        let reminder_repo =
            Arc::new(DexReminderRepository::new(client.clone())) as Arc<dyn ReminderRepository>;

        let history_tools = Arc::new(RelationshipHistoryTools::new(
            contact_repo,
            note_repo,
            reminder_repo,
        ));

        let _service = HistoryServiceImpl::new(history_tools);
        // Just verify it constructs without panic
    }
}

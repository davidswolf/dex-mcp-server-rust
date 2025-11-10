//! Reminder service layer.
//!
//! Business logic for reminder management and retrieval.

use crate::error::DexApiResult;
use crate::models::Reminder;
use crate::tools::{
    ContactEnrichmentTools, CreateReminderParams, HistoryFilterParams, RelationshipHistoryTools,
};
use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;

/// Status filter for reminders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReminderStatus {
    /// Only active (not completed) reminders
    Active,
    /// Only completed reminders
    Completed,
    /// All reminders regardless of status
    All,
}

impl FromStr for ReminderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(ReminderStatus::Active),
            "completed" => Ok(ReminderStatus::Completed),
            "all" => Ok(ReminderStatus::All),
            _ => Ok(ReminderStatus::All), // Default to All for unknown values
        }
    }
}

/// Reminder service trait for business operations.
#[async_trait]
pub trait ReminderService: Send + Sync {
    /// Get all reminders for a contact with optional filtering.
    async fn get_contact_reminders(
        &self,
        contact_id: &str,
        date_from: Option<String>,
        status: Option<ReminderStatus>,
    ) -> DexApiResult<Vec<Reminder>>;

    /// Create a new reminder for a contact.
    async fn create_reminder(
        &self,
        contact_id: String,
        text: String,
        due_date: String,
        priority: Option<String>,
    ) -> DexApiResult<Reminder>;
}

/// Default implementation of ReminderService.
pub struct ReminderServiceImpl {
    history_tools: Arc<RelationshipHistoryTools>,
    enrichment_tools: Arc<ContactEnrichmentTools>,
}

/// Validation helper functions.
impl ReminderServiceImpl {
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

    /// Validate reminder text.
    fn validate_reminder_text(text: &str) -> Result<(), String> {
        if text.trim().is_empty() {
            return Err("Reminder text cannot be empty".to_string());
        }
        if text.len() > 500 {
            return Err("Reminder text too long (max 500 characters)".to_string());
        }
        Ok(())
    }

    /// Validate ISO 8601 date format (basic check).
    fn validate_date_format(date: &str) -> Result<(), String> {
        // Basic check: should contain at least YYYY-MM-DD
        if date.len() < 10 {
            return Err("Invalid date format (expected ISO 8601)".to_string());
        }
        Ok(())
    }
}

impl ReminderServiceImpl {
    /// Create a new reminder service.
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
impl ReminderService for ReminderServiceImpl {
    async fn get_contact_reminders(
        &self,
        contact_id: &str,
        date_from: Option<String>,
        status: Option<ReminderStatus>,
    ) -> DexApiResult<Vec<Reminder>> {
        // Validate contact ID
        Self::validate_contact_id(contact_id)
            .map_err(crate::error::DexApiError::InvalidRequest)?;

        let filter = HistoryFilterParams {
            start_date: date_from,
            end_date: None,
            entry_types: None,
            limit: None,
        };

        let reminders = self
            .history_tools
            .get_contact_reminders(contact_id, Some(filter))
            .await?;

        // Apply status filter
        let filtered_reminders = if let Some(status) = status {
            match status {
                ReminderStatus::Active => reminders
                    .into_iter()
                    .filter(|r| !r.completed)
                    .collect::<Vec<_>>(),
                ReminderStatus::Completed => reminders
                    .into_iter()
                    .filter(|r| r.completed)
                    .collect::<Vec<_>>(),
                ReminderStatus::All => reminders,
            }
        } else {
            reminders
        };

        Ok(filtered_reminders)
    }

    async fn create_reminder(
        &self,
        contact_id: String,
        text: String,
        due_date: String,
        priority: Option<String>,
    ) -> DexApiResult<Reminder> {
        // Validate contact ID
        Self::validate_contact_id(&contact_id)
            .map_err(crate::error::DexApiError::InvalidRequest)?;

        // Validate reminder text
        Self::validate_reminder_text(&text)
            .map_err(crate::error::DexApiError::InvalidRequest)?;

        // Validate due date format
        Self::validate_date_format(&due_date)
            .map_err(crate::error::DexApiError::InvalidRequest)?;

        let reminder_params = CreateReminderParams {
            contact_id,
            text,
            due_date,
            tags: None,
            priority,
        };

        self.enrichment_tools
            .create_contact_reminder(reminder_params)
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
    fn test_reminder_service_creation() {
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

        let _service = ReminderServiceImpl::new(history_tools, enrichment_tools);
        // Just verify it constructs without panic
    }

    #[test]
    fn test_reminder_status_from_str() {
        assert_eq!(
            "active".parse::<ReminderStatus>().unwrap(),
            ReminderStatus::Active
        );
        assert_eq!(
            "completed".parse::<ReminderStatus>().unwrap(),
            ReminderStatus::Completed
        );
        assert_eq!(
            "all".parse::<ReminderStatus>().unwrap(),
            ReminderStatus::All
        );
        assert_eq!(
            "unknown".parse::<ReminderStatus>().unwrap(),
            ReminderStatus::All
        );
    }
}

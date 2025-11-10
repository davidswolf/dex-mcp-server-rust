use async_trait::async_trait;
use std::sync::Arc;
use crate::client::AsyncDexClient;
use crate::repositories::traits::ReminderRepository;
use crate::models::Reminder;
use crate::error::DexApiResult;

/// Reminder repository implementation using Dex API client.
///
/// This repository delegates all operations to the AsyncDexClient,
/// providing a clean abstraction layer between business logic and
/// the underlying HTTP client.
pub struct DexReminderRepository {
    client: Arc<dyn AsyncDexClient>,
}

impl DexReminderRepository {
    /// Create a new DexReminderRepository with the given client.
    pub fn new(client: Arc<dyn AsyncDexClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ReminderRepository for DexReminderRepository {
    async fn get_for_contact(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Reminder>> {
        self.client.get_contact_reminders(contact_id, limit, offset).await
    }

    async fn create(&self, reminder: &Reminder) -> DexApiResult<Reminder> {
        self.client.create_reminder(reminder).await
    }

    async fn update(&self, id: &str, reminder: &Reminder) -> DexApiResult<Reminder> {
        self.client.update_reminder(id, reminder).await
    }

    async fn delete(&self, id: &str) -> DexApiResult<()> {
        self.client.delete_reminder(id).await
    }
}

//! Async wrapper around synchronous DexClient.
//!
//! This module provides an async interface to the synchronous DexClient by using
//! `tokio::task::spawn_blocking` to run HTTP operations on a dedicated thread pool,
//! preventing blocking of the async runtime.

use crate::client::DexClient;
use crate::error::{DexApiError, DexApiResult};
use crate::models::*;
use async_trait::async_trait;
use std::sync::Arc;

/// Async wrapper trait for CRM client operations.
///
/// This trait provides async versions of all DexClient methods,
/// internally using `tokio::task::spawn_blocking` to avoid
/// blocking the async runtime with synchronous HTTP calls.
#[async_trait]
pub trait AsyncDexClient: Send + Sync {
    async fn get_contact(&self, id: &str) -> DexApiResult<Contact>;
    async fn get_contacts(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>>;
    async fn search_contacts_by_email(&self, email: &str) -> DexApiResult<Vec<Contact>>;

    async fn get_contact_notes(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Note>>;
    async fn get_contact_reminders(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Reminder>>;

    async fn create_contact(&self, contact: &Contact) -> DexApiResult<Contact>;
    async fn update_contact(&self, id: &str, contact: &Contact) -> DexApiResult<Contact>;
    async fn delete_contact(&self, id: &str) -> DexApiResult<()>;

    async fn create_note(&self, note: &Note) -> DexApiResult<Note>;
    async fn update_note(&self, id: &str, note: &Note) -> DexApiResult<Note>;
    async fn delete_note(&self, id: &str) -> DexApiResult<()>;

    async fn create_reminder(&self, reminder: &Reminder) -> DexApiResult<Reminder>;
    async fn update_reminder(&self, id: &str, reminder: &Reminder) -> DexApiResult<Reminder>;
    async fn delete_reminder(&self, id: &str) -> DexApiResult<()>;
}

/// Async wrapper around synchronous DexClient.
///
/// Uses `tokio::task::spawn_blocking` to run synchronous HTTP
/// operations on a dedicated thread pool, preventing blocking
/// the async runtime.
#[derive(Clone)]
pub struct AsyncDexClientImpl {
    client: Arc<DexClient>,
}

impl AsyncDexClientImpl {
    pub fn new(client: DexClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

#[async_trait]
impl AsyncDexClient for AsyncDexClientImpl {
    async fn get_contact(&self, id: &str) -> DexApiResult<Contact> {
        let client = self.client.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || client.get_contact(&id))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn get_contacts(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>> {
        let client = self.client.clone();

        tokio::task::spawn_blocking(move || client.get_contacts(limit, offset))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn search_contacts_by_email(&self, email: &str) -> DexApiResult<Vec<Contact>> {
        let client = self.client.clone();
        let email = email.to_string();

        tokio::task::spawn_blocking(move || client.search_contacts_by_email(&email))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn get_contact_notes(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Note>> {
        let client = self.client.clone();
        let contact_id = contact_id.to_string();

        tokio::task::spawn_blocking(move || client.get_contact_notes(&contact_id, limit, offset))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn get_contact_reminders(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Reminder>> {
        let client = self.client.clone();
        let contact_id = contact_id.to_string();

        tokio::task::spawn_blocking(move || {
            client.get_contact_reminders(&contact_id, limit, offset)
        })
        .await
        .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn create_contact(&self, contact: &Contact) -> DexApiResult<Contact> {
        let client = self.client.clone();
        let contact = contact.clone();

        tokio::task::spawn_blocking(move || client.create_contact(&contact))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn update_contact(&self, id: &str, contact: &Contact) -> DexApiResult<Contact> {
        let client = self.client.clone();
        let id = id.to_string();
        let contact = contact.clone();

        tokio::task::spawn_blocking(move || client.update_contact(&id, &contact))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn delete_contact(&self, id: &str) -> DexApiResult<()> {
        let client = self.client.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || client.delete_contact(&id))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn create_note(&self, note: &Note) -> DexApiResult<Note> {
        let client = self.client.clone();
        let note = note.clone();

        tokio::task::spawn_blocking(move || client.create_note(&note))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn update_note(&self, id: &str, note: &Note) -> DexApiResult<Note> {
        let client = self.client.clone();
        let id = id.to_string();
        let note = note.clone();

        tokio::task::spawn_blocking(move || client.update_note(&id, &note))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn delete_note(&self, id: &str) -> DexApiResult<()> {
        let client = self.client.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || client.delete_note(&id))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn create_reminder(&self, reminder: &Reminder) -> DexApiResult<Reminder> {
        let client = self.client.clone();
        let reminder = reminder.clone();

        tokio::task::spawn_blocking(move || client.create_reminder(&reminder))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn update_reminder(&self, id: &str, reminder: &Reminder) -> DexApiResult<Reminder> {
        let client = self.client.clone();
        let id = id.to_string();
        let reminder = reminder.clone();

        tokio::task::spawn_blocking(move || client.update_reminder(&id, &reminder))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn delete_reminder(&self, id: &str) -> DexApiResult<()> {
        let client = self.client.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || client.delete_reminder(&id))
            .await
            .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[tokio::test]
    async fn test_async_client_creation() {
        let config = Config {
            dex_api_key: "test_key".to_string(),
            dex_api_url: "https://api.test.com".to_string(),
            cache_ttl_minutes: 30,
            request_timeout: 10,
            max_match_results: 5,
            match_confidence_threshold: 50,
            log_level: "error".to_string(),
        };
        let client = DexClient::new(&config);
        let async_client = AsyncDexClientImpl::new(client);

        // Should be able to clone
        let _cloned = async_client.clone();
    }
}

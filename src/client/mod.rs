//! HTTP client for interacting with the Dex Personal CRM API.
//!
//! This module provides a synchronous HTTP client that can be used from async contexts
//! via `tokio::task::spawn_blocking`. The client handles authentication, error mapping,
//! and pagination for the Dex API.

mod async_wrapper;
pub use async_wrapper::{AsyncDexClient, AsyncDexClientImpl};

use crate::config::Config;
use crate::error::{DexApiError, DexApiResult};
use crate::metrics::Metrics;
use crate::models::{Contact, Note, Reminder};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Response wrapper for paginated API endpoints.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    /// The data items for this page
    pub data: Vec<T>,

    /// Total number of items across all pages
    #[serde(default)]
    pub total: usize,

    /// Number of items per page
    #[serde(default)]
    pub per_page: usize,

    /// Current page number
    #[serde(default)]
    pub current_page: usize,

    /// Total number of pages
    #[serde(default)]
    pub total_pages: usize,
}

/// Response wrapper for the Dex contacts API endpoint.
#[derive(Debug, Deserialize)]
pub struct ContactsResponse {
    /// The list of contacts
    pub contacts: Vec<Contact>,

    /// Pagination metadata
    #[serde(default)]
    pub pagination: Option<PaginationInfo>,
}

/// Pagination metadata from Dex API.
#[derive(Debug, Deserialize)]
pub struct PaginationInfo {
    /// Total count information
    pub total: TotalCount,
}

/// Total count information.
#[derive(Debug, Deserialize)]
pub struct TotalCount {
    /// Total number of items
    pub count: usize,
}

/// Response wrapper for the Dex timeline_items API endpoint.
#[derive(Debug, Deserialize)]
pub struct TimelineItemsResponse {
    /// The list of timeline items (notes)
    pub timeline_items: Vec<Note>,
}

/// Response wrapper for the Dex reminders API endpoint.
#[derive(Debug, Deserialize)]
pub struct RemindersResponse {
    /// The list of reminders
    pub reminders: Vec<Reminder>,
}

/// HTTP client for the Dex Personal CRM API.
///
/// This client uses `ureq` for synchronous HTTP requests and can be called
/// from async contexts using `tokio::task::spawn_blocking`.
#[derive(Clone)]
pub struct DexClient {
    /// Base URL for the Dex API
    base_url: String,

    /// API key for authentication
    api_key: String,

    /// HTTP client agent
    agent: Arc<ureq::Agent>,

    /// Metrics collector
    metrics: Metrics,
}

impl DexClient {
    /// Create a new DexClient from configuration.
    pub fn new(config: &Config) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(config.request_timeout))
            .build();

        Self {
            base_url: config.dex_api_url.clone(),
            api_key: config.dex_api_key.clone(),
            agent: Arc::new(agent),
            metrics: Metrics::new(),
        }
    }

    /// Create a DexClient with a custom base URL (useful for testing).
    #[doc(hidden)]
    pub fn with_base_url(base_url: String, api_key: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(10))
            .build();

        Self {
            base_url,
            api_key,
            agent: Arc::new(agent),
            metrics: Metrics::new(),
        }
    }

    /// Get a reference to the metrics collector.
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    /// Build a full URL from a path.
    fn build_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }

    /// Execute a GET request with authentication.
    fn get(&self, path: &str) -> Result<ureq::Response, DexApiError> {
        let start = Instant::now();
        let url = self.build_url(path);

        let result = self
            .agent
            .get(&url)
            .set("x-hasura-dex-api-key", &self.api_key)
            .set("Content-Type", "application/json")
            .call()
            .map_err(|e| self.map_error(e));

        let duration = start.elapsed();
        if result.is_err() {
            self.metrics.record_http_error();
        }
        self.metrics.record_http_request(duration);

        result
    }

    /// Execute a POST request with authentication and JSON body.
    fn post(&self, path: &str, body: &serde_json::Value) -> Result<ureq::Response, DexApiError> {
        let start = Instant::now();
        let url = self.build_url(path);

        tracing::debug!("POST {}", url);
        tracing::debug!(
            "Request body: {}",
            serde_json::to_string_pretty(body).unwrap_or_else(|_| "<invalid json>".to_string())
        );

        let result = self
            .agent
            .post(&url)
            .set("x-hasura-dex-api-key", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|e| self.map_error(e));

        let duration = start.elapsed();
        match &result {
            Ok(response) => {
                tracing::debug!("POST {} - Success (status: {})", url, response.status());
                self.metrics.record_http_request(duration);
            }
            Err(e) => {
                tracing::error!("POST {} - Error: {:?}", url, e);
                self.metrics.record_http_error();
                self.metrics.record_http_request(duration);
            }
        }

        result
    }

    /// Execute a PUT request with authentication and JSON body.
    fn put(&self, path: &str, body: &serde_json::Value) -> Result<ureq::Response, DexApiError> {
        let start = Instant::now();
        let url = self.build_url(path);

        let result = self
            .agent
            .put(&url)
            .set("x-hasura-dex-api-key", &self.api_key)
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|e| self.map_error(e));

        let duration = start.elapsed();
        if result.is_err() {
            self.metrics.record_http_error();
        }
        self.metrics.record_http_request(duration);

        result
    }

    /// Execute a DELETE request with authentication.
    fn delete(&self, path: &str) -> Result<ureq::Response, DexApiError> {
        let start = Instant::now();
        let url = self.build_url(path);

        let result = self
            .agent
            .delete(&url)
            .set("x-hasura-dex-api-key", &self.api_key)
            .call()
            .map_err(|e| self.map_error(e));

        let duration = start.elapsed();
        if result.is_err() {
            self.metrics.record_http_error();
        }
        self.metrics.record_http_request(duration);

        result
    }

    /// Map a ureq error to a DexApiError.
    fn map_error(&self, error: ureq::Error) -> DexApiError {
        match error {
            ureq::Error::Status(code, response) => {
                let message = response
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());

                match code {
                    401 => DexApiError::Unauthorized,
                    404 => DexApiError::NotFound(message),
                    429 => DexApiError::RateLimitExceeded,
                    _ => DexApiError::ApiError {
                        status: code,
                        message,
                    },
                }
            }
            ureq::Error::Transport(transport) => {
                if transport.kind() == ureq::ErrorKind::ConnectionFailed {
                    DexApiError::HttpError("Connection failed".to_string())
                } else if transport.kind() == ureq::ErrorKind::Io {
                    DexApiError::Timeout
                } else {
                    DexApiError::HttpError(transport.to_string())
                }
            }
        }
    }

    // ========================= Contact Operations =========================

    /// Get all contacts with pagination.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of contacts to return (default: 100)
    /// * `offset` - Number of contacts to skip (for pagination)
    pub fn get_contacts(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>> {
        let path = format!("/contacts?limit={}&offset={}", limit, offset);
        let response = self.get(&path)?;
        let body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        // Parse using the Dex contacts response format
        let contacts_response: ContactsResponse =
            serde_json::from_str(&body).map_err(DexApiError::JsonError)?;

        // Populate computed fields for each contact
        let mut contacts = contacts_response.contacts;
        for contact in &mut contacts {
            contact.populate_computed_fields();
        }

        self.metrics.record_contacts_fetched(contacts.len());
        Ok(contacts)
    }

    /// Get a single contact by ID.
    pub fn get_contact(&self, contact_id: &str) -> DexApiResult<Contact> {
        let path = format!("/contacts/{}", contact_id);
        let response = self.get(&path)?;
        let body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        // Parse using the Dex contacts response format (same as get_contacts)
        let contacts_response: ContactsResponse =
            serde_json::from_str(&body).map_err(DexApiError::JsonError)?;

        // Extract the first (and only) contact from the array
        let mut contact = contacts_response
            .contacts
            .into_iter()
            .next()
            .ok_or_else(|| DexApiError::NotFound("Contact not found".to_string()))?;

        contact.populate_computed_fields();
        self.metrics.record_contacts_fetched(1);
        Ok(contact)
    }

    /// Create a new contact.
    pub fn create_contact(&self, contact: &Contact) -> DexApiResult<Contact> {
        use crate::models::contact::CreateContactRequest;

        let request = CreateContactRequest::from(contact);
        let body = serde_json::to_value(&request).map_err(DexApiError::JsonError)?;

        let response = self.post("/contacts", &body)?;
        let response_body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        // Parse the wrapped response
        let value: serde_json::Value =
            serde_json::from_str(&response_body).map_err(DexApiError::JsonError)?;

        let contact_data = value.get("insert_contacts_one").ok_or_else(|| {
            DexApiError::HttpError("Missing insert_contacts_one in API response".to_string())
        })?;

        // Deserialize the contact from the wrapped response
        let mut contact: Contact =
            serde_json::from_value(contact_data.clone()).map_err(DexApiError::JsonError)?;
        contact.populate_computed_fields();
        Ok(contact)
    }

    /// Update an existing contact.
    pub fn update_contact(&self, contact_id: &str, contact: &Contact) -> DexApiResult<Contact> {
        use crate::models::contact::UpdateContactRequest;

        let request = UpdateContactRequest::from_contact(contact, contact_id);
        let body = serde_json::to_value(&request).map_err(DexApiError::JsonError)?;

        let path = format!("/contacts/{}", contact_id);
        let response = self.put(&path, &body)?;
        let response_body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        // Parse the wrapped response
        let value: serde_json::Value =
            serde_json::from_str(&response_body).map_err(DexApiError::JsonError)?;

        let contact_data = value.get("update_contacts_by_pk").ok_or_else(|| {
            DexApiError::HttpError("Missing update_contacts_by_pk in API response".to_string())
        })?;

        // Deserialize the contact from the wrapped response
        let mut contact: Contact =
            serde_json::from_value(contact_data.clone()).map_err(DexApiError::JsonError)?;
        contact.populate_computed_fields();
        Ok(contact)
    }

    /// Delete a contact.
    pub fn delete_contact(&self, contact_id: &str) -> DexApiResult<()> {
        let path = format!("/contacts/{}", contact_id);
        self.delete(&path)?;
        Ok(())
    }

    /// Search contacts by email.
    pub fn search_contacts_by_email(&self, email: &str) -> DexApiResult<Vec<Contact>> {
        let path = format!("/contacts/search?email={}", urlencoding::encode(email));
        let response = self.get(&path)?;
        let body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        let mut contacts =
            if let Ok(paginated) = serde_json::from_str::<PaginatedResponse<Contact>>(&body) {
                paginated.data
            } else {
                serde_json::from_str::<Vec<Contact>>(&body).map_err(DexApiError::JsonError)?
            };

        for contact in &mut contacts {
            contact.populate_computed_fields();
        }

        self.metrics.record_contacts_fetched(contacts.len());
        Ok(contacts)
    }

    // ========================= Note Operations =========================

    /// Get all notes for a contact.
    pub fn get_contact_notes(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Note>> {
        let path = format!(
            "/timeline_items/contacts/{}?limit={}&offset={}",
            contact_id, limit, offset
        );
        let response = self.get(&path)?;
        let body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        // Parse using the Dex timeline_items response format
        let timeline_response: TimelineItemsResponse =
            serde_json::from_str(&body).map_err(DexApiError::JsonError)?;

        let notes = timeline_response.timeline_items;
        self.metrics.record_notes_fetched(notes.len());
        Ok(notes)
    }

    /// Get a single note by ID.
    pub fn get_note(&self, note_id: &str) -> DexApiResult<Note> {
        let path = format!("/notes/{}", note_id);
        let response = self.get(&path)?;
        let body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;
        serde_json::from_str::<Note>(&body).map_err(DexApiError::JsonError)
    }

    /// Create a new note for a contact.
    pub fn create_note(&self, note: &Note) -> DexApiResult<Note> {
        tracing::info!("Creating note for contact: {}", note.contact_id);

        // Convert to CreateNoteRequest to properly serialize timeline_event structure
        let request = crate::models::CreateNoteRequest::from(note);
        let body = serde_json::to_value(&request).map_err(DexApiError::JsonError)?;

        tracing::debug!(
            "Note request payload: {}",
            serde_json::to_string_pretty(&body).unwrap_or_else(|_| "<invalid>".to_string())
        );

        // Notes are created via the timeline_items endpoint
        let response = self.post("/timeline_items", &body)?;
        let response_body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        tracing::debug!("Note creation response: {}", response_body);

        // The API wraps the response in insert_timeline_items_one
        // Parse as raw JSON first to extract what we need
        let value: serde_json::Value =
            serde_json::from_str(&response_body).map_err(DexApiError::JsonError)?;

        let timeline_item = value.get("insert_timeline_items_one").ok_or_else(|| {
            DexApiError::HttpError("Missing insert_timeline_items_one in API response".to_string())
        })?;

        // Extract the note details
        let id = timeline_item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let content = timeline_item
            .get("note")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let event_time = timeline_item
            .get("event_time")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract contact ID from timeline_items_contacts
        let contact_id = timeline_item
            .get("timeline_items_contacts")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("contact"))
            .and_then(|contact| contact.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or(&note.contact_id) // Fallback to original
            .to_string();

        let created_note = Note {
            id,
            contact_id,
            content,
            created_at: event_time,
            updated_at: None,
            tags: Vec::new(),
            source: note.source.clone(),
        };

        tracing::info!("Note created successfully with id: {}", created_note.id);

        Ok(created_note)
    }

    /// Update an existing note (timeline item).
    pub fn update_note(&self, note_id: &str, note: &Note) -> DexApiResult<Note> {
        use crate::models::note::UpdateNoteRequest;

        let request = UpdateNoteRequest::from(note);
        let body = serde_json::to_value(&request).map_err(DexApiError::JsonError)?;

        let path = format!("/timeline_items/{}", note_id);
        let response = self.put(&path, &body)?;
        let response_body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        // Parse the wrapped response
        let value: serde_json::Value =
            serde_json::from_str(&response_body).map_err(DexApiError::JsonError)?;

        let timeline_item = value.get("update_timeline_items_by_pk").ok_or_else(|| {
            DexApiError::HttpError(
                "Missing update_timeline_items_by_pk in API response".to_string(),
            )
        })?;

        // Extract fields from the response
        let id = timeline_item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let content = timeline_item
            .get("note")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract contact_id from contact_ids array
        let contact_id = timeline_item
            .get("contact_ids")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("contact_id"))
            .and_then(|v| v.as_str())
            .unwrap_or(&note.contact_id)
            .to_string();

        // Build the Note object
        let updated_note = Note {
            id,
            contact_id,
            content,
            created_at: note.created_at.clone(), // Preserve original created_at
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            tags: note.tags.clone(),
            source: note.source.clone(),
        };

        Ok(updated_note)
    }

    /// Delete a note (timeline item).
    pub fn delete_note(&self, note_id: &str) -> DexApiResult<()> {
        let path = format!("/timeline_items/{}", note_id);
        self.delete(&path)?;
        Ok(())
    }

    // ========================= Reminder Operations =========================

    /// Get all reminders for a contact.
    /// Note: The Dex API doesn't have a direct endpoint for contact reminders,
    /// so we fetch all reminders and filter by contact_id.
    /// This implementation uses proper pagination to handle large reminder sets.
    pub fn get_contact_reminders(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Reminder>> {
        const PAGE_SIZE: usize = 100;
        let mut all_filtered_reminders = Vec::new();
        let mut current_offset = 0;
        let needed_count = offset + limit;

        // Keep fetching pages until we have enough filtered results or no more data
        loop {
            let path = format!("/reminders?limit={}&offset={}", PAGE_SIZE, current_offset);
            let response = self.get(&path)?;
            let body = response
                .into_string()
                .map_err(|e| DexApiError::HttpError(e.to_string()))?;

            // Parse using the Dex reminders response format
            let reminders_response: RemindersResponse =
                serde_json::from_str(&body).map_err(DexApiError::JsonError)?;

            let page_reminders = reminders_response.reminders;
            let fetched_count = page_reminders.len();

            // Filter this page for the target contact
            let filtered_page: Vec<Reminder> = page_reminders
                .into_iter()
                .filter(|r| r.contact_id == contact_id)
                .collect();

            all_filtered_reminders.extend(filtered_page);

            // Stop if we have enough results or if we received fewer than requested (no more pages)
            if all_filtered_reminders.len() >= needed_count || fetched_count < PAGE_SIZE {
                break;
            }

            current_offset += PAGE_SIZE;
        }

        // Apply offset and limit to the filtered results
        let result: Vec<Reminder> = all_filtered_reminders
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        self.metrics.record_reminders_fetched(result.len());
        Ok(result)
    }

    /// Get a single reminder by ID.
    pub fn get_reminder(&self, reminder_id: &str) -> DexApiResult<Reminder> {
        let path = format!("/reminders/{}", reminder_id);
        let response = self.get(&path)?;
        let body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;
        serde_json::from_str::<Reminder>(&body).map_err(DexApiError::JsonError)
    }

    /// Create a new reminder for a contact.
    pub fn create_reminder(&self, reminder: &Reminder) -> DexApiResult<Reminder> {
        tracing::info!(
            "Creating reminder for contact: {}, due: {}",
            reminder.contact_id,
            reminder.due_date
        );

        // Convert to CreateReminderRequest to properly serialize contact_ids
        let request = crate::models::CreateReminderRequest::from(reminder);
        let body = serde_json::to_value(&request).map_err(DexApiError::JsonError)?;

        tracing::debug!(
            "Reminder request payload: {}",
            serde_json::to_string_pretty(&body).unwrap_or_else(|_| "<invalid>".to_string())
        );

        let response = self.post("/reminders", &body)?;
        let response_body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        // Parse the wrapped response
        let value: serde_json::Value =
            serde_json::from_str(&response_body).map_err(DexApiError::JsonError)?;

        let reminder_item = value.get("insert_reminders_one").ok_or_else(|| {
            DexApiError::HttpError("Missing insert_reminders_one in API response".to_string())
        })?;

        // Extract fields from the response
        let id = reminder_item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let text = reminder_item
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let due_date = reminder_item
            .get("due_at_date")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let completed = reminder_item
            .get("is_complete")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Extract contact_id from contact_ids array
        let contact_id = reminder_item
            .get("contact_ids")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("contact_id"))
            .and_then(|v| v.as_str())
            .unwrap_or(&reminder.contact_id)
            .to_string();

        // Build the Reminder object
        let created_reminder = Reminder {
            id,
            contact_id,
            text,
            due_date,
            completed,
            completed_at: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
            tags: reminder.tags.clone(),
            priority: reminder.priority.clone(),
        };

        tracing::info!(
            "Reminder created successfully with id: {}",
            created_reminder.id
        );

        Ok(created_reminder)
    }

    /// Update an existing reminder.
    pub fn update_reminder(
        &self,
        reminder_id: &str,
        reminder: &Reminder,
    ) -> DexApiResult<Reminder> {
        use crate::models::reminder::UpdateReminderRequest;

        let request = UpdateReminderRequest::from(reminder);
        let body = serde_json::to_value(&request).map_err(DexApiError::JsonError)?;

        let path = format!("/reminders/{}", reminder_id);
        let response = self.put(&path, &body)?;
        let response_body = response
            .into_string()
            .map_err(|e| DexApiError::HttpError(e.to_string()))?;

        // Parse the wrapped response
        let value: serde_json::Value =
            serde_json::from_str(&response_body).map_err(DexApiError::JsonError)?;

        let reminder_item = value.get("update_reminders_by_pk").ok_or_else(|| {
            DexApiError::HttpError("Missing update_reminders_by_pk in API response".to_string())
        })?;

        // Extract fields from the response
        let id = reminder_item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let text = reminder_item
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let due_date = reminder_item
            .get("due_at_date")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let completed = reminder_item
            .get("is_complete")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Extract contact_id from reminders_contacts array
        let contact_id = reminder_item
            .get("reminders_contacts")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("contact_id"))
            .and_then(|v| v.as_str())
            .unwrap_or(&reminder.contact_id)
            .to_string();

        // Build the Reminder object
        let updated_reminder = Reminder {
            id,
            contact_id,
            text,
            due_date,
            completed,
            completed_at: reminder.completed_at.clone(),
            created_at: reminder.created_at.clone(),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            tags: reminder.tags.clone(),
            priority: reminder.priority.clone(),
        };

        Ok(updated_reminder)
    }

    /// Delete a reminder.
    pub fn delete_reminder(&self, reminder_id: &str) -> DexApiResult<()> {
        let path = format!("/reminders/{}", reminder_id);
        self.delete(&path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let client = DexClient::with_base_url(
            "https://api.example.com".to_string(),
            "test-key".to_string(),
        );

        assert_eq!(
            client.build_url("/contacts"),
            "https://api.example.com/contacts"
        );

        assert_eq!(
            client.build_url("contacts"),
            "https://api.example.com/contacts"
        );

        let client_with_slash = DexClient::with_base_url(
            "https://api.example.com/".to_string(),
            "test-key".to_string(),
        );

        assert_eq!(
            client_with_slash.build_url("/contacts"),
            "https://api.example.com/contacts"
        );
    }

    #[test]
    fn test_client_creation() {
        let config = Config {
            dex_api_url: "https://api.getdex.com".to_string(),
            dex_api_key: "test-key-123".to_string(),
            cache_ttl_minutes: 30,
            request_timeout: 10,
            max_match_results: 5,
            match_confidence_threshold: 30,
            log_level: "error".to_string(),
        };

        let client = DexClient::new(&config);
        assert_eq!(client.base_url, "https://api.getdex.com");
        assert_eq!(client.api_key, "test-key-123");
    }
}

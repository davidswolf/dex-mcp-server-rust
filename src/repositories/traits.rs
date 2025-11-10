use crate::error::DexApiResult;
use crate::models::*;
use async_trait::async_trait;

/// Repository for managing contacts.
///
/// Provides abstraction over contact storage and retrieval,
/// enabling different implementations (API client, mock, cached).
#[async_trait]
pub trait ContactRepository: Send + Sync {
    /// Retrieve a single contact by ID.
    async fn get(&self, id: &str) -> DexApiResult<Contact>;

    /// Retrieve multiple contacts with pagination.
    async fn list(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>>;

    /// Search contacts by email address.
    async fn search_by_email(
        &self,
        email: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Contact>>;

    /// Search contacts by name.
    async fn search_by_name(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Contact>>;

    /// Create a new contact.
    async fn create(&self, contact: &Contact) -> DexApiResult<Contact>;

    /// Update an existing contact.
    async fn update(&self, id: &str, contact: &Contact) -> DexApiResult<Contact>;

    /// Delete a contact.
    async fn delete(&self, id: &str) -> DexApiResult<()>;
}

/// Repository for managing notes.
#[async_trait]
pub trait NoteRepository: Send + Sync {
    /// Get notes for a specific contact.
    async fn get_for_contact(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Note>>;

    /// Create a new note.
    async fn create(&self, note: &Note) -> DexApiResult<Note>;

    /// Update an existing note.
    async fn update(&self, id: &str, note: &Note) -> DexApiResult<Note>;

    /// Delete a note.
    async fn delete(&self, id: &str) -> DexApiResult<()>;
}

/// Repository for managing reminders.
#[async_trait]
pub trait ReminderRepository: Send + Sync {
    /// Get reminders for a specific contact.
    async fn get_for_contact(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Reminder>>;

    /// Create a new reminder.
    async fn create(&self, reminder: &Reminder) -> DexApiResult<Reminder>;

    /// Update an existing reminder.
    async fn update(&self, id: &str, reminder: &Reminder) -> DexApiResult<Reminder>;

    /// Delete a reminder.
    async fn delete(&self, id: &str) -> DexApiResult<()>;
}

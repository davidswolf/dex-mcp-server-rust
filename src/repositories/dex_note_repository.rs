use crate::client::AsyncDexClient;
use crate::error::DexApiResult;
use crate::models::Note;
use crate::repositories::traits::NoteRepository;
use async_trait::async_trait;
use std::sync::Arc;

/// Note repository implementation using Dex API client.
///
/// This repository delegates all operations to the AsyncDexClient,
/// providing a clean abstraction layer between business logic and
/// the underlying HTTP client.
pub struct DexNoteRepository {
    client: Arc<dyn AsyncDexClient>,
}

impl DexNoteRepository {
    /// Create a new DexNoteRepository with the given client.
    pub fn new(client: Arc<dyn AsyncDexClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl NoteRepository for DexNoteRepository {
    async fn get_for_contact(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Note>> {
        self.client
            .get_contact_notes(contact_id, limit, offset)
            .await
    }

    async fn create(&self, note: &Note) -> DexApiResult<Note> {
        self.client.create_note(note).await
    }

    async fn update(&self, id: &str, note: &Note) -> DexApiResult<Note> {
        self.client.update_note(id, note).await
    }

    async fn delete(&self, id: &str) -> DexApiResult<()> {
        self.client.delete_note(id).await
    }
}

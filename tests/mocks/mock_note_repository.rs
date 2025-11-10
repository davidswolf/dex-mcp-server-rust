use async_trait::async_trait;
use dex_mcp_server::error::{DexApiError, DexApiResult};
use dex_mcp_server::models::Note;
use dex_mcp_server::repositories::NoteRepository;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock note repository for testing.
#[allow(dead_code)]
#[derive(Clone)]
pub struct MockNoteRepository {
    notes: Arc<Mutex<HashMap<String, Note>>>,
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

#[allow(dead_code)]
impl MockNoteRepository {
    pub fn new() -> Self {
        Self {
            notes: Arc::new(Mutex::new(HashMap::new())),
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_note(&self, note: Note) {
        let mut notes = self.notes.lock().unwrap();
        notes.insert(note.id.clone(), note);
    }

    pub fn add_notes(&self, notes_list: Vec<Note>) {
        let mut notes = self.notes.lock().unwrap();
        for note in notes_list {
            notes.insert(note.id.clone(), note);
        }
    }

    pub fn get_call_count(&self, method: &str) -> usize {
        let counts = self.call_counts.lock().unwrap();
        *counts.get(method).unwrap_or(&0)
    }

    pub fn reset_call_counts(&self) {
        let mut counts = self.call_counts.lock().unwrap();
        counts.clear();
    }

    pub fn clear(&self) {
        let mut notes = self.notes.lock().unwrap();
        notes.clear();
    }

    fn track_call(&self, method: &str) {
        let mut counts = self.call_counts.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
}

impl Default for MockNoteRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NoteRepository for MockNoteRepository {
    async fn get_for_contact(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Note>> {
        self.track_call("get_for_contact");

        let notes = self.notes.lock().unwrap();
        let result: Vec<Note> = notes
            .values()
            .filter(|note| note.contact_id == contact_id)
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn create(&self, note: &Note) -> DexApiResult<Note> {
        self.track_call("create");

        let mut notes = self.notes.lock().unwrap();

        if notes.contains_key(&note.id) {
            return Err(DexApiError::InvalidRequest(format!(
                "Note with ID {} already exists",
                note.id
            )));
        }

        notes.insert(note.id.clone(), note.clone());
        Ok(note.clone())
    }

    async fn update(&self, id: &str, note: &Note) -> DexApiResult<Note> {
        self.track_call("update");

        let mut notes = self.notes.lock().unwrap();

        if !notes.contains_key(id) {
            return Err(DexApiError::NotFound(format!("Note {} not found", id)));
        }

        notes.insert(id.to_string(), note.clone());
        Ok(note.clone())
    }

    async fn delete(&self, id: &str) -> DexApiResult<()> {
        self.track_call("delete");

        let mut notes = self.notes.lock().unwrap();

        if !notes.contains_key(id) {
            return Err(DexApiError::NotFound(format!("Note {} not found", id)));
        }

        notes.remove(id);
        Ok(())
    }
}

//! Relationship history tools.
//!
//! Provides access to contact interaction history including notes,
//! reminders, and aggregated timelines.

use crate::error::DexApiResult;
use crate::models::{Contact, Note, Reminder};
use crate::repositories::{ContactRepository, NoteRepository, ReminderRepository};
use std::sync::Arc;

/// Relationship history tools for accessing contact interactions.
pub struct RelationshipHistoryTools {
    contact_repo: Arc<dyn ContactRepository>,
    note_repo: Arc<dyn NoteRepository>,
    reminder_repo: Arc<dyn ReminderRepository>,
}

/// A timeline entry combining notes and reminders.
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineEntry {
    /// A note entry
    Note(Note),
    /// A reminder entry
    Reminder(Reminder),
}

impl TimelineEntry {
    /// Get the timestamp for sorting.
    pub fn timestamp(&self) -> &str {
        match self {
            TimelineEntry::Note(note) => &note.created_at,
            // Reminders don't have created_at from API, use due_date instead
            TimelineEntry::Reminder(reminder) => &reminder.due_date,
        }
    }

    /// Get the entry type as a string.
    pub fn entry_type(&self) -> &'static str {
        match self {
            TimelineEntry::Note(_) => "note",
            TimelineEntry::Reminder(_) => "reminder",
        }
    }
}

/// Parameters for filtering contact history.
#[derive(Debug, Clone, Default)]
pub struct HistoryFilterParams {
    /// Start date (ISO 8601 format)
    pub start_date: Option<String>,

    /// End date (ISO 8601 format)
    pub end_date: Option<String>,

    /// Entry types to include (e.g., ["note", "reminder"])
    pub entry_types: Option<Vec<String>>,

    /// Maximum number of entries to return
    pub limit: Option<usize>,
}

/// Response from get_contact_history.
#[derive(Debug, Clone)]
pub struct ContactHistoryResponse {
    /// The contact
    pub contact: Contact,

    /// Timeline entries sorted chronologically (newest first)
    pub timeline: Vec<TimelineEntry>,

    /// Total number of entries
    pub total_entries: usize,
}

impl RelationshipHistoryTools {
    /// Create new relationship history tools.
    ///
    /// # Arguments
    /// * `contact_repo` - ContactRepository for contact data access
    /// * `note_repo` - NoteRepository for note data access
    /// * `reminder_repo` - ReminderRepository for reminder data access
    pub fn new(
        contact_repo: Arc<dyn ContactRepository>,
        note_repo: Arc<dyn NoteRepository>,
        reminder_repo: Arc<dyn ReminderRepository>,
    ) -> Self {
        Self {
            contact_repo,
            note_repo,
            reminder_repo,
        }
    }

    /// Get the complete interaction history for a contact.
    ///
    /// This aggregates notes and reminders into a unified timeline,
    /// sorted chronologically (newest first).
    ///
    /// # Arguments
    /// * `contact_id` - ID of the contact
    /// * `filter` - Optional filtering parameters
    ///
    /// # Returns
    /// Complete timeline with notes and reminders
    pub async fn get_contact_history(
        &self,
        contact_id: &str,
        filter: Option<HistoryFilterParams>,
    ) -> DexApiResult<ContactHistoryResponse> {
        // Fetch contact details
        let contact = self.contact_repo.get(contact_id).await?;

        // Fetch notes and reminders (asynchronously)
        let notes = self.fetch_all_notes(contact_id).await?;
        let reminders = self.fetch_all_reminders(contact_id).await?;

        // Build timeline
        let mut timeline: Vec<TimelineEntry> = Vec::new();

        // Add notes
        for note in notes {
            timeline.push(TimelineEntry::Note(note));
        }

        // Add reminders
        for reminder in reminders {
            timeline.push(TimelineEntry::Reminder(reminder));
        }

        // Apply filters if provided
        if let Some(filter) = filter {
            timeline = self.apply_filters(timeline, &filter);
        }

        // Sort by timestamp (newest first)
        timeline.sort_by(|a, b| b.timestamp().cmp(a.timestamp()));

        let total_entries = timeline.len();

        Ok(ContactHistoryResponse {
            contact,
            timeline,
            total_entries,
        })
    }

    /// Get only notes for a contact.
    ///
    /// # Arguments
    /// * `contact_id` - ID of the contact
    /// * `filter` - Optional filtering parameters
    ///
    /// # Returns
    /// Filtered and sorted notes
    pub async fn get_contact_notes(
        &self,
        contact_id: &str,
        filter: Option<HistoryFilterParams>,
    ) -> DexApiResult<Vec<Note>> {
        let mut notes = self.fetch_all_notes(contact_id).await?;

        // Apply date filters if provided
        if let Some(filter) = filter {
            notes = self.filter_notes(notes, &filter);
        }

        // Sort by created date (newest first)
        notes.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(notes)
    }

    /// Get only reminders for a contact.
    ///
    /// # Arguments
    /// * `contact_id` - ID of the contact
    /// * `filter` - Optional filtering parameters
    ///
    /// # Returns
    /// Filtered and sorted reminders
    pub async fn get_contact_reminders(
        &self,
        contact_id: &str,
        filter: Option<HistoryFilterParams>,
    ) -> DexApiResult<Vec<Reminder>> {
        let mut reminders = self.fetch_all_reminders(contact_id).await?;

        // Apply date filters if provided
        if let Some(filter) = filter {
            reminders = self.filter_reminders(reminders, &filter);
        }

        // Sort by due date (nearest first)
        reminders.sort_by(|a, b| a.due_date.cmp(&b.due_date));

        Ok(reminders)
    }

    /// Fetch all notes for a contact, handling pagination.
    async fn fetch_all_notes(&self, contact_id: &str) -> DexApiResult<Vec<Note>> {
        let mut all_notes = Vec::new();
        let mut offset = 0;
        const PAGE_SIZE: usize = 100;

        loop {
            let notes = self
                .note_repo
                .get_for_contact(contact_id, PAGE_SIZE, offset).await?;
            let count = notes.len();
            all_notes.extend(notes);

            if count < PAGE_SIZE {
                break;
            }

            offset += PAGE_SIZE;
        }

        Ok(all_notes)
    }

    /// Fetch all reminders for a contact, handling pagination.
    async fn fetch_all_reminders(&self, contact_id: &str) -> DexApiResult<Vec<Reminder>> {
        let mut all_reminders = Vec::new();
        let mut offset = 0;
        const PAGE_SIZE: usize = 100;

        loop {
            let reminders = self
                .reminder_repo
                .get_for_contact(contact_id, PAGE_SIZE, offset).await?;
            let count = reminders.len();
            all_reminders.extend(reminders);

            if count < PAGE_SIZE {
                break;
            }

            offset += PAGE_SIZE;
        }

        Ok(all_reminders)
    }

    /// Apply filters to timeline entries.
    fn apply_filters(
        &self,
        timeline: Vec<TimelineEntry>,
        filter: &HistoryFilterParams,
    ) -> Vec<TimelineEntry> {
        let mut filtered = timeline;

        // Filter by entry type
        if let Some(ref types) = filter.entry_types {
            filtered.retain(|entry| types.contains(&entry.entry_type().to_string()));
        }

        // Filter by date range
        if let Some(ref start_date) = filter.start_date {
            filtered.retain(|entry| entry.timestamp() >= start_date.as_str());
        }

        if let Some(ref end_date) = filter.end_date {
            filtered.retain(|entry| entry.timestamp() <= end_date.as_str());
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            filtered.truncate(limit);
        }

        filtered
    }

    /// Filter notes by date range and limit.
    fn filter_notes(&self, notes: Vec<Note>, filter: &HistoryFilterParams) -> Vec<Note> {
        let mut filtered = notes;

        if let Some(ref start_date) = filter.start_date {
            filtered.retain(|note| note.created_at.as_str() >= start_date.as_str());
        }

        if let Some(ref end_date) = filter.end_date {
            filtered.retain(|note| note.created_at.as_str() <= end_date.as_str());
        }

        if let Some(limit) = filter.limit {
            filtered.truncate(limit);
        }

        filtered
    }

    /// Filter reminders by date range and limit.
    fn filter_reminders(
        &self,
        reminders: Vec<Reminder>,
        filter: &HistoryFilterParams,
    ) -> Vec<Reminder> {
        let mut filtered = reminders;

        if let Some(ref start_date) = filter.start_date {
            filtered.retain(|reminder| reminder.created_at.as_str() >= start_date.as_str());
        }

        if let Some(ref end_date) = filter.end_date {
            filtered.retain(|reminder| reminder.created_at.as_str() <= end_date.as_str());
        }

        if let Some(limit) = filter.limit {
            filtered.truncate(limit);
        }

        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::client::{AsyncDexClient, AsyncDexClientImpl, DexClient};
    use crate::repositories::{DexContactRepository, DexNoteRepository, DexReminderRepository};

    fn create_test_note(id: &str, created_at: &str) -> Note {
        Note::new(
            id.to_string(),
            "contact1".to_string(),
            "Test note".to_string(),
            created_at.to_string(),
        )
    }

    fn create_test_reminder(id: &str, created_at: &str, due_date: &str) -> Reminder {
        Reminder::new(
            id.to_string(),
            "contact1".to_string(),
            "Test reminder".to_string(),
            due_date.to_string(),
            created_at.to_string(),
        )
    }

    #[test]
    fn test_timeline_entry_timestamp() {
        let note = create_test_note("note1", "2024-01-01T10:00:00Z");
        let entry = TimelineEntry::Note(note);
        assert_eq!(entry.timestamp(), "2024-01-01T10:00:00Z");
        assert_eq!(entry.entry_type(), "note");

        let reminder = create_test_reminder("rem1", "2024-01-02T10:00:00Z", "2024-01-15T10:00:00Z");
        let entry = TimelineEntry::Reminder(reminder);
        // Reminders use due_date as timestamp since API doesn't provide created_at
        assert_eq!(entry.timestamp(), "2024-01-15T10:00:00Z");
        assert_eq!(entry.entry_type(), "reminder");
    }

    #[test]
    fn test_relationship_history_tools_creation() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo = Arc::new(DexContactRepository::new(client.clone()));
        let note_repo = Arc::new(DexNoteRepository::new(client.clone()));
        let reminder_repo = Arc::new(DexReminderRepository::new(client));

        let _tools = RelationshipHistoryTools::new(contact_repo, note_repo, reminder_repo);
        // Just verify it constructs without panic
    }

    #[test]
    fn test_filter_notes_by_date() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo = Arc::new(DexContactRepository::new(client.clone()));
        let note_repo = Arc::new(DexNoteRepository::new(client.clone()));
        let reminder_repo = Arc::new(DexReminderRepository::new(client));

        let tools = RelationshipHistoryTools::new(contact_repo, note_repo, reminder_repo);

        let notes = vec![
            create_test_note("note1", "2024-01-01T10:00:00Z"),
            create_test_note("note2", "2024-01-15T10:00:00Z"),
            create_test_note("note3", "2024-02-01T10:00:00Z"),
        ];

        let filter = HistoryFilterParams {
            start_date: Some("2024-01-10T00:00:00Z".to_string()),
            end_date: Some("2024-01-20T00:00:00Z".to_string()),
            entry_types: None,
            limit: None,
        };

        let filtered = tools.filter_notes(notes, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "note2");
    }

    #[test]
    fn test_filter_notes_by_limit() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo = Arc::new(DexContactRepository::new(client.clone()));
        let note_repo = Arc::new(DexNoteRepository::new(client.clone()));
        let reminder_repo = Arc::new(DexReminderRepository::new(client));

        let tools = RelationshipHistoryTools::new(contact_repo, note_repo, reminder_repo);

        let notes = vec![
            create_test_note("note1", "2024-01-01T10:00:00Z"),
            create_test_note("note2", "2024-01-15T10:00:00Z"),
            create_test_note("note3", "2024-02-01T10:00:00Z"),
        ];

        let filter = HistoryFilterParams {
            start_date: None,
            end_date: None,
            entry_types: None,
            limit: Some(2),
        };

        let filtered = tools.filter_notes(notes, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_apply_filters_by_type() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo = Arc::new(DexContactRepository::new(client.clone()));
        let note_repo = Arc::new(DexNoteRepository::new(client.clone()));
        let reminder_repo = Arc::new(DexReminderRepository::new(client));

        let tools = RelationshipHistoryTools::new(contact_repo, note_repo, reminder_repo);

        let timeline = vec![
            TimelineEntry::Note(create_test_note("note1", "2024-01-01T10:00:00Z")),
            TimelineEntry::Reminder(create_test_reminder(
                "rem1",
                "2024-01-02T10:00:00Z",
                "2024-01-15T10:00:00Z",
            )),
        ];

        let filter = HistoryFilterParams {
            start_date: None,
            end_date: None,
            entry_types: Some(vec!["note".to_string()]),
            limit: None,
        };

        let filtered = tools.apply_filters(timeline, &filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].entry_type(), "note");
    }

    #[test]
    fn test_history_filter_params_default() {
        let filter = HistoryFilterParams::default();
        assert!(filter.start_date.is_none());
        assert!(filter.end_date.is_none());
        assert!(filter.entry_types.is_none());
        assert!(filter.limit.is_none());
    }
}

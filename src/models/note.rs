//! Note model representing a note associated with a contact.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

static HTML_TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<[^>]*>").expect("Failed to compile HTML tag regex"));

/// Helper struct for deserializing contact IDs from API
#[derive(Debug, Deserialize)]
struct ContactIdEntry {
    contact_id: String,
}

/// Custom deserializer for contact_id from contacts array
fn deserialize_contact_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let entries: Vec<ContactIdEntry> = Vec::deserialize(deserializer)?;
    Ok(entries
        .into_iter()
        .next()
        .map(|e| e.contact_id)
        .unwrap_or_default())
}

/// A note associated with a contact in Dex Personal CRM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Note {
    /// Unique identifier for the note
    #[serde(skip_serializing)]
    pub id: String,

    /// ID of the contact this note is associated with (API field: contacts)
    #[serde(
        rename = "contacts",
        deserialize_with = "deserialize_contact_id",
        skip_serializing
    )]
    pub contact_id: String,

    /// The note content (may contain HTML) (API field: note)
    #[serde(rename = "note")]
    pub content: String,

    /// When the note was created (ISO 8601 timestamp) (API field: event_time)
    #[serde(rename = "event_time", skip_serializing)]
    pub created_at: String,

    /// When the note was last updated (ISO 8601 timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    /// Tags associated with the note
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Source of the note (e.g., "manual", "email", "import")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Inner structure for timeline item contacts data
#[derive(Debug, Clone, Serialize)]
struct TimelineItemsContactsData {
    contact_id: String,
}

/// Wrapper for timeline items contacts
#[derive(Debug, Clone, Serialize)]
struct TimelineItemsContacts {
    data: Vec<TimelineItemsContactsData>,
}

/// Inner timeline event payload matching Dex API structure
#[derive(Debug, Clone, Serialize)]
struct TimelineEventPayload {
    /// The note content
    note: String,

    /// Event timestamp (ISO 8601 format)
    event_time: String,

    /// Meeting type - always "note" for notes
    meeting_type: String,

    /// Associated contacts
    timeline_items_contacts: TimelineItemsContacts,
}

/// Request payload for creating a new note.
/// This matches the Dex API structure: { "timeline_event": { ... } }
#[derive(Debug, Clone, Serialize)]
pub struct CreateNoteRequest {
    timeline_event: TimelineEventPayload,
}

/// Changes object for updating a note
#[derive(Debug, Clone, Serialize)]
struct NoteChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_time: Option<String>,
}

/// Request payload for updating a note.
/// This matches the Dex API structure: { "changes": { ... } }
#[derive(Debug, Clone, Serialize)]
pub struct UpdateNoteRequest {
    changes: NoteChanges,
}

impl From<&Note> for UpdateNoteRequest {
    fn from(note: &Note) -> Self {
        Self {
            changes: NoteChanges {
                note: if note.content.is_empty() {
                    None
                } else {
                    Some(note.content.clone())
                },
                event_time: None, // Don't update event_time
            },
        }
    }
}

impl From<&Note> for CreateNoteRequest {
    fn from(note: &Note) -> Self {
        // Use current time if note doesn't have a timestamp
        let event_time = if note.created_at.is_empty() {
            // Use current time in ISO 8601 format
            chrono::Utc::now().to_rfc3339()
        } else {
            note.created_at.clone()
        };

        Self {
            timeline_event: TimelineEventPayload {
                note: note.content.clone(),
                event_time,
                meeting_type: "note".to_string(),
                timeline_items_contacts: TimelineItemsContacts {
                    data: vec![TimelineItemsContactsData {
                        contact_id: note.contact_id.clone(),
                    }],
                },
            },
        }
    }
}

impl Note {
    /// Create a new note with required fields.
    pub fn new(id: String, contact_id: String, content: String, created_at: String) -> Self {
        Self {
            id,
            contact_id,
            content,
            created_at,
            updated_at: None,
            tags: Vec::new(),
            source: None,
        }
    }

    /// Strip HTML tags from the content to get plain text.
    pub fn plain_text(&self) -> String {
        // Simple HTML stripping - remove tags but keep content
        HTML_TAG_REGEX.replace_all(&self.content, "").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_new() {
        let note = Note::new(
            "note123".to_string(),
            "contact123".to_string(),
            "Met for coffee".to_string(),
            "2024-01-15T10:00:00Z".to_string(),
        );
        assert_eq!(note.id, "note123");
        assert_eq!(note.contact_id, "contact123");
        assert_eq!(note.content, "Met for coffee");
    }

    #[test]
    fn test_note_serialization() {
        let note = Note::new(
            "note123".to_string(),
            "contact123".to_string(),
            "Test content".to_string(),
            "2024-01-15T10:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&note).unwrap();
        // id is marked skip_serializing, so it won't be in JSON
        assert!(!json.contains("\"id\":\"note123\""));
        // contact_id is marked skip_serializing, so it won't be in JSON
        assert!(json.contains("\"note\":\"Test content\""));
        // event_time is marked skip_serializing, so it won't be in JSON
        assert!(!json.contains("\"event_time\":\"2024-01-15T10:00:00Z\""));
    }

    #[test]
    fn test_note_deserialization() {
        let json = r#"{
            "id": "note123",
            "contacts": [{"contact_id": "contact123"}],
            "note": "Test content",
            "event_time": "2024-01-15T10:00:00Z"
        }"#;
        let note: Note = serde_json::from_str(json).unwrap();
        assert_eq!(note.id, "note123");
        assert_eq!(note.contact_id, "contact123");
        assert_eq!(note.content, "Test content");
        assert_eq!(note.created_at, "2024-01-15T10:00:00Z");
    }

    #[test]
    fn test_note_plain_text() {
        let note = Note::new(
            "note123".to_string(),
            "contact123".to_string(),
            "<p>This is <strong>bold</strong> text</p>".to_string(),
            "2024-01-15T10:00:00Z".to_string(),
        );
        let plain = note.plain_text();
        assert!(!plain.contains("<p>"));
        assert!(!plain.contains("<strong>"));
        assert!(plain.contains("This is"));
        assert!(plain.contains("bold"));
    }

    #[test]
    fn test_note_create_serialization_bug() {
        // This test demonstrates the bug: when creating a note,
        // the contact_id is NOT serialized due to skip_serializing
        let note = Note::new(
            String::new(),
            "abb29721-d8c1-4a9f-a684-05c3ec7595ee".to_string(),
            "Reach out to Sayee".to_string(),
            String::new(),
        );
        let json = serde_json::to_string_pretty(&note).unwrap();
        println!("Serialized note for CREATE:\n{}", json);

        // This will PASS because contact_id has skip_serializing
        assert!(
            !json.contains("contact"),
            "BUG: contact_id is not serialized, so API will reject this!"
        );
    }

    #[test]
    fn test_create_note_request_serialization() {
        // This test verifies the fix: CreateNoteRequest properly serializes timeline_event structure
        let note = Note::new(
            String::new(),
            "abb29721-d8c1-4a9f-a684-05c3ec7595ee".to_string(),
            "Reach out to Sayee".to_string(),
            String::new(),
        );

        let request = CreateNoteRequest::from(&note);
        let json = serde_json::to_string_pretty(&request).unwrap();
        println!("CreateNoteRequest JSON:\n{}", json);

        // Verify timeline_event wrapper is present
        assert!(
            json.contains("timeline_event"),
            "timeline_event wrapper should be present"
        );
        assert!(
            json.contains("abb29721-d8c1-4a9f-a684-05c3ec7595ee"),
            "contact_id value should be in the request"
        );

        // Deserialize to check the structure
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(
            value["timeline_event"].is_object(),
            "timeline_event should be an object"
        );

        let timeline_event = &value["timeline_event"];
        assert_eq!(
            timeline_event["note"].as_str().unwrap(),
            "Reach out to Sayee"
        );
        assert_eq!(timeline_event["meeting_type"].as_str().unwrap(), "note");
        assert!(
            timeline_event["event_time"].is_string(),
            "event_time should be a string"
        );

        // Check timeline_items_contacts structure
        let contacts = &timeline_event["timeline_items_contacts"]["data"];
        assert!(
            contacts.is_array(),
            "timeline_items_contacts.data should be an array"
        );
        assert_eq!(
            contacts.as_array().unwrap().len(),
            1,
            "contacts array should have exactly 1 element"
        );
        assert_eq!(
            contacts[0]["contact_id"].as_str().unwrap(),
            "abb29721-d8c1-4a9f-a684-05c3ec7595ee",
            "contact_id should match"
        );
    }
}

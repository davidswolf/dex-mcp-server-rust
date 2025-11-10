//! Reminder model representing a reminder associated with a contact.

use serde::{Deserialize, Deserializer, Serialize};

/// Helper function to check if a boolean is false (for serde skip_serializing_if)
fn is_false(b: &bool) -> bool {
    !*b
}

/// Helper struct for deserializing contact IDs from API
#[derive(Debug, Deserialize)]
struct ContactIdEntry {
    contact_id: String,
}

/// Custom deserializer for contact_id from contact_ids array
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

/// A reminder associated with a contact in Dex Personal CRM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Reminder {
    /// Unique identifier for the reminder
    #[serde(skip_serializing)]
    pub id: String,

    /// ID of the contact this reminder is associated with (API field: contact_ids)
    #[serde(
        rename = "contact_ids",
        deserialize_with = "deserialize_contact_id",
        skip_serializing
    )]
    pub contact_id: String,

    /// The reminder text/description (API field: body)
    #[serde(rename = "body")]
    pub text: String,

    /// When the reminder is due (API field: due_at_date)
    #[serde(rename = "due_at_date")]
    pub due_date: String,

    /// Whether the reminder has been completed (API field: is_complete)
    #[serde(rename = "is_complete", default, skip_serializing_if = "is_false")]
    pub completed: bool,

    /// When the reminder was completed (ISO 8601 timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,

    /// When the reminder was created (ISO 8601 timestamp) - not provided by API
    #[serde(default = "default_timestamp", skip_serializing)]
    pub created_at: String,

    /// When the reminder was last updated (ISO 8601 timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    /// Tags associated with the reminder
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Priority level (e.g., "high", "medium", "low")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
}

/// Inner structure for reminder contacts data
#[derive(Debug, Clone, Serialize)]
struct RemindersContactsData {
    contact_id: String,
}

/// Wrapper for reminders contacts
#[derive(Debug, Clone, Serialize)]
struct RemindersContacts {
    data: Vec<RemindersContactsData>,
}

/// Inner reminder payload matching Dex API structure
#[derive(Debug, Clone, Serialize)]
struct ReminderPayload {
    /// Title of the reminder (using the first line of text or the full text if short)
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    /// The reminder text/description
    text: String,

    /// Whether the reminder is completed
    #[serde(skip_serializing_if = "Option::is_none")]
    is_complete: Option<bool>,

    /// When the reminder is due (YYYY-MM-DD format)
    due_at_date: String,

    /// Associated contacts
    reminders_contacts: RemindersContacts,

    /// Priority level
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
}

/// Request payload for creating a new reminder.
/// This matches the Dex API structure: { "reminder": { ... } }
#[derive(Debug, Clone, Serialize)]
pub struct CreateReminderRequest {
    reminder: ReminderPayload,
}

/// Changes object for updating a reminder
#[derive(Debug, Clone, Serialize)]
struct ReminderChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_complete: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_at_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_at_time: Option<String>,
}

/// Request payload for updating a reminder.
/// This matches the Dex API structure: { "changes": { ... } }
#[derive(Debug, Clone, Serialize)]
pub struct UpdateReminderRequest {
    changes: ReminderChanges,
}

impl From<&Reminder> for UpdateReminderRequest {
    fn from(reminder: &Reminder) -> Self {
        Self {
            changes: ReminderChanges {
                text: if reminder.text.is_empty() {
                    None
                } else {
                    Some(reminder.text.clone())
                },
                is_complete: Some(reminder.completed),
                due_at_date: if reminder.due_date.is_empty() {
                    None
                } else {
                    Some(reminder.due_date.clone())
                },
                due_at_time: None,
            },
        }
    }
}

impl From<&Reminder> for CreateReminderRequest {
    fn from(reminder: &Reminder) -> Self {
        Self {
            reminder: ReminderPayload {
                title: None, // API doesn't support title field during creation
                text: reminder.text.clone(),
                is_complete: if reminder.completed { Some(true) } else { None },
                due_at_date: reminder.due_date.clone(),
                reminders_contacts: RemindersContacts {
                    data: vec![RemindersContactsData {
                        contact_id: reminder.contact_id.clone(),
                    }],
                },
                priority: reminder.priority.clone(),
            },
        }
    }
}

/// Default timestamp for created_at when API doesn't provide it
fn default_timestamp() -> String {
    String::new()
}

impl Reminder {
    /// Create a new reminder with required fields.
    pub fn new(
        id: String,
        contact_id: String,
        text: String,
        due_date: String,
        created_at: String,
    ) -> Self {
        Self {
            id,
            contact_id,
            text,
            due_date,
            completed: false,
            completed_at: None,
            created_at,
            updated_at: None,
            tags: Vec::new(),
            priority: None,
        }
    }

    /// Check if the reminder is overdue based on the current date.
    pub fn is_overdue(&self, current_date: &str) -> bool {
        !self.completed && self.due_date.as_str() < current_date
    }

    /// Mark the reminder as completed with the given timestamp.
    pub fn mark_completed(&mut self, completed_at: String) {
        self.completed = true;
        self.completed_at = Some(completed_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reminder_new() {
        let reminder = Reminder::new(
            "reminder123".to_string(),
            "contact123".to_string(),
            "Follow up on proposal".to_string(),
            "2024-02-01T10:00:00Z".to_string(),
            "2024-01-15T10:00:00Z".to_string(),
        );
        assert_eq!(reminder.id, "reminder123");
        assert_eq!(reminder.contact_id, "contact123");
        assert_eq!(reminder.text, "Follow up on proposal");
        assert!(!reminder.completed);
    }

    #[test]
    fn test_reminder_is_overdue() {
        let reminder = Reminder::new(
            "reminder123".to_string(),
            "contact123".to_string(),
            "Old reminder".to_string(),
            "2024-01-01T10:00:00Z".to_string(),
            "2023-12-15T10:00:00Z".to_string(),
        );

        // Check against a date after the due date
        assert!(reminder.is_overdue("2024-01-15T10:00:00Z"));

        // Check against a date before the due date
        assert!(!reminder.is_overdue("2023-12-20T10:00:00Z"));
    }

    #[test]
    fn test_reminder_mark_completed() {
        let mut reminder = Reminder::new(
            "reminder123".to_string(),
            "contact123".to_string(),
            "Task".to_string(),
            "2024-02-01T10:00:00Z".to_string(),
            "2024-01-15T10:00:00Z".to_string(),
        );

        assert!(!reminder.completed);
        assert!(reminder.completed_at.is_none());

        reminder.mark_completed("2024-01-20T15:30:00Z".to_string());

        assert!(reminder.completed);
        assert_eq!(
            reminder.completed_at,
            Some("2024-01-20T15:30:00Z".to_string())
        );

        // Completed reminders are not overdue
        assert!(!reminder.is_overdue("2024-02-15T10:00:00Z"));
    }

    #[test]
    fn test_reminder_serialization() {
        let reminder = Reminder::new(
            "reminder123".to_string(),
            "contact123".to_string(),
            "Test reminder".to_string(),
            "2024-02-01".to_string(),
            "2024-01-15T10:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&reminder).unwrap();
        // id is marked skip_serializing, so it won't be in JSON
        assert!(!json.contains("\"id\":\"reminder123\""));
        // contact_id is marked skip_serializing, so it won't be in JSON
        assert!(json.contains("\"body\":\"Test reminder\""));
        assert!(json.contains("\"due_at_date\":\"2024-02-01\""));
        // is_complete field should NOT be present when false
        assert!(!json.contains("\"is_complete\""));

        // When completed is true, it should be serialized
        let mut completed_reminder = reminder.clone();
        completed_reminder.completed = true;
        let json2 = serde_json::to_string(&completed_reminder).unwrap();
        assert!(json2.contains("\"is_complete\":true"));
    }

    #[test]
    fn test_reminder_deserialization() {
        let json = r#"{
            "id": "reminder123",
            "contact_ids": [{"contact_id": "contact123"}],
            "body": "Test reminder",
            "due_at_date": "2024-02-01",
            "is_complete": false
        }"#;
        let reminder: Reminder = serde_json::from_str(json).unwrap();
        assert_eq!(reminder.id, "reminder123");
        assert_eq!(reminder.contact_id, "contact123");
        assert_eq!(reminder.text, "Test reminder");
        assert_eq!(reminder.due_date, "2024-02-01");
        assert!(!reminder.completed);
    }

    #[test]
    fn test_reminder_create_serialization_bug() {
        // This test demonstrates the bug: when creating a reminder,
        // the contact_id is NOT serialized due to skip_serializing
        let reminder = Reminder::new(
            String::new(),
            "abb29721-d8c1-4a9f-a684-05c3ec7595ee".to_string(),
            "Reach out to Sayee again".to_string(),
            "2025-10-22".to_string(),
            String::new(),
        );
        let json = serde_json::to_string_pretty(&reminder).unwrap();
        println!("Serialized reminder for CREATE:\n{}", json);

        // This will FAIL because contact_id has skip_serializing
        assert!(!json.contains("contact"),
            "BUG: contact_id is not serialized, so API will reject this!");
    }

    #[test]
    fn test_create_reminder_request_serialization() {
        // This test verifies the fix: CreateReminderRequest properly serializes reminder structure
        let reminder = Reminder::new(
            String::new(),
            "abb29721-d8c1-4a9f-a684-05c3ec7595ee".to_string(),
            "Reach out to Sayee again".to_string(),
            "2025-10-22".to_string(),
            String::new(),
        );

        let request = CreateReminderRequest::from(&reminder);
        let json = serde_json::to_string_pretty(&request).unwrap();
        println!("CreateReminderRequest JSON:\n{}", json);

        // Verify reminder wrapper is present
        assert!(json.contains("reminder"), "reminder wrapper should be present");
        assert!(json.contains("abb29721-d8c1-4a9f-a684-05c3ec7595ee"),
            "contact_id value should be in the request");

        // Deserialize to check the structure
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value["reminder"].is_object(), "reminder should be an object");

        let reminder_obj = &value["reminder"];
        assert_eq!(reminder_obj["text"].as_str().unwrap(), "Reach out to Sayee again");
        // title is None (API doesn't support title field during creation), so it won't be in JSON
        assert!(reminder_obj.get("title").is_none(), "title should not be present when None");
        assert_eq!(reminder_obj["due_at_date"].as_str().unwrap(), "2025-10-22");

        // Check reminders_contacts structure
        let contacts = &reminder_obj["reminders_contacts"]["data"];
        assert!(contacts.is_array(), "reminders_contacts.data should be an array");
        assert_eq!(contacts.as_array().unwrap().len(), 1,
            "contacts array should have exactly 1 element");
        assert_eq!(contacts[0]["contact_id"].as_str().unwrap(),
            "abb29721-d8c1-4a9f-a684-05c3ec7595ee",
            "contact_id should match");
    }
}

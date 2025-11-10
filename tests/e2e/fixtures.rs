//! Test fixtures and sample data for E2E tests.
//!
//! This module provides reusable test data and fixtures for creating
//! contacts, notes, and reminders in tests.

use chrono::Utc;
use dex_mcp_server::{Contact, Note, Reminder};

/// Create a sample contact for testing with first name, last name, and email.
///
/// Returns a Contact with minimal required fields set.
/// The ID will be empty and should be set after creation.
pub fn sample_contact(first_name: &str, last_name: &str, email: &str) -> Contact {
    Contact {
        first_name: Some(first_name.to_string()),
        last_name: Some(last_name.to_string()),
        emails: vec![email.to_string()],
        ..Default::default()
    }
}

/// Create a sample contact with just a name (legacy function).
#[allow(dead_code)]
pub fn sample_contact_name_only(name: &str) -> Contact {
    Contact::new("".to_string(), name.to_string())
}

/// Create a sample contact with email (legacy function).
#[allow(dead_code)]
pub fn sample_contact_with_email(name: &str, email: &str) -> Contact {
    let mut contact = Contact::new("".to_string(), name.to_string());
    contact.email = Some(email.to_string());
    contact
}

/// Create a sample note for testing.
///
/// The note ID will be empty and should be set after creation.
pub fn sample_note(contact_id: &str, content: &str) -> Note {
    let now = Utc::now().to_rfc3339();
    Note::new("".to_string(), contact_id.to_string(), content.to_string(), now)
}

/// Create a sample note with a specific timestamp.
#[allow(dead_code)]
pub fn sample_note_with_timestamp(
    contact_id: &str,
    content: &str,
    created_at: &str,
) -> Note {
    Note::new(
        "".to_string(),
        contact_id.to_string(),
        content.to_string(),
        created_at.to_string(),
    )
}

/// Create a sample reminder for testing.
///
/// The reminder ID will be empty and should be set after creation.
/// Due date is set to 7 days from now by default.
pub fn sample_reminder(contact_id: &str, text: &str) -> Reminder {
    let now = Utc::now();
    let due_date = (now + chrono::Duration::days(7)).to_rfc3339();
    let created_at = now.to_rfc3339();

    Reminder::new(
        "".to_string(),
        contact_id.to_string(),
        text.to_string(),
        due_date,
        created_at,
    )
}

/// Create a sample reminder with a specific due date.
#[allow(dead_code)]
pub fn sample_reminder_with_due_date(
    contact_id: &str,
    text: &str,
    due_date: &str,
) -> Reminder {
    let now = Utc::now().to_rfc3339();

    Reminder::new(
        "".to_string(),
        contact_id.to_string(),
        text.to_string(),
        due_date.to_string(),
        now,
    )
}

/// Generate a unique test identifier based on current timestamp.
///
/// Useful for creating unique test data that won't conflict.
pub fn generate_unique_id() -> String {
    format!("test_{}", Utc::now().timestamp_millis())
}

/// Generate a test note content with timestamp to ensure uniqueness.
#[allow(dead_code)]
pub fn generate_test_note_content(prefix: &str) -> String {
    format!("{} - {}", prefix, Utc::now().to_rfc3339())
}

/// Generate a test reminder text with timestamp to ensure uniqueness.
#[allow(dead_code)]
pub fn generate_test_reminder_text(prefix: &str) -> String {
    format!("{} - {}", prefix, Utc::now().to_rfc3339())
}

/// Sample contact names for testing name variations.
#[allow(dead_code)]
pub fn name_variations() -> Vec<(&'static str, &'static str)> {
    vec![
        ("John Doe", "john doe"),        // Case insensitive
        ("Jane Smith", "jane"),          // Partial match
        ("Robert Johnson", "Bob Johnson"), // Name variation
        ("Michael Brown", "Mike Brown"), // Nickname
    ]
}

/// Sample email addresses for testing email search.
#[allow(dead_code)]
pub fn sample_emails() -> Vec<&'static str> {
    vec![
        "test@example.com",
        "john.doe@company.com",
        "jane_smith123@email.co.uk",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_contact() {
        let contact = sample_contact("Test", "User", "test@example.com");
        assert_eq!(contact.first_name, Some("Test".to_string()));
        assert_eq!(contact.last_name, Some("User".to_string()));
        assert_eq!(contact.emails, vec!["test@example.com".to_string()]);
    }

    #[test]
    fn test_sample_contact_with_email() {
        let contact = sample_contact_with_email("Test User", "test@example.com");
        assert_eq!(contact.name, "Test User");
        assert_eq!(contact.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_sample_note() {
        let note = sample_note("contact123", "Test note");
        assert_eq!(note.contact_id, "contact123");
        assert_eq!(note.content, "Test note");
        assert!(!note.created_at.is_empty());
    }

    #[test]
    fn test_sample_reminder() {
        let reminder = sample_reminder("contact123", "Follow up");
        assert_eq!(reminder.contact_id, "contact123");
        assert_eq!(reminder.text, "Follow up");
        assert!(!reminder.due_date.is_empty());
        assert!(!reminder.created_at.is_empty());
    }

    #[test]
    fn test_generate_unique_id() {
        let id1 = generate_unique_id();
        let _id2 = generate_unique_id();
        // IDs should be different (though timing could make them same in rare cases)
        assert!(id1.starts_with("test_"));
    }
}

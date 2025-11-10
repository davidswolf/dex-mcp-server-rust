//! End-to-end test utilities and shared configuration.
//!
//! This module provides common setup, helpers, and assertions for E2E tests
//! that interact with the live Dex API.

use dex_mcp_server::{Contact, DexClient, Note, Reminder};
use std::env;

pub mod fixtures;

/// Test configuration loaded from environment variables.
pub struct TestConfig {
    pub api_key: String,
    pub base_url: String,
}

impl TestConfig {
    /// Load configuration from .env file.
    ///
    /// # Panics
    /// Panics if DEX_API_KEY is not set in the environment.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            api_key: env::var("DEX_API_KEY")
                .expect("DEX_API_KEY must be set in .env file for E2E tests"),
            base_url: env::var("DEX_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.getdex.com/api/rest".to_string()),
        }
    }
}

/// Create a DexClient configured for testing.
pub fn setup_test_client() -> DexClient {
    let config = TestConfig::from_env();
    DexClient::with_base_url(config.base_url, config.api_key)
}

/// Assert that a contact has valid required fields.
#[allow(dead_code)]
pub fn assert_contact_valid(contact: &Contact) {
    assert!(!contact.id.is_empty(), "Contact ID should not be empty");
    assert!(!contact.name.is_empty(), "Contact name should not be empty");
}

/// Assert that a note has valid required fields.
#[allow(dead_code)]
pub fn assert_note_valid(note: &Note) {
    assert!(!note.id.is_empty(), "Note ID should not be empty");
    assert!(!note.contact_id.is_empty(), "Note contact_id should not be empty");
    assert!(!note.content.is_empty(), "Note content should not be empty");
    assert!(!note.created_at.is_empty(), "Note created_at should not be empty");
}

/// Assert that a reminder has valid required fields.
#[allow(dead_code)]
pub fn assert_reminder_valid(reminder: &Reminder) {
    assert!(!reminder.id.is_empty(), "Reminder ID should not be empty");
    assert!(!reminder.contact_id.is_empty(), "Reminder contact_id should not be empty");
    assert!(!reminder.text.is_empty(), "Reminder text should not be empty");
    assert!(!reminder.due_date.is_empty(), "Reminder due_date should not be empty");
    // Note: created_at is not provided by the Dex API for reminders
}

/// Get a list of known test contact names from environment or defaults.
///
/// These are contacts that should exist in your Dex database for testing.
/// Configure them in .env with TEST_CONTACT_NAME_1, TEST_CONTACT_NAME_2, etc.
#[allow(dead_code)]
pub fn get_known_test_contacts() -> Vec<String> {
    let mut contacts = Vec::new();

    // Try to load from environment
    if let Ok(name) = env::var("TEST_CONTACT_NAME_1") {
        contacts.push(name);
    }
    if let Ok(name) = env::var("TEST_CONTACT_NAME_2") {
        contacts.push(name);
    }
    if let Ok(name) = env::var("TEST_CONTACT_NAME_3") {
        contacts.push(name);
    }

    // If none set, use defaults (these should be updated to match your actual test data)
    if contacts.is_empty() {
        contacts.push("Greg Hoy".to_string());
        contacts.push("Peter Wong".to_string());
    }

    contacts
}

/// Get a known test contact email from environment.
#[allow(dead_code)]
pub fn get_test_contact_email() -> Option<String> {
    env::var("TEST_CONTACT_EMAIL").ok()
}

/// Get a known test contact ID from environment.
#[allow(dead_code)]
pub fn get_test_contact_id() -> Option<String> {
    env::var("TEST_CONTACT_ID").ok()
}

/// Sleep for a short duration to avoid rate limits.
///
/// Use this between API calls in tests that make multiple requests.
#[allow(dead_code)]
pub async fn rate_limit_delay() {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

/// Retry a function with exponential backoff.
///
/// This is useful for handling transient network errors or rate limits.
#[allow(dead_code)]
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut f: F,
    max_retries: u32,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                let delay = tokio::time::Duration::from_millis(100 * 2_u64.pow(retries));
                eprintln!("Retry {}/{}: {}", retries, max_retries, e);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loads_from_env() {
        // This test will panic if .env is not configured properly
        let config = TestConfig::from_env();
        assert!(!config.api_key.is_empty());
        assert!(!config.base_url.is_empty());
    }

    #[test]
    fn test_setup_client() {
        let _client = setup_test_client();
        // Just verify we can create a client
        // Actual API calls tested in integration tests
    }
}

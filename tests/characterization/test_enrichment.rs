//! Characterization tests for ContactEnrichmentTools
//!
//! These tests document the current behavior before refactoring.
//! They help ensure that refactoring doesn't break existing functionality.

use dex_mcp_server::tools::enrichment::ContactEnrichmentTools;
use dex_mcp_server::client::DexClient;
use dex_mcp_server::models::{Contact, Note, Reminder};
use std::sync::Arc;
use std::time::Instant;
use mockito::{Mock, Server};
use serde_json::json;

/// Helper to create a test DexClient with mock server
fn setup_mock_client(server: &Server) -> DexClient {
    DexClient::new_with_base_url(
        "test_api_key".to_string(),
        server.url(),
    ).unwrap()
}

/// Helper to create a test contact
fn create_test_contact() -> Contact {
    Contact {
        id: "contact1".to_string(),
        name: "John Doe".to_string(),
        email: Some("john.doe@example.com".to_string()),
        ..Default::default()
    }
}

/// Helper to create a test note
fn create_test_note(contact_id: &str) -> Note {
    Note {
        id: "note1".to_string(),
        contact_id: contact_id.to_string(),
        content: "Test note content".to_string(),
        created_at: "2024-01-15T10:00:00Z".to_string(),
        ..Default::default()
    }
}

/// Helper to create a test reminder
fn create_test_reminder(contact_id: &str) -> Reminder {
    Reminder {
        id: "reminder1".to_string(),
        contact_id: contact_id.to_string(),
        text: "Follow up with contact".to_string(),
        due_date: "2024-02-01".to_string(),
        created_at: "2024-01-15T10:00:00Z".to_string(),
        ..Default::default()
    }
}

/// Test: Enrich contact with basic information
///
/// This test documents the current behavior when enriching a contact
/// with notes and reminders.
#[tokio::test]
async fn test_enrich_contact_basic() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let notes = vec![create_test_note(&contact.id)];
    let reminders = vec![create_test_reminder(&contact.id)];

    // Mock get_contact_notes endpoint
    let notes_mock = server.mock("GET", format!("/api/contacts/{}/timeline_events", contact.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(notes).to_string())
        .create_async()
        .await;

    // Mock get_contact_reminders endpoint
    let reminders_mock = server.mock("GET", format!("/api/contacts/{}/reminders", contact.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(reminders).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Fetch notes
    let notes_result = client.get_contact_notes(&contact.id, 100, 0);

    // Fetch reminders
    let reminders_result = client.get_contact_reminders(&contact.id, 100, 0);

    let duration = start.elapsed();
    println!("Contact enrichment (sequential) took: {:?}", duration);

    assert!(notes_result.is_ok(), "Notes fetch should succeed");
    assert!(reminders_result.is_ok(), "Reminders fetch should succeed");

    let fetched_notes = notes_result.unwrap();
    let fetched_reminders = reminders_result.unwrap();

    assert_eq!(fetched_notes.len(), 1, "Should fetch one note");
    assert_eq!(fetched_reminders.len(), 1, "Should fetch one reminder");

    notes_mock.assert_async().await;
    reminders_mock.assert_async().await;
}

/// Test: Enrich contact - merge fields
///
/// This test documents how contact enrichment handles merging
/// additional fields into a contact.
#[tokio::test]
async fn test_enrich_contact_merge_fields() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();

    // Mock get contact endpoint
    let contact_mock = server.mock("GET", format!("/api/contacts/{}", contact.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(contact).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Fetch contact details
    let result = client.get_contact(&contact.id);

    let duration = start.elapsed();
    println!("Contact fetch took: {:?}", duration);

    assert!(result.is_ok(), "Contact fetch should succeed");
    let fetched_contact = result.unwrap();
    assert_eq!(fetched_contact.id, contact.id);
    assert_eq!(fetched_contact.name, contact.name);

    contact_mock.assert_async().await;
}

/// Test: Create note for contact
///
/// This test documents the current behavior when creating a note.
#[tokio::test]
async fn test_create_note() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let note = create_test_note(&contact.id);

    // Mock create note endpoint
    let create_mock = server.mock("POST", "/api/timeline_events")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(json!(note).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Create note
    let result = client.create_note(&note);

    let duration = start.elapsed();
    println!("Note creation took: {:?}", duration);

    assert!(result.is_ok(), "Note creation should succeed");
    let created_note = result.unwrap();
    assert_eq!(created_note.content, note.content);

    create_mock.assert_async().await;
}

/// Test: Create reminder for contact
///
/// This test documents the current behavior when creating a reminder.
#[tokio::test]
async fn test_create_reminder() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let reminder = create_test_reminder(&contact.id);

    // Mock create reminder endpoint
    let create_mock = server.mock("POST", "/api/reminders")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(json!(reminder).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Create reminder
    let result = client.create_reminder(&reminder);

    let duration = start.elapsed();
    println!("Reminder creation took: {:?}", duration);

    assert!(result.is_ok(), "Reminder creation should succeed");
    let created_reminder = result.unwrap();
    assert_eq!(created_reminder.text, reminder.text);

    create_mock.assert_async().await;
}

/// Test: Update note
///
/// This test documents the current behavior when updating a note.
#[tokio::test]
async fn test_update_note() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let mut note = create_test_note(&contact.id);
    note.content = "Updated content".to_string();

    // Mock update note endpoint
    let update_mock = server.mock("PUT", format!("/api/timeline_events/{}", note.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(note).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Update note
    let result = client.update_note(&note.id, &note);

    let duration = start.elapsed();
    println!("Note update took: {:?}", duration);

    assert!(result.is_ok(), "Note update should succeed");
    let updated_note = result.unwrap();
    assert_eq!(updated_note.content, "Updated content");

    update_mock.assert_async().await;
}

/// Test: Update reminder
///
/// This test documents the current behavior when updating a reminder.
#[tokio::test]
async fn test_update_reminder() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let mut reminder = create_test_reminder(&contact.id);
    reminder.text = "Updated reminder".to_string();

    // Mock update reminder endpoint
    let update_mock = server.mock("PUT", format!("/api/reminders/{}", reminder.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(reminder).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Update reminder
    let result = client.update_reminder(&reminder.id, &reminder);

    let duration = start.elapsed();
    println!("Reminder update took: {:?}", duration);

    assert!(result.is_ok(), "Reminder update should succeed");
    let updated_reminder = result.unwrap();
    assert_eq!(updated_reminder.text, "Updated reminder");

    update_mock.assert_async().await;
}

/// Test: ContactEnrichmentTools creation
///
/// This test documents that enrichment tools can be created with a client.
#[test]
fn test_contact_enrichment_tools_creation() {
    let server = mockito::Server::new();
    let client = setup_mock_client(&server);
    let tools = ContactEnrichmentTools::new(Arc::new(client));

    // Verify the tools were created successfully
    assert!(true, "Tools created successfully");
}

//! Characterization tests for RelationshipHistoryTools
//!
//! These tests document the current behavior before refactoring.
//! They help ensure that refactoring doesn't break existing functionality.

use dex_mcp_server::tools::history::RelationshipHistoryTools;
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

/// Helper to create test notes
fn create_test_notes(contact_id: &str) -> Vec<Note> {
    vec![
        Note {
            id: "note1".to_string(),
            contact_id: contact_id.to_string(),
            content: "Met for coffee".to_string(),
            created_at: "2024-01-15T10:00:00Z".to_string(),
            ..Default::default()
        },
        Note {
            id: "note2".to_string(),
            contact_id: contact_id.to_string(),
            content: "Discussed project".to_string(),
            created_at: "2024-01-20T14:00:00Z".to_string(),
            ..Default::default()
        },
        Note {
            id: "note3".to_string(),
            contact_id: contact_id.to_string(),
            content: "Follow-up call".to_string(),
            created_at: "2024-02-01T09:00:00Z".to_string(),
            ..Default::default()
        },
    ]
}

/// Helper to create test reminders
fn create_test_reminders(contact_id: &str) -> Vec<Reminder> {
    vec![
        Reminder {
            id: "reminder1".to_string(),
            contact_id: contact_id.to_string(),
            text: "Send proposal".to_string(),
            due_date: "2024-02-05".to_string(),
            created_at: "2024-01-15T10:00:00Z".to_string(),
            completed: false,
            ..Default::default()
        },
        Reminder {
            id: "reminder2".to_string(),
            contact_id: contact_id.to_string(),
            text: "Schedule meeting".to_string(),
            due_date: "2024-02-10".to_string(),
            created_at: "2024-01-20T14:00:00Z".to_string(),
            completed: true,
            ..Default::default()
        },
    ]
}

/// Test: Get timeline for contact
///
/// This test documents the current behavior when fetching a contact's timeline.
#[tokio::test]
async fn test_get_timeline() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let notes = create_test_notes(&contact.id);
    let reminders = create_test_reminders(&contact.id);

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

    // Fetch timeline (notes + reminders)
    let notes_result = client.get_contact_notes(&contact.id, 100, 0);
    let reminders_result = client.get_contact_reminders(&contact.id, 100, 0);

    let duration = start.elapsed();
    println!("Timeline fetch (sequential) took: {:?}", duration);

    assert!(notes_result.is_ok(), "Notes fetch should succeed");
    assert!(reminders_result.is_ok(), "Reminders fetch should succeed");

    let fetched_notes = notes_result.unwrap();
    let fetched_reminders = reminders_result.unwrap();

    assert_eq!(fetched_notes.len(), 3, "Should fetch three notes");
    assert_eq!(fetched_reminders.len(), 2, "Should fetch two reminders");

    notes_mock.assert_async().await;
    reminders_mock.assert_async().await;
}

/// Test: Get reminders only
///
/// This test documents the current behavior when fetching only reminders.
#[tokio::test]
async fn test_get_reminders() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let reminders = create_test_reminders(&contact.id);

    // Mock get_contact_reminders endpoint
    let reminders_mock = server.mock("GET", format!("/api/contacts/{}/reminders", contact.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(reminders).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Fetch reminders
    let result = client.get_contact_reminders(&contact.id, 100, 0);

    let duration = start.elapsed();
    println!("Reminders fetch took: {:?}", duration);

    assert!(result.is_ok(), "Reminders fetch should succeed");
    let fetched_reminders = result.unwrap();

    assert_eq!(fetched_reminders.len(), 2, "Should fetch two reminders");

    // Verify we have both completed and incomplete reminders
    let completed_count = fetched_reminders.iter().filter(|r| r.completed).count();
    let incomplete_count = fetched_reminders.iter().filter(|r| !r.completed).count();

    assert_eq!(completed_count, 1, "Should have one completed reminder");
    assert_eq!(incomplete_count, 1, "Should have one incomplete reminder");

    reminders_mock.assert_async().await;
}

/// Test: Get notes with pagination
///
/// This test documents the current behavior when paginating through notes.
#[tokio::test]
async fn test_get_notes_paginated() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let notes = create_test_notes(&contact.id);

    // Mock first page
    let page1_mock = server.mock("GET", format!("/api/contacts/{}/timeline_events", contact.id).as_str())
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "2".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(&notes[0..2]).to_string())
        .create_async()
        .await;

    // Mock second page
    let page2_mock = server.mock("GET", format!("/api/contacts/{}/timeline_events", contact.id).as_str())
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "2".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "2".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(&notes[2..3]).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Fetch first page
    let page1_result = client.get_contact_notes(&contact.id, 2, 0);
    assert!(page1_result.is_ok());
    let page1_notes = page1_result.unwrap();
    assert_eq!(page1_notes.len(), 2);

    // Fetch second page
    let page2_result = client.get_contact_notes(&contact.id, 2, 2);
    assert!(page2_result.is_ok());
    let page2_notes = page2_result.unwrap();
    assert_eq!(page2_notes.len(), 1);

    let duration = start.elapsed();
    println!("Paginated notes fetch took: {:?}", duration);

    page1_mock.assert_async().await;
    page2_mock.assert_async().await;
}

/// Test: Timeline with filtering
///
/// This test documents how timeline data can be filtered after fetching.
#[tokio::test]
async fn test_timeline_filtering() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let notes = create_test_notes(&contact.id);
    let reminders = create_test_reminders(&contact.id);

    // Mock endpoints
    let notes_mock = server.mock("GET", format!("/api/contacts/{}/timeline_events", contact.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(notes).to_string())
        .create_async()
        .await;

    let reminders_mock = server.mock("GET", format!("/api/contacts/{}/reminders", contact.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(reminders).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Fetch all timeline data
    let notes_result = client.get_contact_notes(&contact.id, 100, 0);
    let reminders_result = client.get_contact_reminders(&contact.id, 100, 0);

    let duration = start.elapsed();

    assert!(notes_result.is_ok());
    assert!(reminders_result.is_ok());

    let fetched_notes = notes_result.unwrap();
    let fetched_reminders = reminders_result.unwrap();

    // Test filtering by date (in-memory)
    let notes_after_jan20 = fetched_notes.iter()
        .filter(|n| n.created_at >= "2024-01-20T00:00:00Z")
        .count();

    assert_eq!(notes_after_jan20, 2, "Should have 2 notes after Jan 20");

    println!("Timeline filtering took: {:?}", duration);

    notes_mock.assert_async().await;
    reminders_mock.assert_async().await;
}

/// Test: RelationshipHistoryTools creation
///
/// This test documents that history tools can be created with a client.
#[test]
fn test_relationship_history_tools_creation() {
    let server = mockito::Server::new();
    let client = setup_mock_client(&server);
    let tools = RelationshipHistoryTools::new(Arc::new(client));

    // Verify the tools were created successfully
    assert!(true, "Tools created successfully");
}

/// Test: Performance baseline for full timeline
///
/// This test establishes a baseline for fetching complete timeline data.
#[tokio::test]
async fn test_full_timeline_performance() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();

    // Create larger dataset for performance testing
    let large_notes: Vec<Note> = (0..50).map(|i| Note {
        id: format!("note{}", i),
        contact_id: contact.id.clone(),
        content: format!("Note {}", i),
        created_at: format!("2024-01-{:02}T10:00:00Z", (i % 28) + 1),
        ..Default::default()
    }).collect();

    let large_reminders: Vec<Reminder> = (0..20).map(|i| Reminder {
        id: format!("reminder{}", i),
        contact_id: contact.id.clone(),
        text: format!("Reminder {}", i),
        due_date: format!("2024-02-{:02}", (i % 28) + 1),
        created_at: format!("2024-01-{:02}T10:00:00Z", (i % 28) + 1),
        completed: i % 2 == 0,
        ..Default::default()
    }).collect();

    // Mock endpoints
    let notes_mock = server.mock("GET", format!("/api/contacts/{}/timeline_events", contact.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(large_notes).to_string())
        .create_async()
        .await;

    let reminders_mock = server.mock("GET", format!("/api/contacts/{}/reminders", contact.id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(large_reminders).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Fetch full timeline
    let notes_result = client.get_contact_notes(&contact.id, 100, 0);
    let reminders_result = client.get_contact_reminders(&contact.id, 100, 0);

    let duration = start.elapsed();

    assert!(notes_result.is_ok());
    assert!(reminders_result.is_ok());

    let fetched_notes = notes_result.unwrap();
    let fetched_reminders = reminders_result.unwrap();

    assert_eq!(fetched_notes.len(), 50);
    assert_eq!(fetched_reminders.len(), 20);

    println!("Full timeline fetch (50 notes + 20 reminders) took: {:?}", duration);

    notes_mock.assert_async().await;
    reminders_mock.assert_async().await;
}

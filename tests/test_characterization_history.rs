//! Characterization tests for RelationshipHistoryTools

use dex_mcp_server::client::{AsyncDexClient, AsyncDexClientImpl, DexClient};
use dex_mcp_server::models::{Contact, Note, Reminder};
use dex_mcp_server::repositories::{
    ContactRepository, DexContactRepository, DexNoteRepository, DexReminderRepository,
    NoteRepository, ReminderRepository,
};
use dex_mcp_server::tools::history::RelationshipHistoryTools;
use mockito::Server;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;

fn setup_mock_client(server: &Server) -> DexClient {
    DexClient::with_base_url(server.url(), "test_api_key".to_string())
}

fn create_test_contact() -> Contact {
    Contact {
        id: "contact1".to_string(),
        name: "John Doe".to_string(),
        email: Some("john.doe@example.com".to_string()),
        ..Default::default()
    }
}

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
    ]
}

fn create_test_reminders(contact_id: &str) -> Vec<Reminder> {
    vec![Reminder {
        id: "reminder1".to_string(),
        contact_id: contact_id.to_string(),
        text: "Send proposal".to_string(),
        due_date: "2024-02-05".to_string(),
        created_at: "2024-01-15T10:00:00Z".to_string(),
        completed: false,
        ..Default::default()
    }]
}

#[tokio::test]
async fn test_get_timeline() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let notes = create_test_notes(&contact.id);
    let reminders = create_test_reminders(&contact.id);

    let notes_mock = server
        .mock(
            "GET",
            format!("/timeline_items/contacts/{}", contact.id).as_str(),
        )
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "100".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({"timeline_items": notes}).to_string())
        .create_async()
        .await;

    let reminders_mock = server
        .mock("GET", "/reminders")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "100".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({"reminders": reminders}).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);
    let start = Instant::now();

    let notes_result = client.get_contact_notes(&contact.id, 100, 0);
    let reminders_result = client.get_contact_reminders(&contact.id, 100, 0);
    let duration = start.elapsed();

    println!("✓ Timeline fetch (sequential) took: {:?}", duration);

    assert!(notes_result.is_ok());
    assert!(reminders_result.is_ok());

    notes_mock.assert_async().await;
    reminders_mock.assert_async().await;
}

#[test]
fn test_relationship_history_tools_creation() {
    let server = mockito::Server::new();
    let sync_client = setup_mock_client(&server);
    let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

    let contact_repo =
        Arc::new(DexContactRepository::new(client.clone())) as Arc<dyn ContactRepository>;
    let note_repo = Arc::new(DexNoteRepository::new(client.clone())) as Arc<dyn NoteRepository>;
    let reminder_repo =
        Arc::new(DexReminderRepository::new(client.clone())) as Arc<dyn ReminderRepository>;

    let _tools = RelationshipHistoryTools::new(contact_repo, note_repo, reminder_repo);

    println!("✓ History tools created successfully");
}

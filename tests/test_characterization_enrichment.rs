//! Characterization tests for ContactEnrichmentTools

use dex_mcp_server::client::{AsyncDexClient, AsyncDexClientImpl, DexClient};
use dex_mcp_server::models::{Contact, Note, Reminder};
use dex_mcp_server::repositories::{
    ContactRepository, DexContactRepository, DexNoteRepository, DexReminderRepository,
    NoteRepository, ReminderRepository,
};
use dex_mcp_server::tools::enrichment::ContactEnrichmentTools;
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

fn create_test_note(contact_id: &str) -> Note {
    Note {
        id: "note1".to_string(),
        contact_id: contact_id.to_string(),
        content: "Test note content".to_string(),
        created_at: "2024-01-15T10:00:00Z".to_string(),
        ..Default::default()
    }
}

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

#[tokio::test]
async fn test_enrich_contact_basic() {
    let mut server = Server::new_async().await;
    let contact = create_test_contact();
    let notes = vec![create_test_note(&contact.id)];
    let reminders = vec![create_test_reminder(&contact.id)];

    let notes_mock = server
        .mock(
            "GET",
            format!("/api/contacts/{}/timeline_events", contact.id).as_str(),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(notes).to_string())
        .create_async()
        .await;

    let reminders_mock = server
        .mock(
            "GET",
            format!("/api/contacts/{}/reminders", contact.id).as_str(),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(reminders).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);
    let start = Instant::now();

    let notes_result = client.get_contact_notes(&contact.id, 100, 0);
    let reminders_result = client.get_contact_reminders(&contact.id, 100, 0);
    let duration = start.elapsed();

    println!("✓ Contact enrichment (sequential) took: {:?}", duration);

    assert!(notes_result.is_ok());
    assert!(reminders_result.is_ok());

    notes_mock.assert_async().await;
    reminders_mock.assert_async().await;
}

#[test]
fn test_contact_enrichment_tools_creation() {
    let server = mockito::Server::new();
    let sync_client = setup_mock_client(&server);
    let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

    let contact_repo =
        Arc::new(DexContactRepository::new(client.clone())) as Arc<dyn ContactRepository>;
    let note_repo = Arc::new(DexNoteRepository::new(client.clone())) as Arc<dyn NoteRepository>;
    let reminder_repo =
        Arc::new(DexReminderRepository::new(client.clone())) as Arc<dyn ReminderRepository>;

    let _tools = ContactEnrichmentTools::new(contact_repo, note_repo, reminder_repo);

    println!("✓ Enrichment tools created successfully");
}

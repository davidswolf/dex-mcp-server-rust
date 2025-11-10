//! Characterization tests for ContactDiscoveryTools
//!
//! These tests document the current behavior before refactoring.
//! They help ensure that refactoring doesn't break existing functionality.

use dex_mcp_server::client::{AsyncDexClient, AsyncDexClientImpl, DexClient};
use dex_mcp_server::models::Contact;
use dex_mcp_server::repositories::{ContactRepository, DexContactRepository};
use dex_mcp_server::tools::discovery::ContactDiscoveryTools;
use mockito::Server;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;

/// Helper to create a test DexClient with mock server
fn setup_mock_client(server: &Server) -> DexClient {
    DexClient::with_base_url(
        server.url(),
        "test_api_key".to_string(),
    )
}

/// Helper to create mock contacts
fn create_test_contacts() -> Vec<Contact> {
    vec![
        Contact {
            id: "contact1".to_string(),
            name: "John Doe".to_string(),
            email: Some("john.doe@example.com".to_string()),
            phone: Some("+1234567890".to_string()),
            linkedin: Some("https://www.linkedin.com/in/johndoe".to_string()),
            ..Default::default()
        },
        Contact {
            id: "contact2".to_string(),
            name: "Jane Smith".to_string(),
            email: Some("jane.smith@example.com".to_string()),
            phone: Some("+0987654321".to_string()),
            ..Default::default()
        },
        Contact {
            id: "contact3".to_string(),
            name: "Bob Johnson".to_string(),
            email: Some("bob.johnson@example.com".to_string()),
            ..Default::default()
        },
    ]
}

/// Test: Find contact by exact email match
#[tokio::test]
async fn test_find_contact_by_email() {
    let mut server = Server::new_async().await;
    let contacts = create_test_contacts();

    let mock = server.mock("GET", "/api/contacts")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("email".into(), "john.doe@example.com".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!([contacts[0].clone()]).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);
    let start = Instant::now();

    let result = client.search_contacts_by_email("john.doe@example.com");
    let duration = start.elapsed();

    println!("✓ Email search took: {:?}", duration);

    assert!(result.is_ok(), "Search should succeed");
    let found_contacts = result.unwrap();
    assert_eq!(found_contacts.len(), 1);

    mock.assert_async().await;
}

/// Test: Find contact by name
#[tokio::test]
async fn test_find_contact_by_name() {
    let mut server = Server::new_async().await;
    let contacts = create_test_contacts();

    let mock = server.mock("GET", "/api/contacts")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "100".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(contacts).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);
    let start = Instant::now();

    let result = client.get_contacts(100, 0);
    let duration = start.elapsed();

    println!("✓ Name search (fetch all) took: {:?}", duration);

    assert!(result.is_ok());
    let fetched_contacts = result.unwrap();
    assert_eq!(fetched_contacts.len(), 3);

    mock.assert_async().await;
}

/// Test: ContactDiscoveryTools creation
#[test]
fn test_contact_discovery_tools_creation() {
    let server = mockito::Server::new();
    let sync_client = setup_mock_client(&server);
    let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

    let contact_repo = Arc::new(DexContactRepository::new(client)) as Arc<dyn ContactRepository>;
    let _tools = ContactDiscoveryTools::new(contact_repo, 300);

    println!("✓ Tools created successfully");
}

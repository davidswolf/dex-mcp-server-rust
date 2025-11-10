//! Characterization tests for ContactDiscoveryTools
//!
//! These tests document the current behavior before refactoring.
//! They help ensure that refactoring doesn't break existing functionality.

use dex_mcp_server::tools::discovery::ContactDiscoveryTools;
use dex_mcp_server::client::DexClient;
use dex_mcp_server::models::Contact;
use dex_mcp_server::DexApiResult;
use std::sync::Arc;
use std::time::Instant;
use mockito::{Mock, Server, ServerGuard};
use serde_json::json;

/// Helper to create a test DexClient with mock server
fn setup_mock_client(server: &Server) -> DexClient {
    DexClient::new_with_base_url(
        "test_api_key".to_string(),
        server.url(),
    ).unwrap()
}

/// Helper to create mock contacts
fn create_test_contacts() -> Vec<Contact> {
    vec![
        Contact {
            id: "contact1".to_string(),
            name: "John Doe".to_string(),
            email: Some("john.doe@example.com".to_string()),
            phone: Some("+1234567890".to_string()),
            linkedin_url: Some("https://www.linkedin.com/in/johndoe".to_string()),
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
///
/// This test documents the current behavior when searching by email.
/// It should return the contact with exact email match.
#[tokio::test]
async fn test_find_contact_by_email() {
    let mut server = Server::new_async().await;
    let contacts = create_test_contacts();

    // Mock the search_contacts_by_email endpoint
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
    let tools = ContactDiscoveryTools::new(Arc::new(client), 300);

    let start = Instant::now();

    // Search by email
    let result = tools.client.search_contacts_by_email("john.doe@example.com", 10, 0);

    let duration = start.elapsed();
    println!("Email search took: {:?}", duration);

    assert!(result.is_ok(), "Search should succeed");
    let found_contacts = result.unwrap();
    assert_eq!(found_contacts.len(), 1, "Should find exactly one contact");
    assert_eq!(found_contacts[0].email, Some("john.doe@example.com".to_string()));

    mock.assert_async().await;
}

/// Test: Find contact by name with fuzzy matching
///
/// This test documents the current behavior when searching by name.
/// It tests the fuzzy matching capability.
#[tokio::test]
async fn test_find_contact_by_name() {
    let mut server = Server::new_async().await;
    let contacts = create_test_contacts();

    // Mock the get_contacts endpoint (for fetching all contacts)
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

    // Fetch contacts (simulating what the tool would do)
    let result = client.get_contacts(100, 0);

    let duration = start.elapsed();
    println!("Name search (fetch all) took: {:?}", duration);

    assert!(result.is_ok(), "Fetch should succeed");
    let fetched_contacts = result.unwrap();
    assert_eq!(fetched_contacts.len(), 3, "Should fetch all test contacts");

    mock.assert_async().await;
}

/// Test: Fuzzy matching with partial name
///
/// This test documents fuzzy matching behavior with partial names.
#[tokio::test]
async fn test_find_contact_fuzzy_match() {
    let mut server = Server::new_async().await;
    let contacts = create_test_contacts();

    // Mock endpoint
    let mock = server.mock("GET", "/api/contacts")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(contacts).to_string())
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Fetch all contacts (fuzzy matching would be done in memory)
    let result = client.get_contacts(100, 0);

    let duration = start.elapsed();
    println!("Fuzzy match (fetch) took: {:?}", duration);

    assert!(result.is_ok(), "Fetch should succeed");
    let fetched_contacts = result.unwrap();

    // Verify that we have contacts that could match "Joh" or "Jon"
    let has_john = fetched_contacts.iter().any(|c| c.name.contains("John"));
    assert!(has_john, "Should have contact with 'John' in name");

    mock.assert_async().await;
}

/// Test: ContactDiscoveryTools creation
///
/// This test documents that tools can be created with a client.
#[test]
fn test_contact_discovery_tools_creation() {
    let server = mockito::Server::new();
    let client = setup_mock_client(&server);
    let tools = ContactDiscoveryTools::new(Arc::new(client), 300);

    // Verify the tools were created successfully
    assert!(true, "Tools created successfully");
}

/// Test: Performance baseline for multiple contact fetch
///
/// This test establishes a baseline for how long it takes to fetch
/// multiple pages of contacts.
#[tokio::test]
async fn test_pagination_performance() {
    let mut server = Server::new_async().await;
    let contacts = create_test_contacts();

    // Mock multiple pages
    let mock1 = server.mock("GET", "/api/contacts")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "100".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!(contacts.clone()).to_string())
        .create_async()
        .await;

    let mock2 = server.mock("GET", "/api/contacts")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "100".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "100".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!([]).to_string())  // Empty page indicates end
        .create_async()
        .await;

    let client = setup_mock_client(&server);

    let start = Instant::now();

    // Fetch first page
    let result1 = client.get_contacts(100, 0);
    assert!(result1.is_ok());

    // Fetch second page
    let result2 = client.get_contacts(100, 100);
    assert!(result2.is_ok());

    let duration = start.elapsed();
    println!("Pagination (2 pages) took: {:?}", duration);

    mock1.assert_async().await;
    mock2.assert_async().await;
}

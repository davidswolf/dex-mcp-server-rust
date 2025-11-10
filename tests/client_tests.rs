//! Integration tests for the DexClient using mockito for HTTP mocking.

use dex_mcp_server::{Contact, DexClient, Note, Reminder};
use mockito::{Matcher, Server};

#[test]
fn test_get_contacts() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/contacts")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "100".into()),
            Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .match_header("x-hasura-dex-api-key", "test-api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "contacts": [{
                "id": "contact1",
                "first_name": "John",
                "last_name": "Doe",
                "emails": [{"email": "john@example.com"}]
            }]
        }"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let contacts = client.get_contacts(100, 0).unwrap();

    mock.assert();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].id, "contact1");
    assert_eq!(contacts[0].name, "John Doe");
}

#[test]
fn test_get_contacts_paginated_response() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/contacts")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "50".into()),
            Matcher::UrlEncoded("offset".into(), "10".into()),
        ]))
        .match_header("x-hasura-dex-api-key", "test-api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "contacts": [{
                "id": "contact2",
                "first_name": "Jane",
                "last_name": "Smith",
                "emails": [{"email": "jane@example.com"}]
            }],
            "pagination": {
                "total": {
                    "count": 100
                }
            }
        }"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let contacts = client.get_contacts(50, 10).unwrap();

    mock.assert();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].id, "contact2");
}

#[test]
fn test_get_contact() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/contacts/contact123")
        .match_header("x-hasura-dex-api-key", "test-api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "contacts": [{
                "id": "contact123",
                "first_name": "Test",
                "last_name": "User",
                "emails": [{"email": "test@example.com"}],
                "phones": [{"phone_number": "+1234567890"}]
            }]
        }"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let contact = client.get_contact("contact123").unwrap();

    mock.assert();
    assert_eq!(contact.id, "contact123");
    assert_eq!(contact.name, "Test User");
    assert_eq!(contact.email, Some("test@example.com".to_string()));
}

#[test]
fn test_get_contact_not_found() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/contacts/nonexistent")
        .with_status(404)
        .with_body("Contact not found")
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let result = client.get_contact("nonexistent");

    mock.assert();
    assert!(result.is_err());
    match result {
        Err(dex_mcp_server::DexApiError::NotFound(msg)) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_create_contact() {
    let mut server = Server::new();

    let mock = server
        .mock("POST", "/contacts")
        .match_header("x-hasura-dex-api-key", "test-api-key")
        .match_header("content-type", "application/json")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "id": "new-contact",
            "first_name": "New",
            "last_name": "User",
            "emails": [{"email": "new@example.com"}]
        }"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());

    let new_contact = Contact::new("".to_string(), "New User".to_string());
    let created = client.create_contact(&new_contact).unwrap();

    mock.assert();
    assert_eq!(created.id, "new-contact");
    assert_eq!(created.name, "New User");
}

#[test]
fn test_update_contact() {
    let mut server = Server::new();

    let mock = server
        .mock("PUT", "/contacts/contact123")
        .match_header("x-hasura-dex-api-key", "test-api-key")
        .match_header("content-type", "application/json")
        .with_status(200)
        .with_body(
            r#"{
            "id": "contact123",
            "first_name": "Updated",
            "last_name": "User",
            "emails": [{"email": "updated@example.com"}]
        }"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());

    let mut contact = Contact::new("contact123".to_string(), "Updated User".to_string());
    contact.email = Some("updated@example.com".to_string());

    let updated = client.update_contact("contact123", &contact).unwrap();

    mock.assert();
    assert_eq!(updated.name, "Updated User");
}

#[test]
fn test_delete_contact() {
    let mut server = Server::new();

    let mock = server
        .mock("DELETE", "/contacts/contact123")
        .match_header("x-hasura-dex-api-key", "test-api-key")
        .with_status(204)
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let result = client.delete_contact("contact123");

    mock.assert();
    assert!(result.is_ok());
}

#[test]
fn test_search_contacts_by_email() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/contacts/search")
        .match_query(Matcher::UrlEncoded(
            "email".into(),
            "john@example.com".into(),
        ))
        .match_header("x-hasura-dex-api-key", "test-api-key")
        .with_status(200)
        .with_body(
            r#"[{
            "id": "contact1",
            "first_name": "John",
            "last_name": "Doe",
            "emails": [{"email": "john@example.com"}]
        }]"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let contacts = client.search_contacts_by_email("john@example.com").unwrap();

    mock.assert();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].email, Some("john@example.com".to_string()));
}

#[test]
fn test_get_contact_notes() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/timeline_items/contacts/contact123")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "50".into()),
            Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(200)
        .with_body(
            r#"{
            "timeline_items": [{
                "id": "note1",
                "contacts": [{"contact_id": "contact123"}],
                "note": "Test note",
                "event_time": "2024-01-15T10:00:00Z"
            }]
        }"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let notes = client.get_contact_notes("contact123", 50, 0).unwrap();

    mock.assert();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, "note1");
    assert_eq!(notes[0].content, "Test note");
}

#[test]
fn test_create_note() {
    let mut server = Server::new();

    let mock = server
        .mock("POST", "/notes")
        .with_status(201)
        .with_body(
            r#"{
            "id": "new-note",
            "contacts": [{"contact_id": "contact123"}],
            "note": "New note content",
            "event_time": "2024-01-15T10:00:00Z"
        }"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());

    let note = Note::new(
        "".to_string(),
        "contact123".to_string(),
        "New note content".to_string(),
        "2024-01-15T10:00:00Z".to_string(),
    );

    let created = client.create_note(&note).unwrap();

    mock.assert();
    assert_eq!(created.id, "new-note");
    assert_eq!(created.content, "New note content");
}

#[test]
fn test_get_contact_reminders() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/reminders")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "1000".into()),
            Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(200)
        .with_body(
            r#"{
            "reminders": [{
                "id": "reminder1",
                "contact_ids": [{"contact_id": "contact123"}],
                "body": "Follow up",
                "due_at_date": "2024-02-01",
                "is_complete": false
            }, {
                "id": "reminder2",
                "contact_ids": [{"contact_id": "other-contact"}],
                "body": "Different contact",
                "due_at_date": "2024-02-01",
                "is_complete": false
            }]
        }"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let reminders = client.get_contact_reminders("contact123", 50, 0).unwrap();

    mock.assert();
    assert_eq!(reminders.len(), 1);
    assert_eq!(reminders[0].id, "reminder1");
    assert_eq!(reminders[0].text, "Follow up");
}

#[test]
fn test_create_reminder() {
    let mut server = Server::new();

    let mock = server
        .mock("POST", "/reminders")
        .with_status(201)
        .with_body(
            r#"{
            "id": "new-reminder",
            "contact_ids": [{"contact_id": "contact123"}],
            "body": "New reminder",
            "due_at_date": "2024-03-01",
            "is_complete": false
        }"#,
        )
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());

    let reminder = Reminder::new(
        "".to_string(),
        "contact123".to_string(),
        "New reminder".to_string(),
        "2024-03-01".to_string(),
        "2024-01-15T10:00:00Z".to_string(),
    );

    let created = client.create_reminder(&reminder).unwrap();

    mock.assert();
    assert_eq!(created.id, "new-reminder");
    assert_eq!(created.text, "New reminder");
}

#[test]
fn test_unauthorized_error() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/contacts")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "100".into()),
            Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(401)
        .with_body("Unauthorized")
        .create();

    let client = DexClient::with_base_url(server.url(), "invalid-key".to_string());
    let result = client.get_contacts(100, 0);

    mock.assert();
    assert!(result.is_err());
    match result {
        Err(dex_mcp_server::DexApiError::Unauthorized) => {}
        _ => panic!("Expected Unauthorized error"),
    }
}

#[test]
fn test_rate_limit_error() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/contacts")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "100".into()),
            Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(429)
        .with_body("Rate limit exceeded")
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let result = client.get_contacts(100, 0);

    mock.assert();
    assert!(result.is_err());
    match result {
        Err(dex_mcp_server::DexApiError::RateLimitExceeded) => {}
        _ => panic!("Expected RateLimitExceeded error"),
    }
}

#[test]
fn test_generic_api_error() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/contacts")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "100".into()),
            Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .with_status(500)
        .with_body("Internal server error")
        .create();

    let client = DexClient::with_base_url(server.url(), "test-api-key".to_string());
    let result = client.get_contacts(100, 0);

    mock.assert();
    assert!(result.is_err());
    match result {
        Err(dex_mcp_server::DexApiError::ApiError { status, message }) => {
            assert_eq!(status, 500);
            assert!(message.contains("Internal server error"));
        }
        _ => panic!("Expected ApiError"),
    }
}

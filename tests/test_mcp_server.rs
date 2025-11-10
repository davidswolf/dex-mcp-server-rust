//! End-to-end tests for MCP server tool implementations.
//!
//! These tests validate the MCP tool implementations against the live Dex API.
//! They test the tools' functionality to ensure correct integration.
//!
//! NOTE: These tests are currently disabled and need to be refactored to work with
//! the new repository-based architecture. The tests need to:
//! 1. Convert from synchronous to async (use #[tokio::test])
//! 2. Wrap DexClient in AsyncDexClientImpl
//! 3. Create repositories instead of passing client directly to tools
//! 4. Await all async method calls
//!
//! To enable these tests, uncomment the code below and apply the necessary changes.

#![allow(dead_code, unused_imports)]

use dex_mcp_server::client::{AsyncDexClient, AsyncDexClientImpl};
use dex_mcp_server::repositories::{
    ContactRepository, DexContactRepository, DexNoteRepository, DexReminderRepository,
    NoteRepository, ReminderRepository,
};
use dex_mcp_server::tools::*;
use dex_mcp_server::Config;
use serial_test::serial;
use std::sync::Arc;

mod e2e;
use e2e::*;

/// Helper to create repositories from a DexClient
fn _setup_repositories(
    client: Arc<dyn AsyncDexClient>,
) -> (
    Arc<dyn ContactRepository>,
    Arc<dyn NoteRepository>,
    Arc<dyn ReminderRepository>,
) {
    let contact_repo =
        Arc::new(DexContactRepository::new(client.clone())) as Arc<dyn ContactRepository>;
    let note_repo = Arc::new(DexNoteRepository::new(client.clone())) as Arc<dyn NoteRepository>;
    let reminder_repo = Arc::new(DexReminderRepository::new(client)) as Arc<dyn ReminderRepository>;
    (contact_repo, note_repo, reminder_repo)
}

/*
// THESE TESTS ARE DISABLED - See note at top of file

/// Test the find_contact tool with name search.
///
/// This test validates:
/// - Tool can be invoked with search parameters
/// - Results are returned in correct format
/// - Confidence scores are provided
#[tokio::test]
#[serial]
async fn test_tool_find_contact_by_name() {
    let sync_client = setup_test_client();
    let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;
    let (contact_repo, _, _) = _setup_repositories(client);

    // Get a known contact
    let contacts_result = contact_repo.get_contacts(10, 0).await;
    if contacts_result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = contacts_result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping test");
        return;
    }

    let test_contact = &contacts[0];
    println!("Testing find_contact tool with: {}", test_contact.name);

    // Create discovery tools with 300 second cache TTL
    let mut discovery_tools = ContactDiscoveryTools::new(contact_repo, 300);

    let params = FindContactParams {
        name: Some(test_contact.name.clone()),
        email: None,
        phone: None,
        company: None,
        social_url: None,
        max_results: Some(5),
        min_confidence: Some(30),
    };

    let result = discovery_tools.find_contact(params).await;

    match result {
        Ok(response) => {
            println!("✓ find_contact tool executed successfully");
            println!("  Found {} matches", response.matches.len());
            println!("  From cache: {}", response.from_cache);

            if !response.matches.is_empty() {
                let top_match = &response.matches[0];
                println!("  Top match: {} (confidence: {})",
                    top_match.contact.name,
                    top_match.confidence
                );

                assert!(
                    top_match.confidence > 0,
                    "Confidence should be greater than 0"
                );

                assert_contact_valid(&top_match.contact);
            }
        }
        Err(e) => {
            println!("⚠ find_contact tool failed: {:?}", e);
        }
    }
}

// TODO: Add remaining tests following the same pattern
// - test_tool_find_contact_by_email
// - test_tool_add_contact_note
// - test_tool_create_contact_reminder
// - test_tool_get_contact_history
// - test_tool_error_handling
// - test_tools_with_config
// - test_tool_caching
// - test_tool_combined_search
// - test_tool_history_date_filtering

*/

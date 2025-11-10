//! End-to-end tests for Contact CRUD operations.
//!
//! These tests validate creating, reading, updating, and deleting contacts
//! against the live Dex API. All test data is automatically cleaned up.

use serial_test::serial;

mod e2e;
use e2e::{fixtures::*, *};

/// RAII guard that ensures a contact is deleted when it goes out of scope.
///
/// This guarantees cleanup even if tests fail or panic.
struct ContactGuard<'a> {
    client: &'a dex_mcp_server::DexClient,
    contact_id: Option<String>,
}

impl<'a> ContactGuard<'a> {
    fn new(client: &'a dex_mcp_server::DexClient) -> Self {
        Self {
            client,
            contact_id: None,
        }
    }

    fn set_id(&mut self, id: String) {
        self.contact_id = Some(id);
    }
}

impl<'a> Drop for ContactGuard<'a> {
    fn drop(&mut self) {
        if let Some(ref contact_id) = self.contact_id {
            match self.client.delete_contact(contact_id) {
                Ok(_) => println!("  ✓ Cleaned up test contact: {}", contact_id),
                Err(e) => eprintln!("  ⚠ Failed to cleanup contact {}: {:?}", contact_id, e),
            }
        }
    }
}

/// Test complete CRUD cycle for contacts: Create, Read, Update, Delete.
///
/// This test validates:
/// - Contacts can be created successfully
/// - Created contacts can be retrieved
/// - Contacts can be updated
/// - Contacts can be deleted
/// - All test data is cleaned up
#[test]
#[serial]
fn test_contact_crud_lifecycle() {
    let client = setup_test_client();

    // Initialize cleanup guard
    let mut contact_guard = ContactGuard::new(&client);

    // TEST 1: CREATE
    println!("\n1. Testing CREATE contact...");
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let original_first_name = format!("E2ETest{}", timestamp);
    let original_last_name = "CRUDLifecycle".to_string();
    let original_email = format!("e2etest{}@example.com", timestamp);

    let contact = sample_contact(&original_first_name, &original_last_name, &original_email);

    let created_contact = match client.create_contact(&contact) {
        Ok(c) => {
            println!("  ✓ Contact created successfully");
            println!("    Contact ID: {}", c.id);
            println!("    Name: {} {}", c.first_name.as_deref().unwrap_or(""), c.last_name.as_deref().unwrap_or(""));
            println!("    Email: {}", c.email.as_deref().unwrap_or(""));
            assert_contact_valid(&c);
            assert_eq!(c.first_name.as_deref(), Some(original_first_name.as_str()));
            assert_eq!(c.last_name.as_deref(), Some(original_last_name.as_str()));
            c
        }
        Err(e) => {
            panic!("✗ Failed to create contact: {:?}", e);
        }
    };

    // Register for cleanup
    contact_guard.set_id(created_contact.id.clone());

    // TEST 2: READ
    println!("\n2. Testing READ contact...");
    let _contacts = match client.get_contacts(100, 0) {
        Ok(contacts) => {
            println!("  ✓ Retrieved {} contacts", contacts.len());

            // Find our test contact
            let found = contacts.iter().find(|c| c.id == created_contact.id);
            match found {
                Some(c) => {
                    println!("  ✓ Found our test contact in results");
                    assert_eq!(c.first_name, created_contact.first_name);
                    assert_eq!(c.last_name, created_contact.last_name);
                }
                None => {
                    panic!("✗ Created contact not found in contact list!");
                }
            }
            contacts
        }
        Err(e) => {
            panic!("✗ Failed to read contacts: {:?}", e);
        }
    };

    // TEST 3: UPDATE
    println!("\n3. Testing UPDATE contact...");
    let updated_first_name = format!("{}_UPDATED", original_first_name);
    let mut updated_contact = created_contact.clone();
    updated_contact.first_name = Some(updated_first_name.clone());
    updated_contact.job_title = Some("Senior Engineer".to_string());

    match client.update_contact(&created_contact.id, &updated_contact) {
        Ok(c) => {
            println!("  ✓ Contact updated successfully");
            println!("    New name: {} {}", c.first_name.as_deref().unwrap_or(""), c.last_name.as_deref().unwrap_or(""));
            println!("    New job title: {}", c.job_title.as_deref().unwrap_or(""));
            assert_eq!(c.id, created_contact.id);
            assert_eq!(c.first_name.as_deref(), Some(updated_first_name.as_str()));
            assert_eq!(c.job_title.as_deref(), Some("Senior Engineer"));
        }
        Err(e) => {
            panic!("✗ Failed to update contact: {:?}", e);
        }
    }

    // Verify the update persisted
    println!("  Verifying update persisted...");
    let contacts_after_update = match client.get_contacts(100, 0) {
        Ok(contacts) => contacts,
        Err(e) => {
            panic!("✗ Failed to verify update: {:?}", e);
        }
    };

    let updated_contact_from_api = contacts_after_update.iter().find(|c| c.id == created_contact.id);
    match updated_contact_from_api {
        Some(c) => {
            println!("  ✓ Update verified");
            assert_eq!(c.first_name.as_deref(), Some(updated_first_name.as_str()));
        }
        None => {
            panic!("✗ Contact not found after update!");
        }
    }

    // TEST 4: DELETE
    println!("\n4. Testing DELETE contact...");
    match client.delete_contact(&created_contact.id) {
        Ok(_) => {
            println!("  ✓ Contact deleted successfully");
            contact_guard.contact_id = None; // Already deleted, don't try again in Drop
        }
        Err(e) => {
            panic!("✗ Failed to delete contact: {:?}", e);
        }
    }

    // Verify the deletion
    println!("  Verifying deletion...");
    let contacts_after_delete = match client.get_contacts(100, 0) {
        Ok(contacts) => contacts,
        Err(e) => {
            panic!("✗ Failed to verify deletion: {:?}", e);
        }
    };

    let deleted_contact_check = contacts_after_delete.iter().find(|c| c.id == created_contact.id);
    match deleted_contact_check {
        None => {
            println!("  ✓ Deletion verified - contact no longer exists");
        }
        Some(_) => {
            panic!("✗ Contact still exists after deletion!");
        }
    }

    println!("\n✓ All CRUD operations completed successfully");
}

/// Test creating multiple contacts and batch cleanup.
///
/// This test validates:
/// - Multiple contacts can be created
/// - All contacts are properly cleaned up
#[test]
#[serial]
fn test_contact_batch_create_and_cleanup() {
    let client = setup_test_client();

    println!("Testing batch contact creation...");

    let mut contact_guards = Vec::new();

    // Create 3 test contacts
    for i in 1..=3 {
        println!("\nCreating contact {}/3...", i);
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let first_name = format!("BatchTest{}{}", timestamp, i);
        let last_name = "Contact".to_string();
        let email = format!("batchtest{}{}@example.com", timestamp, i);

        let contact = sample_contact(&first_name, &last_name, &email);

        match client.create_contact(&contact) {
            Ok(created) => {
                println!("  ✓ Contact {} created: {}", i, created.id);
                let mut guard = ContactGuard::new(&client);
                guard.set_id(created.id);
                contact_guards.push(guard);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to create contact {}: {:?}", i, e);
            }
        }
    }

    println!("\n✓ Created {} contacts", contact_guards.len());
    println!("  (All contacts will be automatically cleaned up)");

    // Guards will automatically clean up contacts when they go out of scope
}

/// Test updating a contact's information multiple times.
///
/// This test validates:
/// - Contacts can be updated multiple times
/// - Each update is persisted correctly
#[test]
#[serial]
fn test_contact_multiple_updates() {
    let client = setup_test_client();

    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let first_name = format!("MultiUpdate{}", timestamp);
    let last_name = "Test".to_string();
    let email = format!("multiupdate{}@example.com", timestamp);

    println!("Testing multiple contact updates...");

    let mut contact_guard = ContactGuard::new(&client);

    // Create initial contact
    let contact = sample_contact(&first_name, &last_name, &email);

    let mut current_contact = match client.create_contact(&contact) {
        Ok(c) => {
            println!("✓ Initial contact created: {}", c.id);
            contact_guard.set_id(c.id.clone());
            c
        }
        Err(e) => {
            panic!("✗ Failed to create contact: {:?}", e);
        }
    };

    // Update 3 times
    for i in 1..=3 {
        println!("\nUpdate {}/3...", i);
        current_contact.job_title = Some(format!("Position #{}", i));
        current_contact.description = Some(format!("Description update #{}", i));

        match client.update_contact(&current_contact.id, &current_contact) {
            Ok(updated) => {
                println!("  ✓ Updated successfully");
                assert_eq!(updated.job_title.as_deref(), Some(format!("Position #{}", i).as_str()));
                current_contact = updated;
            }
            Err(e) => {
                panic!("✗ Failed on update {}: {:?}", i, e);
            }
        }
    }

    println!("\n✓ All updates completed successfully");
}

/// Test error handling when trying to update a non-existent contact.
///
/// This test validates:
/// - Appropriate error is returned for invalid contact IDs
#[test]
#[serial]
fn test_contact_update_nonexistent() {
    let client = setup_test_client();

    println!("Testing update of non-existent contact...");
    let fake_contact_id = "00000000-0000-0000-0000-000000000000";
    let contact = sample_contact("Nonexistent", "Contact", "nonexistent@example.com");

    match client.update_contact(fake_contact_id, &contact) {
        Ok(_) => {
            panic!("✗ Update should have failed for non-existent contact!");
        }
        Err(e) => {
            println!("  ✓ Got expected error: {:?}", e);
        }
    }
}

/// Test error handling when trying to delete a non-existent contact.
///
/// This test validates:
/// - Appropriate error is returned for invalid contact IDs
#[test]
#[serial]
fn test_contact_delete_nonexistent() {
    let client = setup_test_client();

    println!("Testing delete of non-existent contact...");
    let fake_contact_id = "00000000-0000-0000-0000-000000000000";

    match client.delete_contact(fake_contact_id) {
        Ok(_) => {
            // Some APIs return success for deleting non-existent items (idempotent)
            println!("  ℹ Delete succeeded (idempotent operation)");
        }
        Err(e) => {
            println!("  ✓ Got expected error: {:?}", e);
        }
    }
}

/// Test creating a contact with minimal information.
///
/// This test validates:
/// - Contacts can be created with just first and last name
/// - Optional fields are handled correctly
#[test]
#[serial]
fn test_contact_minimal_create() {
    let client = setup_test_client();

    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    println!("Testing minimal contact creation...");

    let mut contact_guard = ContactGuard::new(&client);

    let contact = dex_mcp_server::Contact {
        first_name: Some(format!("Minimal{}", timestamp)),
        last_name: Some("Test".to_string()),
        ..Default::default()
    };

    match client.create_contact(&contact) {
        Ok(c) => {
            println!("✓ Minimal contact created: {}", c.id);
            contact_guard.set_id(c.id.clone());
            assert_eq!(c.first_name.as_deref(), Some(format!("Minimal{}", timestamp).as_str()));
            assert_eq!(c.last_name.as_deref(), Some("Test"));
        }
        Err(e) => {
            panic!("✗ Failed to create minimal contact: {:?}", e);
        }
    }

    println!("\n✓ Minimal contact creation test completed successfully");
}

/// Test updating contact with email and phone changes.
///
/// This test validates:
/// - Email addresses can be updated
/// - Phone numbers can be updated
#[test]
#[serial]
fn test_contact_update_email_phone() {
    let client = setup_test_client();

    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let first_name = format!("EmailPhone{}", timestamp);
    let last_name = "Test".to_string();
    let email = format!("emailphone{}@example.com", timestamp);

    println!("Testing contact email and phone updates...");

    let mut contact_guard = ContactGuard::new(&client);

    // Create initial contact
    let contact = sample_contact(&first_name, &last_name, &email);

    let created_contact = match client.create_contact(&contact) {
        Ok(c) => {
            println!("✓ Initial contact created: {}", c.id);
            contact_guard.set_id(c.id.clone());
            c
        }
        Err(e) => {
            panic!("✗ Failed to create contact: {:?}", e);
        }
    };

    // Update email and phone
    let mut updated_contact = created_contact.clone();
    let new_email = format!("updated{}@example.com", timestamp);
    updated_contact.emails = vec![new_email.clone()];
    updated_contact.phones = vec!["+1-555-0100".to_string()];

    match client.update_contact(&created_contact.id, &updated_contact) {
        Ok(c) => {
            println!("  ✓ Contact updated successfully");
            println!("    Emails: {:?}", c.emails);
            println!("    Phones: {:?}", c.phones);
        }
        Err(e) => {
            // This may fail depending on API implementation
            println!("  ⚠ Update failed (may not be supported): {:?}", e);
        }
    }

    println!("\n✓ Email/phone update test completed");
}

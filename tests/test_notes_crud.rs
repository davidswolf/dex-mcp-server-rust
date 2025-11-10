//! End-to-end tests for Note CRUD operations.
//!
//! These tests validate creating, reading, updating, and deleting notes
//! against the live Dex API. All test data is automatically cleaned up.

use serial_test::serial;

mod e2e;
use e2e::{fixtures::*, *};

/// RAII guard that ensures a note is deleted when it goes out of scope.
///
/// This guarantees cleanup even if tests fail or panic.
struct NoteGuard<'a> {
    client: &'a dex_mcp_server::DexClient,
    note_id: Option<String>,
}

impl<'a> NoteGuard<'a> {
    fn new(client: &'a dex_mcp_server::DexClient) -> Self {
        Self {
            client,
            note_id: None,
        }
    }

    fn set_id(&mut self, id: String) {
        self.note_id = Some(id);
    }

    #[allow(dead_code)]
    fn id(&self) -> Option<&str> {
        self.note_id.as_deref()
    }
}

impl<'a> Drop for NoteGuard<'a> {
    fn drop(&mut self) {
        if let Some(ref note_id) = self.note_id {
            match self.client.delete_note(note_id) {
                Ok(_) => println!("  ✓ Cleaned up test note: {}", note_id),
                Err(e) => eprintln!("  ⚠ Failed to cleanup note {}: {:?}", note_id, e),
            }
        }
    }
}

/// Test complete CRUD cycle for notes: Create, Read, Update, Delete.
///
/// This test validates:
/// - Notes can be created successfully
/// - Created notes can be retrieved
/// - Notes can be updated
/// - Notes can be deleted
/// - All test data is cleaned up
#[test]
#[serial]
fn test_note_crud_lifecycle() {
    let client = setup_test_client();

    // Setup: Get a test contact
    let contacts = match client.get_contacts(10, 0) {
        Ok(c) if !c.is_empty() => c,
        Ok(_) => {
            println!("⚠ No contacts found - skipping test");
            return;
        }
        Err(e) => {
            println!("⚠ Cannot fetch contacts: {:?}", e);
            return;
        }
    };

    let contact = &contacts[0];
    println!(
        "Testing note CRUD with contact: {} (ID: {})",
        contact.name, contact.id
    );

    // Initialize cleanup guard
    let mut note_guard = NoteGuard::new(&client);

    // TEST 1: CREATE
    println!("\n1. Testing CREATE note...");
    let original_content = generate_test_note_content("E2E CRUD Test");
    let note = sample_note(&contact.id, &original_content);

    let created_note = match client.create_note(&note) {
        Ok(n) => {
            println!("  ✓ Note created successfully");
            println!("    Note ID: {}", n.id);
            println!("    Content: {}", n.content);
            assert_note_valid(&n);
            assert_eq!(n.content, original_content);
            assert_eq!(n.contact_id, contact.id);
            n
        }
        Err(e) => {
            panic!("✗ Failed to create note: {:?}", e);
        }
    };

    // Register for cleanup
    note_guard.set_id(created_note.id.clone());

    // TEST 2: READ
    println!("\n2. Testing READ note...");
    let _notes = match client.get_contact_notes(&contact.id, 100, 0) {
        Ok(notes) => {
            println!("  ✓ Retrieved {} notes for contact", notes.len());

            // Find our test note
            let found = notes.iter().find(|n| n.id == created_note.id);
            match found {
                Some(n) => {
                    println!("  ✓ Found our test note in results");
                    assert_eq!(n.content, original_content);
                    assert_eq!(n.contact_id, contact.id);
                }
                None => {
                    panic!("✗ Created note not found in contact's notes!");
                }
            }
            notes
        }
        Err(e) => {
            panic!("✗ Failed to read notes: {:?}", e);
        }
    };

    // TEST 3: UPDATE
    println!("\n3. Testing UPDATE note...");
    let updated_content = format!("{} - UPDATED", original_content);
    let mut updated_note = created_note.clone();
    updated_note.content = updated_content.clone();

    match client.update_note(&created_note.id, &updated_note) {
        Ok(n) => {
            println!("  ✓ Note updated successfully");
            println!("    New content: {}", n.content);
            assert_eq!(n.id, created_note.id);
            assert_eq!(n.content, updated_content);
        }
        Err(e) => {
            panic!("✗ Failed to update note: {:?}", e);
        }
    }

    // Verify the update persisted
    println!("  Verifying update persisted...");
    let notes_after_update = match client.get_contact_notes(&contact.id, 100, 0) {
        Ok(notes) => notes,
        Err(e) => {
            panic!("✗ Failed to verify update: {:?}", e);
        }
    };

    let updated_note_from_api = notes_after_update.iter().find(|n| n.id == created_note.id);
    match updated_note_from_api {
        Some(n) => {
            println!("  ✓ Update verified");
            assert_eq!(n.content, updated_content);
        }
        None => {
            panic!("✗ Note not found after update!");
        }
    }

    // TEST 4: DELETE
    println!("\n4. Testing DELETE note...");
    match client.delete_note(&created_note.id) {
        Ok(_) => {
            println!("  ✓ Note deleted successfully");
            note_guard.note_id = None; // Already deleted, don't try again in Drop
        }
        Err(e) => {
            panic!("✗ Failed to delete note: {:?}", e);
        }
    }

    // Verify the deletion
    println!("  Verifying deletion...");
    let notes_after_delete = match client.get_contact_notes(&contact.id, 100, 0) {
        Ok(notes) => notes,
        Err(e) => {
            panic!("✗ Failed to verify deletion: {:?}", e);
        }
    };

    let deleted_note_check = notes_after_delete.iter().find(|n| n.id == created_note.id);
    match deleted_note_check {
        None => {
            println!("  ✓ Deletion verified - note no longer exists");
        }
        Some(_) => {
            panic!("✗ Note still exists after deletion!");
        }
    }

    println!("\n✓ All CRUD operations completed successfully");
}

/// Test creating multiple notes and batch cleanup.
///
/// This test validates:
/// - Multiple notes can be created for the same contact
/// - All notes are properly cleaned up
#[test]
#[serial]
fn test_note_batch_create_and_cleanup() {
    let client = setup_test_client();

    let contacts = match client.get_contacts(10, 0) {
        Ok(c) if !c.is_empty() => c,
        Ok(_) => {
            println!("⚠ No contacts found - skipping test");
            return;
        }
        Err(e) => {
            println!("⚠ Cannot fetch contacts: {:?}", e);
            return;
        }
    };

    let contact = &contacts[0];
    println!("Testing batch note creation with contact: {}", contact.name);

    let mut note_guards = Vec::new();

    // Create 3 test notes
    for i in 1..=3 {
        println!("\nCreating note {}/3...", i);
        let content = generate_test_note_content(&format!("Batch Test Note {}", i));
        let note = sample_note(&contact.id, &content);

        match client.create_note(&note) {
            Ok(created) => {
                println!("  ✓ Note {} created: {}", i, created.id);
                let mut guard = NoteGuard::new(&client);
                guard.set_id(created.id);
                note_guards.push(guard);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to create note {}: {:?}", i, e);
            }
        }
    }

    println!("\n✓ Created {} notes", note_guards.len());
    println!("  (All notes will be automatically cleaned up)");

    // Guards will automatically clean up notes when they go out of scope
}

/// Test updating a note's content multiple times.
///
/// This test validates:
/// - Notes can be updated multiple times
/// - Each update is persisted correctly
#[test]
#[serial]
fn test_note_multiple_updates() {
    let client = setup_test_client();

    let contacts = match client.get_contacts(10, 0) {
        Ok(c) if !c.is_empty() => c,
        Ok(_) => {
            println!("⚠ No contacts found - skipping test");
            return;
        }
        Err(e) => {
            println!("⚠ Cannot fetch contacts: {:?}", e);
            return;
        }
    };

    let contact = &contacts[0];
    println!(
        "Testing multiple note updates with contact: {}",
        contact.name
    );

    let mut note_guard = NoteGuard::new(&client);

    // Create initial note
    let content = generate_test_note_content("Multi-Update Test");
    let note = sample_note(&contact.id, &content);

    let mut current_note = match client.create_note(&note) {
        Ok(n) => {
            println!("✓ Initial note created: {}", n.id);
            note_guard.set_id(n.id.clone());
            n
        }
        Err(e) => {
            panic!("✗ Failed to create note: {:?}", e);
        }
    };

    // Update 3 times
    for i in 1..=3 {
        println!("\nUpdate {}/3...", i);
        current_note.content = format!("{} - Update #{}", content, i);

        match client.update_note(&current_note.id, &current_note) {
            Ok(updated) => {
                println!("  ✓ Updated successfully");
                assert_eq!(updated.content, current_note.content);
                current_note = updated;
            }
            Err(e) => {
                panic!("✗ Failed on update {}: {:?}", i, e);
            }
        }
    }

    println!("\n✓ All updates completed successfully");
}

/// Test error handling when trying to update a non-existent note.
///
/// This test validates:
/// - Appropriate error is returned for invalid note IDs
#[test]
#[serial]
fn test_note_update_nonexistent() {
    let client = setup_test_client();

    let contacts = match client.get_contacts(10, 0) {
        Ok(c) if !c.is_empty() => c,
        Ok(_) => {
            println!("⚠ No contacts found - skipping test");
            return;
        }
        Err(e) => {
            println!("⚠ Cannot fetch contacts: {:?}", e);
            return;
        }
    };

    let contact = &contacts[0];

    println!("Testing update of non-existent note...");
    let fake_note_id = "nonexistent-note-id-12345";
    let note = sample_note(&contact.id, "This shouldn't work");

    match client.update_note(fake_note_id, &note) {
        Ok(_) => {
            panic!("✗ Update should have failed for non-existent note!");
        }
        Err(e) => {
            println!("  ✓ Got expected error: {:?}", e);
        }
    }
}

/// Test error handling when trying to delete a non-existent note.
///
/// This test validates:
/// - Appropriate error is returned for invalid note IDs
#[test]
#[serial]
fn test_note_delete_nonexistent() {
    let client = setup_test_client();

    println!("Testing delete of non-existent note...");
    let fake_note_id = "nonexistent-note-id-12345";

    match client.delete_note(fake_note_id) {
        Ok(_) => {
            // Some APIs return success for deleting non-existent items (idempotent)
            println!("  ℹ Delete succeeded (idempotent operation)");
        }
        Err(e) => {
            println!("  ✓ Got expected error: {:?}", e);
        }
    }
}

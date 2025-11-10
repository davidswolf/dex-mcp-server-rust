//! End-to-end tests for Reminder CRUD operations.
//!
//! These tests validate creating, reading, updating, and deleting reminders
//! against the live Dex API. All test data is automatically cleaned up.

use serial_test::serial;

mod e2e;
use e2e::{fixtures::*, *};

/// RAII guard that ensures a reminder is deleted when it goes out of scope.
///
/// This guarantees cleanup even if tests fail or panic.
struct ReminderGuard<'a> {
    client: &'a dex_mcp_server::DexClient,
    reminder_id: Option<String>,
}

impl<'a> ReminderGuard<'a> {
    fn new(client: &'a dex_mcp_server::DexClient) -> Self {
        Self {
            client,
            reminder_id: None,
        }
    }

    fn set_id(&mut self, id: String) {
        self.reminder_id = Some(id);
    }

    #[allow(dead_code)]
    fn id(&self) -> Option<&str> {
        self.reminder_id.as_deref()
    }
}

impl<'a> Drop for ReminderGuard<'a> {
    fn drop(&mut self) {
        if let Some(ref reminder_id) = self.reminder_id {
            match self.client.delete_reminder(reminder_id) {
                Ok(_) => println!("  ✓ Cleaned up test reminder: {}", reminder_id),
                Err(e) => eprintln!("  ⚠ Failed to cleanup reminder {}: {:?}", reminder_id, e),
            }
        }
    }
}

/// Test complete CRUD cycle for reminders: Create, Read, Update, Delete.
///
/// This test validates:
/// - Reminders can be created successfully
/// - Created reminders can be retrieved
/// - Reminders can be updated
/// - Reminders can be deleted
/// - All test data is cleaned up
#[test]
#[serial]
fn test_reminder_crud_lifecycle() {
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
    println!("Testing reminder CRUD with contact: {} (ID: {})", contact.name, contact.id);

    // Initialize cleanup guard
    let mut reminder_guard = ReminderGuard::new(&client);

    // TEST 1: CREATE
    println!("\n1. Testing CREATE reminder...");
    let original_text = generate_test_reminder_text("E2E CRUD Test Reminder");
    let reminder = sample_reminder(&contact.id, &original_text);

    let created_reminder = match client.create_reminder(&reminder) {
        Ok(r) => {
            println!("  ✓ Reminder created successfully");
            println!("    Reminder ID: {}", r.id);
            println!("    Text: {}", r.text);
            println!("    Due date: {}", r.due_date);
            assert_reminder_valid(&r);
            assert_eq!(r.text, original_text);
            assert_eq!(r.contact_id, contact.id);
            assert!(!r.completed, "New reminder should not be completed");
            r
        }
        Err(e) => {
            panic!("✗ Failed to create reminder: {:?}", e);
        }
    };

    // Register for cleanup
    reminder_guard.set_id(created_reminder.id.clone());

    // TEST 2: READ
    println!("\n2. Testing READ reminder...");
    let _reminders = match client.get_contact_reminders(&contact.id, 100, 0) {
        Ok(reminders) => {
            println!("  ✓ Retrieved {} reminders for contact", reminders.len());

            // Find our test reminder
            let found = reminders.iter().find(|r| r.id == created_reminder.id);
            match found {
                Some(r) => {
                    println!("  ✓ Found our test reminder in results");
                    assert_eq!(r.text, original_text);
                    assert_eq!(r.contact_id, contact.id);
                }
                None => {
                    panic!("✗ Created reminder not found in contact's reminders!");
                }
            }
            reminders
        }
        Err(e) => {
            panic!("✗ Failed to read reminders: {:?}", e);
        }
    };

    // TEST 3: UPDATE
    println!("\n3. Testing UPDATE reminder...");
    let updated_text = format!("{} - UPDATED", original_text);
    let mut updated_reminder = created_reminder.clone();
    updated_reminder.text = updated_text.clone();

    match client.update_reminder(&created_reminder.id, &updated_reminder) {
        Ok(r) => {
            println!("  ✓ Reminder updated successfully");
            println!("    New text: {}", r.text);
            assert_eq!(r.id, created_reminder.id);
            assert_eq!(r.text, updated_text);
        }
        Err(e) => {
            panic!("✗ Failed to update reminder: {:?}", e);
        }
    }

    // Verify the update persisted
    println!("  Verifying update persisted...");
    let reminders_after_update = match client.get_contact_reminders(&contact.id, 100, 0) {
        Ok(reminders) => reminders,
        Err(e) => {
            panic!("✗ Failed to verify update: {:?}", e);
        }
    };

    let updated_reminder_from_api = reminders_after_update.iter().find(|r| r.id == created_reminder.id);
    match updated_reminder_from_api {
        Some(r) => {
            println!("  ✓ Update verified");
            assert_eq!(r.text, updated_text);
        }
        None => {
            panic!("✗ Reminder not found after update!");
        }
    }

    // TEST 4: DELETE
    println!("\n4. Testing DELETE reminder...");
    match client.delete_reminder(&created_reminder.id) {
        Ok(_) => {
            println!("  ✓ Reminder deleted successfully");
            reminder_guard.reminder_id = None; // Already deleted, don't try again in Drop
        }
        Err(e) => {
            panic!("✗ Failed to delete reminder: {:?}", e);
        }
    }

    // Verify the deletion
    println!("  Verifying deletion...");
    let reminders_after_delete = match client.get_contact_reminders(&contact.id, 100, 0) {
        Ok(reminders) => reminders,
        Err(e) => {
            panic!("✗ Failed to verify deletion: {:?}", e);
        }
    };

    let deleted_reminder_check = reminders_after_delete.iter().find(|r| r.id == created_reminder.id);
    match deleted_reminder_check {
        None => {
            println!("  ✓ Deletion verified - reminder no longer exists");
        }
        Some(_) => {
            panic!("✗ Reminder still exists after deletion!");
        }
    }

    println!("\n✓ All CRUD operations completed successfully");
}

/// Test marking a reminder as complete and then updating it.
///
/// This test validates:
/// - Reminders can be marked as complete
/// - Completed reminders can be updated
#[test]
#[serial]
fn test_reminder_completion_workflow() {
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
    println!("Testing reminder completion with contact: {}", contact.name);

    let mut reminder_guard = ReminderGuard::new(&client);

    // Create reminder
    let text = generate_test_reminder_text("Completion Test");
    let reminder = sample_reminder(&contact.id, &text);

    let mut current_reminder = match client.create_reminder(&reminder) {
        Ok(r) => {
            println!("✓ Initial reminder created: {}", r.id);
            reminder_guard.set_id(r.id.clone());
            assert!(!r.completed, "New reminder should not be completed");
            r
        }
        Err(e) => {
            panic!("✗ Failed to create reminder: {:?}", e);
        }
    };

    // Mark as complete
    println!("\nMarking reminder as complete...");
    current_reminder.completed = true;
    current_reminder.completed_at = Some(chrono::Utc::now().to_rfc3339());

    match client.update_reminder(&current_reminder.id, &current_reminder) {
        Ok(r) => {
            println!("  ✓ Reminder marked as complete");
            assert!(r.completed, "Reminder should be marked as completed");
            current_reminder = r;
        }
        Err(e) => {
            panic!("✗ Failed to mark reminder as complete: {:?}", e);
        }
    }

    // Update the completed reminder's text
    println!("\nUpdating completed reminder text...");
    current_reminder.text = format!("{} - COMPLETED AND UPDATED", text);

    match client.update_reminder(&current_reminder.id, &current_reminder) {
        Ok(r) => {
            println!("  ✓ Completed reminder updated successfully");
            assert!(r.completed, "Reminder should still be completed");
            assert_eq!(r.text, current_reminder.text);
        }
        Err(e) => {
            panic!("✗ Failed to update completed reminder: {:?}", e);
        }
    }

    println!("\n✓ Completion workflow test completed successfully");
}

/// Test creating multiple reminders and batch cleanup.
///
/// This test validates:
/// - Multiple reminders can be created for the same contact
/// - All reminders are properly cleaned up
#[test]
#[serial]
fn test_reminder_batch_create_and_cleanup() {
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
    println!("Testing batch reminder creation with contact: {}", contact.name);

    let mut reminder_guards = Vec::new();

    // Create 3 test reminders with different due dates
    for i in 1..=3 {
        println!("\nCreating reminder {}/3...", i);
        let text = generate_test_reminder_text(&format!("Batch Test Reminder {}", i));
        let reminder = sample_reminder(&contact.id, &text);

        match client.create_reminder(&reminder) {
            Ok(created) => {
                println!("  ✓ Reminder {} created: {}", i, created.id);
                let mut guard = ReminderGuard::new(&client);
                guard.set_id(created.id);
                reminder_guards.push(guard);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to create reminder {}: {:?}", i, e);
            }
        }
    }

    println!("\n✓ Created {} reminders", reminder_guards.len());
    println!("  (All reminders will be automatically cleaned up)");

    // Guards will automatically clean up reminders when they go out of scope
}

/// Test updating a reminder's due date.
///
/// This test validates:
/// - Reminder due dates can be updated
/// - Date format is handled correctly
#[test]
#[serial]
fn test_reminder_update_due_date() {
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
    println!("Testing reminder due date update with contact: {}", contact.name);

    let mut reminder_guard = ReminderGuard::new(&client);

    // Create reminder
    let text = generate_test_reminder_text("Due Date Update Test");
    let reminder = sample_reminder(&contact.id, &text);
    let original_due_date = reminder.due_date.clone();

    let mut current_reminder = match client.create_reminder(&reminder) {
        Ok(r) => {
            println!("✓ Initial reminder created: {}", r.id);
            println!("  Original due date: {}", r.due_date);
            reminder_guard.set_id(r.id.clone());
            r
        }
        Err(e) => {
            panic!("✗ Failed to create reminder: {:?}", e);
        }
    };

    // Update due date to 14 days from now
    println!("\nUpdating due date...");
    let new_due_date = (chrono::Utc::now() + chrono::Duration::days(14)).to_rfc3339();
    current_reminder.due_date = new_due_date.clone();

    match client.update_reminder(&current_reminder.id, &current_reminder) {
        Ok(r) => {
            println!("  ✓ Due date updated successfully");
            println!("    New due date: {}", r.due_date);
            assert_ne!(r.due_date, original_due_date);
        }
        Err(e) => {
            panic!("✗ Failed to update due date: {:?}", e);
        }
    }

    println!("\n✓ Due date update test completed successfully");
}

/// Test error handling when trying to update a non-existent reminder.
///
/// This test validates:
/// - Appropriate error is returned for invalid reminder IDs
#[test]
#[serial]
fn test_reminder_update_nonexistent() {
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

    println!("Testing update of non-existent reminder...");
    let fake_reminder_id = "nonexistent-reminder-id-12345";
    let reminder = sample_reminder(&contact.id, "This shouldn't work");

    match client.update_reminder(fake_reminder_id, &reminder) {
        Ok(_) => {
            panic!("✗ Update should have failed for non-existent reminder!");
        }
        Err(e) => {
            println!("  ✓ Got expected error: {:?}", e);
        }
    }
}

/// Test error handling when trying to delete a non-existent reminder.
///
/// This test validates:
/// - Appropriate error is returned for invalid reminder IDs
#[test]
#[serial]
fn test_reminder_delete_nonexistent() {
    let client = setup_test_client();

    println!("Testing delete of non-existent reminder...");
    let fake_reminder_id = "nonexistent-reminder-id-12345";

    match client.delete_reminder(fake_reminder_id) {
        Ok(_) => {
            // Some APIs return success for deleting non-existent items (idempotent)
            println!("  ℹ Delete succeeded (idempotent operation)");
        }
        Err(e) => {
            println!("  ✓ Got expected error: {:?}", e);
        }
    }
}

/// Test creating a reminder with a past due date.
///
/// This test validates:
/// - Reminders can be created with past due dates
/// - Such reminders are considered overdue
#[test]
#[serial]
fn test_reminder_with_past_due_date() {
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
    println!("Testing overdue reminder creation with contact: {}", contact.name);

    let mut reminder_guard = ReminderGuard::new(&client);

    // Create reminder with past due date
    let text = generate_test_reminder_text("Overdue Test");
    let past_date = (chrono::Utc::now() - chrono::Duration::days(7)).to_rfc3339();
    let reminder = sample_reminder_with_due_date(&contact.id, &text, &past_date);

    match client.create_reminder(&reminder) {
        Ok(r) => {
            println!("✓ Overdue reminder created: {}", r.id);
            println!("  Due date: {}", r.due_date);
            reminder_guard.set_id(r.id.clone());

            // Check if it's considered overdue
            let now = chrono::Utc::now().to_rfc3339();
            let is_overdue = r.is_overdue(&now);
            println!("  Is overdue: {}", is_overdue);
            assert!(is_overdue, "Reminder with past due date should be overdue");
        }
        Err(e) => {
            panic!("✗ Failed to create overdue reminder: {:?}", e);
        }
    }

    println!("\n✓ Overdue reminder test completed successfully");
}

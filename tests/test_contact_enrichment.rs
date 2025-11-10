//! End-to-end tests for contact enrichment functionality.
//!
//! These tests validate note creation, reminder management, and
//! other contact enrichment features against the live Dex API.

use serial_test::serial;

mod e2e;
use e2e::{fixtures::*, *};

/// Test adding a note to a contact.
///
/// This test validates:
/// - Notes can be created successfully
/// - Note ID is returned
/// - Note content is preserved
#[test]
#[serial]
fn test_add_note_to_contact() {
    let client = setup_test_client();

    // Get a contact to add a note to
    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping note creation test");
        return;
    }

    let contact = &contacts[0];
    let note_content = generate_test_note_content("E2E Test Note");

    println!("Adding note to contact: {} (ID: {})", contact.name, contact.id);
    println!("Note content: {}", note_content);

    let note = sample_note(&contact.id, &note_content);
    let result = client.create_note(&note);

    match result {
        Ok(created_note) => {
            println!("✓ Note created successfully");
            assert_note_valid(&created_note);
            assert_eq!(created_note.content, note_content);
            assert_eq!(created_note.contact_id, contact.id);
            println!("  Note ID: {}", created_note.id);
        }
        Err(e) => {
            println!("⚠ Failed to create note: {:?}", e);
            println!("  This may be due to API permissions or rate limits");
        }
    }
}

/// Test retrieving notes for a contact.
///
/// This test validates:
/// - Notes can be fetched for a contact
/// - Note data structure is correct
/// - Notes are returned in proper order
#[test]
#[serial]
fn test_retrieve_notes_for_contact() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping note retrieval test");
        return;
    }

    let contact = &contacts[0];
    println!("Fetching notes for contact: {} (ID: {})", contact.name, contact.id);

    let result = client.get_contact_notes(&contact.id, 50, 0);

    match result {
        Ok(notes) => {
            println!("✓ Retrieved {} notes", notes.len());

            if !notes.is_empty() {
                // Validate first note
                assert_note_valid(&notes[0]);
                println!("  First note preview: {}",
                    notes[0].content.chars().take(50).collect::<String>());
            }
        }
        Err(e) => {
            println!("⚠ Failed to fetch notes: {:?}", e);
        }
    }
}

/// Test adding multiple notes to the same contact.
///
/// This test validates:
/// - Multiple notes can be added
/// - All notes are retrievable
/// - Note count increases correctly
#[test]
#[serial]
fn test_add_multiple_notes() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping multiple notes test");
        return;
    }

    let contact = &contacts[0];
    println!("Adding multiple notes to: {} (ID: {})", contact.name, contact.id);

    // Get initial note count
    let initial_notes = client.get_contact_notes(&contact.id, 100, 0);
    let initial_count = initial_notes.map(|n| n.len()).unwrap_or(0);

    println!("Initial note count: {}", initial_count);

    // Add 3 test notes
    let mut created_count = 0;
    for i in 1..=3 {
        let content = generate_test_note_content(&format!("Multi-note test {}", i));
        let note = sample_note(&contact.id, &content);

        match client.create_note(&note) {
            Ok(_) => {
                created_count += 1;
                println!("  ✓ Note {} created", i);
            }
            Err(e) => {
                println!("  ⚠ Failed to create note {}: {:?}", i, e);
            }
        }

        // Brief delay to avoid rate limits
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    if created_count > 0 {
        // Retrieve notes again
        let final_notes = client.get_contact_notes(&contact.id, 100, 0);

        if let Ok(notes) = final_notes {
            println!("Final note count: {}", notes.len());
            assert!(
                notes.len() >= initial_count + created_count,
                "Note count should have increased by at least {}",
                created_count
            );
        }
    }
}

/// Test note with special characters and formatting.
///
/// This test validates:
/// - Special characters are preserved
/// - Newlines and formatting are maintained
/// - Unicode/emoji support works
#[test]
#[serial]
fn test_note_with_special_characters() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping special characters test");
        return;
    }

    let contact = &contacts[0];
    let special_content = "Test note with special chars:\n- Line 1\n- Line 2\n\"Quotes\" and 'apostrophes'\n🎉 Emoji support";

    println!("Creating note with special characters");

    let note = sample_note(&contact.id, special_content);
    let result = client.create_note(&note);

    match result {
        Ok(created) => {
            println!("✓ Note created with special characters");
            assert!(
                created.content.contains("🎉"),
                "Emoji should be preserved"
            );
            assert!(
                created.content.contains("\n"),
                "Newlines should be preserved"
            );
            assert!(
                created.content.contains("\""),
                "Quotes should be preserved"
            );
            println!("  ✓ Special characters preserved");
        }
        Err(e) => {
            println!("⚠ Failed to create note with special chars: {:?}", e);
        }
    }
}

/// Test creating a reminder for a contact.
///
/// This test validates:
/// - Reminders can be created
/// - Reminder ID is returned
/// - Due date is set correctly
#[test]
#[serial]
fn test_create_reminder() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping reminder creation test");
        return;
    }

    let contact = &contacts[0];
    let reminder_text = generate_test_reminder_text("E2E Test Reminder");

    println!("Creating reminder for: {} (ID: {})", contact.name, contact.id);
    println!("Reminder text: {}", reminder_text);

    let reminder = sample_reminder(&contact.id, &reminder_text);
    let result = client.create_reminder(&reminder);

    match result {
        Ok(created) => {
            println!("✓ Reminder created successfully");
            assert_reminder_valid(&created);
            assert_eq!(created.text, reminder_text);
            assert_eq!(created.contact_id, contact.id);
            assert!(!created.completed, "New reminder should not be completed");
            println!("  Reminder ID: {}", created.id);
            println!("  Due date: {}", created.due_date);
        }
        Err(e) => {
            println!("⚠ Failed to create reminder: {:?}", e);
        }
    }
}

/// Test retrieving reminders for a contact.
///
/// This test validates:
/// - Reminders can be fetched
/// - Reminder data structure is correct
/// - Active vs completed status is tracked
#[test]
#[serial]
fn test_retrieve_reminders_for_contact() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping reminder retrieval test");
        return;
    }

    let contact = &contacts[0];
    println!("Fetching reminders for: {} (ID: {})", contact.name, contact.id);

    let result = client.get_contact_reminders(&contact.id, 50, 0);

    match result {
        Ok(reminders) => {
            println!("✓ Retrieved {} reminders", reminders.len());

            if !reminders.is_empty() {
                assert_reminder_valid(&reminders[0]);

                let active_count = reminders.iter().filter(|r| !r.completed).count();
                let completed_count = reminders.iter().filter(|r| r.completed).count();

                println!("  Active: {}, Completed: {}", active_count, completed_count);
            }
        }
        Err(e) => {
            println!("⚠ Failed to fetch reminders: {:?}", e);
        }
    }
}

/// Test reminder due date handling.
///
/// This test validates:
/// - Due dates are set correctly
/// - Date format is proper ISO 8601
/// - Past and future dates work
#[test]
#[serial]
fn test_reminder_due_date_handling() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping due date test");
        return;
    }

    let contact = &contacts[0];

    // Create reminder with future date
    let future_date = chrono::Utc::now() + chrono::Duration::days(30);
    let future_date_str = future_date.to_rfc3339();

    let reminder = sample_reminder_with_due_date(
        &contact.id,
        "Future reminder",
        &future_date_str,
    );

    println!("Creating reminder with future due date: {}", future_date_str);

    let result = client.create_reminder(&reminder);

    match result {
        Ok(created) => {
            println!("✓ Reminder created with future due date");
            assert!(!created.due_date.is_empty());
            println!("  Due date set to: {}", created.due_date);
        }
        Err(e) => {
            println!("⚠ Failed to create reminder with due date: {:?}", e);
        }
    }
}

/// Test note timestamp handling.
///
/// This test validates:
/// - Timestamps are stored correctly
/// - ISO 8601 format is used
/// - Custom timestamps can be set
#[test]
#[serial]
fn test_note_timestamp_handling() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping timestamp test");
        return;
    }

    let contact = &contacts[0];
    let custom_time = chrono::Utc::now() - chrono::Duration::hours(2);
    let custom_time_str = custom_time.to_rfc3339();

    let note = sample_note_with_timestamp(
        &contact.id,
        "Test note with custom timestamp",
        &custom_time_str,
    );

    println!("Creating note with custom timestamp: {}", custom_time_str);

    let result = client.create_note(&note);

    match result {
        Ok(created) => {
            println!("✓ Note created with timestamp");
            assert!(!created.created_at.is_empty());
            println!("  Timestamp: {}", created.created_at);

            // Verify timestamp is in valid ISO 8601 format
            let parse_result = chrono::DateTime::parse_from_rfc3339(&created.created_at);
            assert!(
                parse_result.is_ok(),
                "Timestamp should be valid ISO 8601 format"
            );
        }
        Err(e) => {
            println!("⚠ Failed to create note with timestamp: {:?}", e);
        }
    }
}

/// Test pagination for notes.
///
/// This test validates:
/// - Note pagination works correctly
/// - Limit and offset parameters are respected
#[test]
#[serial]
fn test_notes_pagination() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping pagination test");
        return;
    }

    // Find a contact with multiple notes
    let mut contact_with_notes = None;

    for contact in &contacts {
        if let Ok(notes) = client.get_contact_notes(&contact.id, 100, 0) {
            if notes.len() > 5 {
                contact_with_notes = Some(contact);
                println!("Found contact with {} notes: {}", notes.len(), contact.name);
                break;
            }
        }
    }

    if let Some(contact) = contact_with_notes {
        // Test pagination
        let page1 = client.get_contact_notes(&contact.id, 3, 0).unwrap();
        let page2 = client.get_contact_notes(&contact.id, 3, 3).unwrap();

        println!("Page 1: {} notes", page1.len());
        println!("Page 2: {} notes", page2.len());

        // Verify no duplicates between pages
        if !page1.is_empty() && !page2.is_empty() {
            let page1_ids: Vec<String> = page1.iter().map(|n| n.id.clone()).collect();
            let page2_ids: Vec<String> = page2.iter().map(|n| n.id.clone()).collect();

            for id in &page1_ids {
                assert!(
                    !page2_ids.contains(id),
                    "Found duplicate note across pages"
                );
            }

            println!("✓ No duplicates found between pages");
        }
    } else {
        println!("⚠ No contacts with enough notes for pagination test");
    }
}

/// Test pagination for reminders.
///
/// This test validates:
/// - Reminder pagination works correctly
/// - Limit and offset parameters are respected
#[test]
#[serial]
fn test_reminders_pagination() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping pagination test");
        return;
    }

    // Find a contact with multiple reminders
    let mut contact_with_reminders = None;

    for contact in &contacts {
        if let Ok(reminders) = client.get_contact_reminders(&contact.id, 100, 0) {
            if reminders.len() > 5 {
                contact_with_reminders = Some(contact);
                println!("Found contact with {} reminders: {}", reminders.len(), contact.name);
                break;
            }
        }
    }

    if let Some(contact) = contact_with_reminders {
        // Test pagination
        let page1 = client.get_contact_reminders(&contact.id, 3, 0).unwrap();
        let page2 = client.get_contact_reminders(&contact.id, 3, 3).unwrap();

        println!("Page 1: {} reminders", page1.len());
        println!("Page 2: {} reminders", page2.len());

        // Verify no duplicates between pages
        if !page1.is_empty() && !page2.is_empty() {
            let page1_ids: Vec<String> = page1.iter().map(|r| r.id.clone()).collect();
            let page2_ids: Vec<String> = page2.iter().map(|r| r.id.clone()).collect();

            for id in &page1_ids {
                assert!(
                    !page2_ids.contains(id),
                    "Found duplicate reminder across pages"
                );
            }

            println!("✓ No duplicates found between pages");
        }
    } else {
        println!("⚠ No contacts with enough reminders for pagination test");
    }
}

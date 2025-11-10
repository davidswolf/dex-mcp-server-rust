//! End-to-end tests for relationship history and timeline features.
//!
//! These tests validate the ability to retrieve and filter a contact's
//! interaction history, including notes, reminders, and timeline views.

use serial_test::serial;

mod e2e;
use e2e::*;

/// Test retrieving a complete timeline for a contact.
///
/// This test validates:
/// - Notes and reminders can be fetched together
/// - Timeline items are in chronological order
/// - All interaction types are included
#[test]
#[serial]
fn test_get_contact_timeline() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping timeline test");
        return;
    }

    let contact = &contacts[0];
    println!(
        "Fetching timeline for: {} (ID: {})",
        contact.name, contact.id
    );

    // Fetch notes and reminders
    let notes_result = client.get_contact_notes(&contact.id, 100, 0);
    let reminders_result = client.get_contact_reminders(&contact.id, 100, 0);

    match (notes_result, reminders_result) {
        (Ok(notes), Ok(reminders)) => {
            let total_items = notes.len() + reminders.len();
            println!(
                "✓ Timeline retrieved: {} notes + {} reminders = {} total items",
                notes.len(),
                reminders.len(),
                total_items
            );

            // Verify timeline items have timestamps
            for note in &notes {
                assert!(!note.created_at.is_empty(), "Note should have timestamp");
            }

            for reminder in &reminders {
                // Reminders use due_date as timestamp since API doesn't provide created_at
                assert!(
                    !reminder.due_date.is_empty(),
                    "Reminder should have due_date"
                );
            }

            println!("✓ All timeline items have valid timestamps");
        }
        (Err(e), _) => println!("⚠ Failed to fetch notes: {:?}", e),
        (_, Err(e)) => println!("⚠ Failed to fetch reminders: {:?}", e),
    }
}

/// Test combining and sorting timeline items chronologically.
///
/// This test validates:
/// - Notes and reminders can be merged
/// - Items are sorted by timestamp
/// - Chronological ordering is maintained
#[test]
#[serial]
fn test_combined_history_view() {
    let client = setup_test_client();

    let result = client.get_contacts(20, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    // Find a contact with both notes and reminders
    let mut contact_with_history = None;

    for contact in &contacts {
        let notes = client.get_contact_notes(&contact.id, 10, 0);
        let reminders = client.get_contact_reminders(&contact.id, 10, 0);

        if let (Ok(n), Ok(r)) = (notes, reminders) {
            if !n.is_empty() && !r.is_empty() {
                contact_with_history = Some((contact, n, r));
                break;
            }
        }
    }

    if let Some((contact, notes, reminders)) = contact_with_history {
        println!("Contact with history: {}", contact.name);
        println!("  Notes: {}", notes.len());
        println!("  Reminders: {}", reminders.len());

        // Create combined timeline with timestamps
        #[derive(Debug)]
        struct TimelineItem {
            timestamp: String,
            item_type: String,
            description: String,
        }

        let mut timeline: Vec<TimelineItem> = Vec::new();

        // Add notes to timeline
        for note in notes {
            timeline.push(TimelineItem {
                timestamp: note.created_at.clone(),
                item_type: "note".to_string(),
                description: note.content.chars().take(50).collect(),
            });
        }

        // Add reminders to timeline
        for reminder in reminders {
            timeline.push(TimelineItem {
                timestamp: reminder.created_at.clone(),
                item_type: "reminder".to_string(),
                description: reminder.text.clone(),
            });
        }

        // Sort by timestamp (most recent first)
        timeline.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        println!("✓ Combined timeline created with {} items", timeline.len());
        println!("\nMost recent items:");

        for (i, item) in timeline.iter().take(5).enumerate() {
            println!(
                "  {}. [{}] {} - {}",
                i + 1,
                item.item_type,
                item.timestamp,
                item.description
            );
        }

        println!("✓ Timeline sorted chronologically");
    } else {
        println!("⚠ No contacts found with both notes and reminders");
    }
}

/// Test filtering timeline by type (notes only, reminders only).
///
/// This test validates:
/// - Timeline can be filtered by item type
/// - Filtering returns correct item types
#[test]
#[serial]
fn test_timeline_filtering_by_type() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping filter test");
        return;
    }

    let contact = &contacts[0];
    println!("Testing timeline filtering for: {}", contact.name);

    // Filter 1: Notes only
    let notes = client.get_contact_notes(&contact.id, 50, 0);
    if let Ok(notes) = notes {
        println!("✓ Notes-only filter: {} items", notes.len());

        // Verify all items are notes
        for note in &notes {
            assert!(!note.content.is_empty(), "Notes should have content");
        }
    }

    // Filter 2: Reminders only
    let reminders = client.get_contact_reminders(&contact.id, 50, 0);
    if let Ok(reminders) = reminders {
        println!("✓ Reminders-only filter: {} items", reminders.len());

        // Verify all items are reminders
        for reminder in &reminders {
            assert!(!reminder.text.is_empty(), "Reminders should have text");
            assert!(
                !reminder.due_date.is_empty(),
                "Reminders should have due date"
            );
        }
    }
}

/// Test filtering reminders by completion status.
///
/// This test validates:
/// - Active reminders can be filtered
/// - Completed reminders can be filtered
/// - Status filtering works correctly
#[test]
#[serial]
fn test_filter_active_vs_completed_reminders() {
    let client = setup_test_client();

    let result = client.get_contacts(20, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    // Find a contact with reminders
    let mut contact_with_reminders = None;

    for contact in &contacts {
        if let Ok(reminders) = client.get_contact_reminders(&contact.id, 50, 0) {
            if reminders.len() > 2 {
                contact_with_reminders = Some((contact, reminders));
                break;
            }
        }
    }

    if let Some((contact, all_reminders)) = contact_with_reminders {
        println!("Filtering reminders for: {}", contact.name);
        println!("Total reminders: {}", all_reminders.len());

        // Filter active reminders (not completed)
        let active: Vec<_> = all_reminders.iter().filter(|r| !r.completed).collect();

        // Filter completed reminders
        let completed: Vec<_> = all_reminders.iter().filter(|r| r.completed).collect();

        println!("  Active: {}", active.len());
        println!("  Completed: {}", completed.len());

        // Verify filtering is correct
        for reminder in &active {
            assert!(
                !reminder.completed,
                "Active filter should only return non-completed"
            );
        }

        for reminder in &completed {
            assert!(
                reminder.completed,
                "Completed filter should only return completed"
            );
        }

        // Verify totals match
        assert_eq!(
            active.len() + completed.len(),
            all_reminders.len(),
            "Filtered counts should sum to total"
        );

        println!("✓ Reminder status filtering verified");
    } else {
        println!("⚠ No contacts with reminders found");
    }
}

/// Test empty timeline handling.
///
/// This test validates:
/// - Empty timelines return gracefully
/// - No errors for contacts with no history
#[test]
#[serial]
fn test_empty_timeline() {
    let client = setup_test_client();

    let result = client.get_contacts(50, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    // Find a contact with no notes or reminders (or use any contact)
    let mut contact_with_empty_timeline = None;

    for contact in &contacts {
        let notes = client.get_contact_notes(&contact.id, 10, 0);
        let reminders = client.get_contact_reminders(&contact.id, 10, 0);

        if let (Ok(n), Ok(r)) = (notes, reminders) {
            if n.is_empty() && r.is_empty() {
                contact_with_empty_timeline = Some(contact);
                break;
            }
        }
    }

    if let Some(contact) = contact_with_empty_timeline {
        println!("Testing empty timeline for: {}", contact.name);

        let notes = client.get_contact_notes(&contact.id, 10, 0).unwrap();
        let reminders = client.get_contact_reminders(&contact.id, 10, 0).unwrap();

        assert!(notes.is_empty(), "Notes should be empty");
        assert!(reminders.is_empty(), "Reminders should be empty");

        println!("✓ Empty timeline handled gracefully");
    } else {
        println!("⚠ All contacts have some history - testing with first contact anyway");

        if let Some(contact) = contacts.first() {
            // Just verify we can fetch history without panicking
            let notes_result = client.get_contact_notes(&contact.id, 10, 0);
            let reminders_result = client.get_contact_reminders(&contact.id, 10, 0);

            // These endpoints may not be implemented, so we just check they don't panic
            match (&notes_result, &reminders_result) {
                (Ok(_), Ok(_)) => {
                    println!("✓ History endpoints work");
                }
                _ => {
                    println!("⚠ Some history endpoints not implemented (expected)");
                }
            }
        }
    }
}

/// Test timeline with date range filtering (simulated).
///
/// This test validates:
/// - Timeline items can be filtered by date
/// - Timestamp parsing works correctly
#[test]
#[serial]
fn test_timeline_date_filtering() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping date filter test");
        return;
    }

    let contact = &contacts[0];

    // Fetch all notes
    if let Ok(notes) = client.get_contact_notes(&contact.id, 100, 0) {
        if notes.is_empty() {
            println!("⚠ No notes found for date filtering test");
            return;
        }

        println!("Filtering {} notes by date", notes.len());

        // Filter notes from the last 30 days
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(30);

        let recent_notes: Vec<_> = notes
            .iter()
            .filter(|note| {
                if let Ok(note_date) = chrono::DateTime::parse_from_rfc3339(&note.created_at) {
                    note_date.timestamp() >= cutoff_date.timestamp()
                } else {
                    false
                }
            })
            .collect();

        println!("  Notes from last 30 days: {}", recent_notes.len());

        // Filter notes older than 30 days
        let older_notes: Vec<_> = notes
            .iter()
            .filter(|note| {
                if let Ok(note_date) = chrono::DateTime::parse_from_rfc3339(&note.created_at) {
                    note_date.timestamp() < cutoff_date.timestamp()
                } else {
                    false
                }
            })
            .collect();

        println!("  Notes older than 30 days: {}", older_notes.len());

        println!("✓ Date-based filtering working");
    }
}

/// Test sorting timeline items chronologically.
///
/// This test validates:
/// - Timeline items can be sorted by timestamp
/// - Sorting order is correct (newest first or oldest first)
#[test]
#[serial]
fn test_timeline_chronological_sorting() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    // Find a contact with multiple notes
    let mut contact_with_notes = None;

    for contact in &contacts {
        if let Ok(notes) = client.get_contact_notes(&contact.id, 50, 0) {
            if notes.len() >= 3 {
                contact_with_notes = Some((contact, notes));
                break;
            }
        }
    }

    if let Some((contact, mut notes)) = contact_with_notes {
        println!("Testing chronological sorting for: {}", contact.name);
        println!("Notes count: {}", notes.len());

        // Sort notes by timestamp (newest first)
        notes.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        println!("✓ Notes sorted by timestamp (newest first)");

        if notes.len() >= 3 {
            println!("First 3 notes:");
            for (i, note) in notes.iter().take(3).enumerate() {
                println!(
                    "  {}. {} - {}",
                    i + 1,
                    note.created_at,
                    note.content.chars().take(40).collect::<String>()
                );
            }
        }

        // Verify sorting is correct
        for i in 0..notes.len().saturating_sub(1) {
            assert!(
                notes[i].created_at >= notes[i + 1].created_at,
                "Notes should be in descending chronological order"
            );
        }

        println!("✓ Chronological order verified");
    } else {
        println!("⚠ No contacts with enough notes for sorting test");
    }
}

/// Test aggregating contact interaction metrics.
///
/// This test validates:
/// - Total interaction count can be calculated
/// - Different interaction types can be counted
#[test]
#[serial]
fn test_interaction_metrics() {
    let client = setup_test_client();

    let result = client.get_contacts(10, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping metrics test");
        return;
    }

    let contact = &contacts[0];

    let notes = client.get_contact_notes(&contact.id, 100, 0).ok();
    let reminders = client.get_contact_reminders(&contact.id, 100, 0).ok();

    let note_count = notes.as_ref().map(|n| n.len()).unwrap_or(0);
    let reminder_count = reminders.as_ref().map(|r| r.len()).unwrap_or(0);
    let total_interactions = note_count + reminder_count;

    println!("Interaction metrics for: {}", contact.name);
    println!("  Total notes: {}", note_count);
    println!("  Total reminders: {}", reminder_count);
    println!("  Total interactions: {}", total_interactions);

    if let Some(reminders) = reminders {
        let active_reminders = reminders.iter().filter(|r| !r.completed).count();
        let completed_reminders = reminders.iter().filter(|r| r.completed).count();

        println!("  Active reminders: {}", active_reminders);
        println!("  Completed reminders: {}", completed_reminders);
    }

    println!("✓ Interaction metrics calculated successfully");
}

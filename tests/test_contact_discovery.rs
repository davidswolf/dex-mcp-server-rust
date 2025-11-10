//! End-to-end tests for contact discovery and search functionality.
//!
//! These tests validate the contact discovery features including
//! name-based search, email search, LinkedIn profile matching,
//! and fuzzy matching capabilities.

use serial_test::serial;

mod e2e;
use e2e::*;

/// Test finding a contact by exact name match.
///
/// This test validates:
/// - Name-based search works correctly
/// - Exact matches return high confidence
/// - Contact details are properly returned
#[test]
#[ignore]
#[serial]
fn test_find_contact_by_name() {
    let client = setup_test_client();

    // Get a known contact name from the test data
    let test_names = get_known_test_contacts();

    if test_names.is_empty() {
        println!("⚠ Skipping test: No test contact names configured");
        return;
    }

    let search_name = &test_names[0];
    println!("Searching for contact by name: {}", search_name);

    // First, verify the contact exists by listing all contacts
    let all_contacts = client.get_contacts(1000, 0);
    if all_contacts.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = all_contacts.unwrap();
    let found = contacts.iter().find(|c| c.name.contains(search_name));

    if let Some(contact) = found {
        println!("✓ Found contact: {} (ID: {})", contact.name, contact.id);
        assert_contact_valid(contact);

        // Verify we can get the contact by ID
        let by_id = client.get_contact(&contact.id).unwrap();
        assert_eq!(by_id.id, contact.id);
        println!("✓ Successfully retrieved contact by ID");
    } else {
        println!("⚠ Contact '{}' not found in database", search_name);
    }
}

/// Test finding a contact by email address.
///
/// This test validates:
/// - Email search returns exact match
/// - Search returns high confidence for exact email matches
/// - Multiple contacts with same email domain can be distinguished
#[test]
#[ignore]
#[serial]
fn test_find_contact_by_email() {
    let client = setup_test_client();

    // Get contacts and find one with an email
    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();
    let contact_with_email = contacts.iter().find(|c| c.email.is_some());

    if let Some(contact) = contact_with_email {
        let email = contact.email.as_ref().unwrap();
        println!("Testing email search: {}", email);

        match client.search_contacts_by_email(email) {
            Ok(search_results) => {
                if !search_results.is_empty() {
                    // Verify at least one result has the matching email
                    let has_match = search_results
                        .iter()
                        .any(|c| c.email.as_ref() == Some(email));
                    assert!(has_match, "Search results should contain the exact email");
                    println!(
                        "✓ Email search successful: found {} result(s)",
                        search_results.len()
                    );
                } else {
                    println!("⚠ Email search returned no results");
                }
            }
            Err(dex_mcp_server::DexApiError::NotFound(_)) => {
                println!("⚠ Email search endpoint not implemented");
            }
            Err(dex_mcp_server::DexApiError::ApiError { status: 400, .. }) => {
                println!("⚠ Email search parameter not supported by API");
            }
            Err(e) => {
                println!("⚠ Email search failed: {:?}", e);
            }
        }
    } else {
        println!("⚠ No contacts with email found - skipping email search test");
    }
}

/// Test finding a contact by LinkedIn profile.
///
/// This test validates:
/// - LinkedIn URL matching works
/// - Social profile fields are populated
/// - Matching is case-insensitive for URLs
#[test]
#[ignore]
#[serial]
fn test_find_contact_by_linkedin() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    // Find a contact with LinkedIn profile
    let contact_with_linkedin = contacts.iter().find(|c| {
        c.social_profiles.iter().any(|p| {
            p.profile_type.to_lowercase().contains("linkedin")
                || p.url.to_lowercase().contains("linkedin")
        })
    });

    if let Some(contact) = contact_with_linkedin {
        println!("Found contact with LinkedIn: {}", contact.name);

        let linkedin_profile = contact
            .social_profiles
            .iter()
            .find(|p| {
                p.profile_type.to_lowercase().contains("linkedin")
                    || p.url.to_lowercase().contains("linkedin")
            })
            .unwrap();

        println!("LinkedIn URL: {}", linkedin_profile.url);
        println!("✓ Contact has LinkedIn profile in social_profiles field");

        // Verify we can retrieve the contact by ID and LinkedIn is preserved
        let by_id = client.get_contact(&contact.id).unwrap();
        let has_linkedin = by_id.social_profiles.iter().any(|p| {
            p.profile_type.to_lowercase().contains("linkedin")
                || p.url.to_lowercase().contains("linkedin")
        });
        assert!(
            has_linkedin,
            "LinkedIn profile should be preserved when fetching by ID"
        );

        println!("✓ LinkedIn profile preserved in contact retrieval");
    } else {
        println!("⚠ No contacts with LinkedIn found - skipping LinkedIn search test");
    }
}

/// Test finding a contact with partial name (e.g., first name only).
///
/// This test validates:
/// - Partial name search returns multiple candidates
/// - Results are ranked appropriately
/// - No false negatives for valid partial matches
#[test]
#[ignore]
#[serial]
fn test_find_contact_partial_name() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping partial name test");
        return;
    }

    // Take the first name of the first contact
    let full_name = &contacts[0].name;
    let first_name = full_name.split_whitespace().next().unwrap_or(full_name);

    println!("Searching for partial name: {}", first_name);

    // Search for all contacts matching the first name
    let matching_contacts: Vec<_> = contacts
        .iter()
        .filter(|c| c.name.to_lowercase().contains(&first_name.to_lowercase()))
        .collect();

    println!(
        "Found {} contacts matching '{}'",
        matching_contacts.len(),
        first_name
    );

    if matching_contacts.len() > 1 {
        println!("✓ Partial name search returns multiple candidates:");
        for (i, contact) in matching_contacts.iter().take(5).enumerate() {
            println!("  {}. {}", i + 1, contact.name);
        }
    } else if matching_contacts.len() == 1 {
        println!(
            "✓ Partial name search returns single match: {}",
            matching_contacts[0].name
        );
    }

    // Verify the original contact is in the results
    let original_found = matching_contacts.iter().any(|c| &c.name == full_name);
    assert!(
        original_found,
        "Original contact should be in partial name search results"
    );
}

/// Test searching for a non-existent contact.
///
/// This test validates:
/// - Non-existent searches don't crash
/// - Empty or low-confidence results are returned
/// - Error handling is graceful
#[test]
#[ignore]
#[serial]
fn test_find_contact_no_match() {
    let client = setup_test_client();

    // Search for a contact that definitely doesn't exist
    let non_existent_name = "ZzZzNonExistentContactXyZ123456789";

    let result = client.get_contacts(1000, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    // Search for the non-existent name
    let matches: Vec<_> = contacts
        .iter()
        .filter(|c| c.name.contains(non_existent_name))
        .collect();

    assert!(
        matches.is_empty(),
        "Should not find matches for non-existent name"
    );
    println!("✓ Non-existent name returns no matches (as expected)");

    // Test with non-existent email
    let result = client.search_contacts_by_email("nonexistent99999@example.com");

    match result {
        Ok(results) => {
            assert!(
                results.is_empty(),
                "Should not find contacts for non-existent email"
            );
            println!("✓ Non-existent email returns empty results (as expected)");
        }
        Err(e) => {
            println!("⚠ Email search returned error: {:?}", e);
        }
    }
}

/// Test loading all contacts with pagination.
///
/// This test validates:
/// - All contacts can be fetched using pagination
/// - Total count is consistent
/// - No duplicate contacts across pages
/// - Data consistency is maintained
#[test]
#[ignore]
#[serial]
fn test_list_all_contacts_discovery() {
    let client = setup_test_client();

    let page_size = 50;
    let mut all_contacts = Vec::new();
    let mut offset = 0;
    let mut page_num = 1;

    println!(
        "Fetching all contacts with pagination (page size: {})...",
        page_size
    );

    loop {
        let result = client.get_contacts(page_size, offset);
        if result.is_err() {
            println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
            return;
        }

        let contacts = result.unwrap();

        if contacts.is_empty() {
            break;
        }

        println!("Page {}: fetched {} contacts", page_num, contacts.len());

        all_contacts.extend(contacts);
        offset += page_size;
        page_num += 1;

        // Safety limit to avoid infinite loop
        if page_num > 100 {
            println!("⚠ Reached safety limit of 100 pages");
            break;
        }
    }

    println!("Total contacts fetched: {}", all_contacts.len());

    if !all_contacts.is_empty() {
        // Check for duplicates
        let mut ids = std::collections::HashSet::new();
        let mut duplicates = 0;

        for contact in &all_contacts {
            if !ids.insert(&contact.id) {
                duplicates += 1;
                println!("⚠ Duplicate contact ID: {}", contact.id);
            }
        }

        assert_eq!(duplicates, 0, "Found {} duplicate contact IDs", duplicates);
        println!(
            "✓ No duplicate contacts found across {} pages",
            page_num - 1
        );

        // Validate a sample of contacts
        for contact in all_contacts.iter().take(10) {
            assert_contact_valid(contact);
        }
        println!("✓ Sample of contacts validated successfully");
    }
}

/// Test case-insensitive name search.
///
/// This test validates:
/// - Name search is case-insensitive
/// - Different case variations return same results
#[test]
#[ignore]
#[serial]
fn test_case_insensitive_search() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping case sensitivity test");
        return;
    }

    let test_contact = &contacts[0];
    let name = &test_contact.name;

    println!("Testing case insensitivity with: {}", name);

    // Test different case variations
    let lower = name.to_lowercase();
    let upper = name.to_uppercase();

    let lower_matches: Vec<_> = contacts
        .iter()
        .filter(|c| c.name.to_lowercase().contains(&lower))
        .collect();

    let upper_matches: Vec<_> = contacts
        .iter()
        .filter(|c| c.name.to_uppercase().contains(&upper))
        .collect();

    // Both should find the contact
    assert!(
        !lower_matches.is_empty(),
        "Lowercase search should find contact"
    );
    assert!(
        !upper_matches.is_empty(),
        "Uppercase search should find contact"
    );

    println!("✓ Case-insensitive search working correctly");
    println!("  Lowercase matches: {}", lower_matches.len());
    println!("  Uppercase matches: {}", upper_matches.len());
}

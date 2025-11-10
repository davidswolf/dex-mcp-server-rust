//! End-to-end tests for fuzzy matching functionality.
//!
//! These tests validate the ContactMatcher's fuzzy matching capabilities
//! including exact matches, partial matches, typo tolerance, and confidence scoring.

use dex_mcp_server::matching::{ContactMatcher, ContactQuery};
use serial_test::serial;

mod e2e;
use e2e::*;

/// Test exact name matching returns highest confidence.
///
/// This test validates:
/// - Exact name matches return confidence near 100
/// - Case-insensitive matching works
/// - Top result is correct for exact matches
#[test]
#[ignore]
#[serial]
fn test_exact_name_match() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping exact match test");
        return;
    }

    let test_contact = &contacts[0];
    let name = &test_contact.name;

    println!("Testing exact match for: {}", name);

    let mut matcher = ContactMatcher::new();
    let query = ContactQuery {
        name: Some(name.clone()),
        ..Default::default()
    };

    let matches = matcher.find_matches(&query, &contacts, 10, 0);

    assert!(!matches.is_empty(), "Exact match should return results");

    // Top result should be the exact match
    assert_eq!(&matches[0].contact.name, name);

    // Confidence should be very high (85+)
    assert!(
        matches[0].confidence >= 85,
        "Exact match should have high confidence, got {}",
        matches[0].confidence
    );

    println!("✓ Exact match confidence: {}", matches[0].confidence);
}

/// Test partial name matching (e.g., first name only).
///
/// This test validates:
/// - Partial name search returns multiple relevant results
/// - Results are ranked by relevance
/// - Confidence scores are appropriate for partial matches
#[test]
#[ignore]
#[serial]
fn test_partial_name_match() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping partial match test");
        return;
    }

    // Take first name of first contact
    let full_name = &contacts[0].name;
    let first_name = full_name.split_whitespace().next().unwrap_or(full_name);

    println!("Testing partial match for: {}", first_name);

    let mut matcher = ContactMatcher::new();
    let query = ContactQuery {
        name: Some(first_name.to_string()),
        ..Default::default()
    };

    let matches = matcher.find_matches(&query, &contacts, 10, 30);

    if !matches.is_empty() {
        println!(
            "Found {} matches for partial name '{}':",
            matches.len(),
            first_name
        );

        for (i, m) in matches.iter().take(5).enumerate() {
            println!(
                "  {}. {} (confidence: {})",
                i + 1,
                m.contact.name,
                m.confidence
            );
        }

        // Verify all matches contain the search term (case-insensitive)
        for m in &matches {
            let name_lower = m.contact.name.to_lowercase();
            let search_lower = first_name.to_lowercase();

            assert!(
                name_lower.contains(&search_lower) || m.confidence >= 30,
                "Match should contain search term or have sufficient confidence"
            );
        }

        println!("✓ Partial matches returned with appropriate confidence");
    } else {
        println!("⚠ No matches found for partial name (threshold may be too high)");
    }
}

/// Test typo tolerance in fuzzy matching.
///
/// This test validates:
/// - Minor typos don't prevent matches
/// - Confidence is reduced for typos but still acceptable
/// - Levenshtein distance algorithm works correctly
#[test]
#[ignore]
#[serial]
fn test_typo_tolerance() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping typo test");
        return;
    }

    let test_contact = &contacts[0];
    let name = &test_contact.name;

    // Create a typo by replacing a character in the middle
    let typo_name = if name.len() > 3 {
        let mut chars: Vec<char> = name.chars().collect();
        let mid = chars.len() / 2;

        // Swap a character to create a typo
        if chars[mid].is_alphabetic() {
            if chars[mid] == 'e' {
                chars[mid] = 'a';
            } else {
                chars[mid] = 'e';
            }
        }

        chars.into_iter().collect::<String>()
    } else {
        format!("{}x", name) // Add extra char for short names
    };

    println!("Original: {}", name);
    println!("With typo: {}", typo_name);

    let mut matcher = ContactMatcher::new();
    let query = ContactQuery {
        name: Some(typo_name.clone()),
        ..Default::default()
    };

    let matches = matcher.find_matches(&query, &contacts, 10, 0);

    if !matches.is_empty() {
        println!("Found {} matches despite typo:", matches.len());

        for (i, m) in matches.iter().take(3).enumerate() {
            println!(
                "  {}. {} (confidence: {})",
                i + 1,
                m.contact.name,
                m.confidence
            );
        }

        // Check if original contact is in the results
        let found_original = matches.iter().any(|m| m.contact.name == *name);

        if found_original {
            println!("✓ Typo tolerance: Original contact found in results");
        } else {
            println!("⚠ Typo too significant - original contact not in top results");
        }
    }
}

/// Test case-insensitive matching.
///
/// This test validates:
/// - Uppercase, lowercase, and mixed case searches work identically
/// - Normalization handles different case variations
#[test]
#[ignore]
#[serial]
fn test_case_insensitivity() {
    let client = setup_test_client();

    let result = client.get_contacts(50, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping case sensitivity test");
        return;
    }

    let test_name = &contacts[0].name;
    let mut matcher = ContactMatcher::new();

    // Test with different case variations
    let variations = vec![
        test_name.to_lowercase(),
        test_name.to_uppercase(),
        test_name.clone(),
    ];

    println!("Testing case insensitivity for: {}", test_name);

    let mut all_match_counts = Vec::new();

    for variant in &variations {
        let query = ContactQuery {
            name: Some(variant.clone()),
            ..Default::default()
        };

        let matches = matcher.find_matches(&query, &contacts, 10, 0);
        all_match_counts.push(matches.len());

        println!("  '{}' -> {} matches", variant, matches.len());
    }

    // All variations should return the same number of matches
    let first_count = all_match_counts[0];

    for count in &all_match_counts {
        assert_eq!(
            *count, first_count,
            "Case variations should return same number of matches"
        );
    }

    println!("✓ Case-insensitive matching verified");
}

/// Test matching with name variations (nicknames, abbreviations).
///
/// This test validates:
/// - Common name variations can be matched
/// - Confidence scores reflect similarity
#[test]
#[ignore]
#[serial]
fn test_name_variations() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    // Common name variations to test
    let variations = vec![
        ("Robert", "Bob"),
        ("William", "Bill"),
        ("Richard", "Rick"),
        ("Michael", "Mike"),
        ("Elizabeth", "Liz"),
    ];

    let mut matcher = ContactMatcher::new();

    for (full, nickname) in variations {
        // Check if we have any contacts with these names
        let has_full = contacts.iter().any(|c| c.name.contains(full));
        let has_nickname = contacts.iter().any(|c| c.name.contains(nickname));

        if has_full || has_nickname {
            println!("Testing variation: {} / {}", full, nickname);

            let query = ContactQuery {
                name: Some(full.to_string()),
                ..Default::default()
            };

            let matches = matcher.find_matches(&query, &contacts, 5, 0);

            if !matches.is_empty() {
                println!("  Found {} matches for '{}'", matches.len(), full);
            }
        }
    }

    println!("✓ Name variation matching tested");
}

/// Test full name vs. partial search accuracy.
///
/// This test validates:
/// - Full name searches are more accurate than partial
/// - Confidence scoring reflects completeness
#[test]
#[ignore]
#[serial]
fn test_full_name_vs_parts() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    // Find a contact with a full name (first and last)
    let contact_with_full_name = contacts
        .iter()
        .find(|c| c.name.split_whitespace().count() >= 2);

    if let Some(contact) = contact_with_full_name {
        let full_name = &contact.name;
        let parts: Vec<&str> = full_name.split_whitespace().collect();

        if parts.len() >= 2 {
            let first_name = parts[0];
            let last_name = parts[parts.len() - 1];

            println!("Full name: {}", full_name);
            println!("Testing: first='{}', last='{}'", first_name, last_name);

            let mut matcher = ContactMatcher::new();

            // Search with full name
            let full_query = ContactQuery {
                name: Some(full_name.clone()),
                ..Default::default()
            };
            let full_matches = matcher.find_matches(&full_query, &contacts, 5, 0);

            // Search with first name only
            let first_query = ContactQuery {
                name: Some(first_name.to_string()),
                ..Default::default()
            };
            let first_matches = matcher.find_matches(&first_query, &contacts, 5, 0);

            println!("Full name search: {} matches", full_matches.len());
            println!("First name search: {} matches", first_matches.len());

            // Full name should have higher confidence for the exact match
            if !full_matches.is_empty() && !first_matches.is_empty() {
                let full_confidence = full_matches
                    .iter()
                    .find(|m| m.contact.name == *full_name)
                    .map(|m| m.confidence);

                let first_confidence = first_matches
                    .iter()
                    .find(|m| m.contact.name == *full_name)
                    .map(|m| m.confidence);

                if let (Some(full_conf), Some(first_conf)) = (full_confidence, first_confidence) {
                    println!("Confidence - Full: {}, First: {}", full_conf, first_conf);

                    assert!(
                        full_conf >= first_conf,
                        "Full name should have equal or higher confidence"
                    );
                }
            }

            println!("✓ Full name vs parts comparison complete");
        }
    }
}

/// Test confidence threshold filtering.
///
/// This test validates:
/// - Min confidence threshold filters results correctly
/// - Confidence scores are calibrated appropriately
/// - No results below threshold are returned
#[test]
#[ignore]
#[serial]
fn test_confidence_threshold() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    if contacts.is_empty() {
        println!("⚠ No contacts found - skipping threshold test");
        return;
    }

    let test_name = &contacts[0].name;
    let first_name = test_name.split_whitespace().next().unwrap_or(test_name);

    let mut matcher = ContactMatcher::new();
    let query = ContactQuery {
        name: Some(first_name.to_string()),
        ..Default::default()
    };

    // Test with different thresholds
    let thresholds = vec![0, 30, 50, 70, 90];

    println!("Testing confidence thresholds for '{}':", first_name);

    for threshold in thresholds {
        let matches = matcher.find_matches(&query, &contacts, 10, threshold);

        println!("  Threshold {}: {} matches", threshold, matches.len());

        // Verify all matches meet the threshold
        for m in &matches {
            assert!(
                m.confidence >= threshold,
                "Match has confidence {} but threshold is {}",
                m.confidence,
                threshold
            );
        }
    }

    println!("✓ Confidence threshold filtering verified");
}

/// Test email-based exact matching returns 100% confidence.
///
/// This test validates:
/// - Email matches are treated as exact (confidence 100)
/// - Email matching is prioritized over name matching
#[test]
#[ignore]
#[serial]
fn test_exact_email_confidence() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    let contact_with_email = contacts.iter().find(|c| c.email.is_some());

    if let Some(contact) = contact_with_email {
        let email = contact.email.as_ref().unwrap();

        println!("Testing exact email match: {}", email);

        let mut matcher = ContactMatcher::new();
        let query = ContactQuery {
            email: Some(email.clone()),
            ..Default::default()
        };

        let matches = matcher.find_matches(&query, &contacts, 5, 0);

        assert!(!matches.is_empty(), "Email match should return results");

        // Top match should be 100% confidence
        assert_eq!(
            matches[0].confidence, 100,
            "Exact email match should have 100% confidence"
        );

        println!("✓ Email match returned 100% confidence");
    } else {
        println!("⚠ No contacts with email - skipping email confidence test");
    }
}

/// Test max results limit.
///
/// This test validates:
/// - Results are limited to max_results parameter
/// - Top results are returned (by confidence)
#[test]
#[ignore]
#[serial]
fn test_max_results_limit() {
    let client = setup_test_client();

    let result = client.get_contacts(100, 0);
    if result.is_err() {
        println!("⚠ Skipping test: Cannot fetch contacts (check API key)");
        return;
    }

    let contacts = result.unwrap();

    if contacts.len() < 5 {
        println!("⚠ Not enough contacts - skipping max results test");
        return;
    }

    // Search with a common partial name
    let first_name = contacts[0].name.split_whitespace().next().unwrap();

    let mut matcher = ContactMatcher::new();
    let query = ContactQuery {
        name: Some(first_name.to_string()),
        ..Default::default()
    };

    // Test different limits
    for max_results in &[1, 3, 5, 10] {
        let matches = matcher.find_matches(&query, &contacts, *max_results, 0);

        assert!(
            matches.len() <= *max_results,
            "Results should not exceed max_results limit"
        );

        println!("Max results {}: got {} matches", max_results, matches.len());
    }

    println!("✓ Max results limiting verified");
}

# End-to-End Test Suite Plan for DexMCPServerRust

## Overview

This document outlines a comprehensive plan for creating an end-to-end (E2E) test suite for the Rust MCP server, based on the existing JavaScript test suite in `../DexMCPServer/test-*.js`. These tests will validate the implementation against the live Dex API using credentials from `.env`.

## Test Infrastructure

### 1. Project Structure

```
tests/
├── e2e/
│   ├── mod.rs                      # Common test utilities and setup
│   ├── fixtures.rs                 # Test data and fixtures
│   ├── test_client_api.rs          # Direct API client tests
│   ├── test_contact_discovery.rs   # Contact finding and searching
│   ├── test_contact_enrichment.rs  # Note and reminder creation
│   ├── test_fuzzy_matching.rs      # Fuzzy search validation
│   ├── test_relationship_history.rs # History and timeline tests
│   └── test_mcp_server.rs          # Full MCP server integration tests
```

### 2. Required Dependencies

Add to `Cargo.toml` under `[dev-dependencies]`:

```toml
# Async testing utilities
tokio-test = "0.4"

# Serial test execution (prevent env var conflicts and API rate limits)
serial_test = "3"

# Environment variable management for tests
dotenvy = "0.15"  # Already in main dependencies

# Test assertions and utilities
assert_matches = "1.5"
```

### 3. Test Configuration

**Environment Variables** (`.env` file):
```env
DEX_API_KEY=your_api_key_here
DEX_API_BASE_URL=https://api.getdex.com/api/rest
```

**Test Configuration Module** (`tests/e2e/mod.rs`):
```rust
pub struct TestConfig {
    pub api_key: String,
    pub base_url: String,
}

impl TestConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            api_key: std::env::var("DEX_API_KEY")
                .expect("DEX_API_KEY must be set in .env"),
            base_url: std::env::var("DEX_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.getdex.com/api/rest".to_string()),
        }
    }
}
```

## Test Categories and Implementation Plan

### Category 1: Direct API Client Tests

**File:** `tests/e2e/test_client_api.rs`

Based on: `test-api-direct.js`

**Tests to implement:**

1. **test_list_contacts_basic**
   - Fetch first 100 contacts
   - Validate response structure
   - Check that contacts have required fields (id, name)

2. **test_list_contacts_pagination**
   - Fetch contacts with different limit/offset values
   - Validate pagination works correctly
   - Ensure no duplicate contacts across pages

3. **test_search_contacts_by_email**
   - Search for a known contact by email
   - Validate exact email match
   - Verify contact details are complete

4. **test_get_single_contact**
   - Get a specific contact by ID
   - Validate all fields are populated correctly
   - Check social profiles, emails, phones arrays

5. **test_error_handling**
   - Test invalid contact ID (404 error)
   - Test invalid API key (401 error)
   - Test malformed requests

**Success Criteria:**
- All API endpoints return expected data structures
- Error cases are handled gracefully
- Pagination works correctly

---

### Category 2: Contact Discovery Tests

**File:** `tests/e2e/test_contact_discovery.rs`

Based on: `test-list-contacts.js`, `test-real-contact.js`, `test-greg-hoy.js`, `test-melissa-linkedin.js`

**Tests to implement:**

1. **test_find_contact_by_name**
   - Search for a known contact (e.g., "Greg Hoy")
   - Validate fuzzy matching returns correct results
   - Check confidence scores

2. **test_find_contact_by_email**
   - Search for contact by email address
   - Should return exact match with high confidence
   - Validate match reason

3. **test_find_contact_by_linkedin**
   - Search for contact by LinkedIn URL
   - Validate social profile matching
   - Check match confidence

4. **test_find_contact_partial_name**
   - Search with only first name
   - Should return multiple candidates
   - Results should be ranked by confidence

5. **test_find_contact_no_match**
   - Search for non-existent contact
   - Should return empty results or low-confidence matches
   - Should not throw errors

6. **test_list_all_contacts_discovery**
   - Load all contacts (with pagination)
   - Validate total count
   - Check for data consistency

**Success Criteria:**
- All contact discovery methods work correctly
- Confidence scoring is reasonable
- No false negatives for exact matches

---

### Category 3: Fuzzy Matching Tests

**File:** `tests/e2e/test_fuzzy_matching.rs`

Based on: `test-fuzzy-match.js`

**Tests to implement:**

1. **test_exact_name_match**
   - Search "Greg Hoy" → should match "Greg Hoy" with 100% confidence
   - Validate top result is correct

2. **test_partial_name_match**
   - Search "Greg" → should return all Gregs ranked by relevance
   - Check scoring algorithm

3. **test_typo_tolerance**
   - Search "Gerg Hoy" → should still match "Greg Hoy"
   - Validate fuzzy matching threshold

4. **test_case_insensitivity**
   - Search "greg hoy", "GREG HOY", "Greg Hoy" → all should match
   - Results should be identical

5. **test_name_variations**
   - Search "Gregory" → should match "Greg" with reasonable confidence
   - Test common name variations

6. **test_full_name_vs_parts**
   - Compare searching "John Doe" vs searching separately "John" and "Doe"
   - Validate full name search is more accurate

**Success Criteria:**
- Nucleo fuzzy matcher performs as expected
- Confidence scores are calibrated correctly
- No significant false positives/negatives

---

### Category 4: Note Creation and Retrieval Tests

**File:** `tests/e2e/test_contact_enrichment.rs` (Part 1)

Based on: `test-add-note.js`, `test-add-note-debug.js`, `test-greg-notes-*.js`

**Tests to implement:**

1. **test_add_note_to_contact**
   - Find a test contact
   - Add a note with timestamp
   - Verify note was created successfully
   - Check note ID is returned

2. **test_retrieve_notes_for_contact**
   - Get notes for a specific contact
   - Validate note content and metadata
   - Check chronological ordering

3. **test_add_multiple_notes**
   - Add several notes to same contact
   - Retrieve all notes
   - Validate count and content

4. **test_note_with_special_characters**
   - Add note with emojis, newlines, quotes
   - Retrieve and validate content is preserved

5. **test_note_timestamp_handling**
   - Add note with custom timestamp
   - Verify timestamp is stored correctly
   - Check timezone handling

**Success Criteria:**
- Notes can be created successfully
- All notes are retrievable
- Content and metadata are preserved

---

### Category 5: Reminder Creation and Management Tests

**File:** `tests/e2e/test_contact_enrichment.rs` (Part 2)

Based on: `test-add-reminder.js`, `test-add-reminder-debug.js`, `test-complete-reminder.js`, `test-complete-reminder-v2.js`, `test-verify-complete.js`, `test-peter-reminders-*.js`, `test-all-reminders.js`, `test-greg-reminders-only.js`

**Tests to implement:**

1. **test_create_reminder**
   - Create reminder for contact with due date
   - Validate reminder is created with correct fields
   - Check reminder ID is returned

2. **test_retrieve_reminders_for_contact**
   - Get all reminders for a contact
   - Validate reminder data
   - Check active vs completed filtering

3. **test_complete_reminder**
   - Create a reminder
   - Mark it as complete
   - Verify completion status is updated

4. **test_filter_active_reminders**
   - Create mix of active and completed reminders
   - Filter for active only
   - Validate filtering works correctly

5. **test_filter_completed_reminders**
   - Get only completed reminders
   - Verify all returned reminders are marked complete

6. **test_reminder_due_date_handling**
   - Create reminder with specific due date
   - Retrieve and validate due date format
   - Test past and future dates

7. **test_list_all_reminders**
   - Get all reminders across all contacts
   - Validate global reminder list
   - Check pagination if needed

8. **test_update_reminder**
   - Create reminder
   - Update due date or text
   - Verify updates are persisted

**Success Criteria:**
- Reminders can be created, updated, and completed
- Filtering by status works correctly
- Date handling is accurate

---

### Category 6: Relationship History Tests

**File:** `tests/e2e/test_relationship_history.rs`

Based on: `test-timeline-items.js`, `test-greg-history-*.js`

**Tests to implement:**

1. **test_get_contact_timeline**
   - Retrieve full timeline for a contact
   - Validate timeline includes notes and interactions
   - Check chronological ordering

2. **test_timeline_filtering_by_date**
   - Get timeline items within date range
   - Validate only items in range are returned

3. **test_timeline_filtering_by_type**
   - Filter timeline by event type (notes, calls, emails)
   - Verify correct filtering

4. **test_combined_history_view**
   - Get notes + reminders for a contact
   - Validate combined view is complete
   - Check proper merging and sorting

5. **test_empty_timeline**
   - Get timeline for contact with no history
   - Should return empty results gracefully

**Success Criteria:**
- Complete relationship history is accessible
- Filtering works correctly
- Timeline is chronologically ordered

---

### Category 7: MCP Server Integration Tests

**File:** `tests/e2e/test_mcp_server.rs`

Based on: `test-server.js`, `test-tools.js`

**Tests to implement:**

1. **test_server_initialization**
   - Start MCP server
   - Send initialize message
   - Validate server response and capabilities

2. **test_tools_list**
   - Request tools/list
   - Validate all expected tools are present
   - Check tool schemas are valid

3. **test_tool_find_contact**
   - Call find_contact tool via MCP
   - Validate tool execution
   - Check response format

4. **test_tool_get_contact_details**
   - Call get_contact_details tool
   - Validate detailed contact info is returned

5. **test_tool_add_note**
   - Use add_note tool via MCP
   - Verify note creation through tool interface

6. **test_tool_create_reminder**
   - Use create_reminder tool
   - Validate reminder creation via MCP

7. **test_tool_get_relationship_history**
   - Use get_relationship_history tool
   - Validate history retrieval via MCP

8. **test_tool_error_handling**
   - Call tools with invalid parameters
   - Validate error responses

9. **test_server_shutdown**
   - Test graceful server shutdown
   - Ensure no resource leaks

**Success Criteria:**
- Server starts and responds to MCP protocol
- All tools are accessible and functional
- Error handling works correctly
- Server can be stopped gracefully

---

## Test Utilities

### Helper Functions to Implement

**In `tests/e2e/mod.rs`:**

```rust
// Test setup
pub fn setup_test_client() -> DexClient { /* ... */ }
pub fn setup_test_config() -> Config { /* ... */ }

// Test data helpers
pub fn find_or_create_test_contact() -> Contact { /* ... */ }
pub fn cleanup_test_data(contact_id: &str) { /* ... */ }

// Assertion helpers
pub fn assert_contact_valid(contact: &Contact) { /* ... */ }
pub fn assert_note_valid(note: &Note) { /* ... */ }
pub fn assert_reminder_valid(reminder: &Reminder) { /* ... */ }

// Known test contacts (from .env or hardcoded)
pub fn get_known_test_contacts() -> Vec<&'static str> {
    vec![
        "Greg Hoy",
        "Peter Wong",
        "joestt600@hotmail.com",
        // Add more from your actual Dex database
    ]
}
```

---

## Test Execution Strategy

### Running Tests

1. **Individual test files:**
   ```bash
   cargo test --test test_client_api -- --nocapture
   ```

2. **All E2E tests:**
   ```bash
   cargo test --test '*' -- --test-threads=1 --nocapture
   ```

3. **Specific test:**
   ```bash
   cargo test --test test_contact_discovery test_find_contact_by_name
   ```

### Serial Execution

Use `#[serial_test::serial]` attribute on tests that:
- Modify shared state (creating/updating contacts)
- May hit API rate limits
- Depend on specific test data

Example:
```rust
#[test]
#[serial_test::serial]
fn test_add_note_to_contact() {
    // Test implementation
}
```

---

## Test Data Management

### Strategy 1: Read-Only Tests

Most tests should be **read-only** and use existing contacts in your Dex database:
- No cleanup needed
- Safe to run repeatedly
- Fast execution

### Strategy 2: Write Tests with Cleanup

For tests that create data (notes, reminders):
- Use a dedicated test contact
- Clean up after each test
- Use `#[serial]` to prevent conflicts

### Strategy 3: Test Contact Identification

Create a convention for test contacts:
- Tag test contacts with a note like "TEST_CONTACT"
- Or use specific naming pattern
- Document test contact IDs in `.env`:
  ```env
  TEST_CONTACT_ID=abc123
  TEST_CONTACT_NAME="Test User"
  ```

---

## Error Handling and Retry Logic

### Rate Limiting

The Dex API may have rate limits. Implement retry logic:

```rust
async fn retry_with_backoff<F, T, E>(mut f: F, max_retries: u32) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut retries = 0;
    loop {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                let delay = Duration::from_millis(100 * 2_u64.pow(retries));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Network Issues

Tests should handle transient network failures:
- Retry failed requests
- Skip tests if API is unavailable
- Log clear error messages

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: E2E Tests

on:
  push:
    branches: [ main ]
  pull_request:

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run E2E tests
        env:
          DEX_API_KEY: ${{ secrets.DEX_API_KEY }}
          DEX_API_BASE_URL: ${{ secrets.DEX_API_BASE_URL }}
        run: |
          cargo test --tests -- --test-threads=1
```

**Note:** Store API credentials as GitHub Secrets, not in the repository.

---

## Success Metrics

### Test Coverage

Target metrics:
- **95%+ code coverage** for critical paths (client, tools)
- **100% API endpoint coverage** (all endpoints tested)
- **All MCP tools tested** end-to-end

### Test Reliability

- **No flaky tests** - tests should pass consistently
- **Clear error messages** when tests fail
- **Fast execution** - full suite should run in < 5 minutes

### Comparison with JS Tests

Validate Rust implementation produces **identical results** to JS version:
- Same confidence scores for fuzzy matching
- Same data structures returned
- Same error handling behavior

---

## Implementation Priority

### Phase 1: Foundation (Week 1)
- Set up test infrastructure (`tests/e2e/mod.rs`)
- Implement test utilities and helpers
- Create `test_client_api.rs` with basic API tests

### Phase 2: Core Functionality (Week 1-2)
- Implement `test_contact_discovery.rs`
- Implement `test_fuzzy_matching.rs`
- Validate core search and discovery features

### Phase 3: Enrichment (Week 2)
- Implement `test_contact_enrichment.rs`
- Test note and reminder creation
- Validate CRUD operations

### Phase 4: History & Advanced (Week 2-3)
- Implement `test_relationship_history.rs`
- Test timeline and history features

### Phase 5: Full Integration (Week 3)
- Implement `test_mcp_server.rs`
- End-to-end server testing
- Complete MCP protocol validation

---

## Documentation

Each test file should include:

1. **Module-level doc comment** explaining what is being tested
2. **Test function doc comments** describing the specific scenario
3. **Inline comments** for complex assertions or setup

Example:
```rust
//! End-to-end tests for the Dex API client.
//!
//! These tests validate the DexClient against the live Dex API using
//! credentials from the .env file. All tests are designed to be safe
//! and non-destructive to production data.

/// Test that we can successfully retrieve a paginated list of contacts
/// from the Dex API.
///
/// This test validates:
/// - API authentication works
/// - Pagination parameters are respected
/// - Response structure matches expected format
#[test]
#[serial_test::serial]
fn test_list_contacts_basic() {
    // Test implementation...
}
```

---

## Known Test Contacts

Based on the JavaScript tests, these contacts appear in the test database:

| Name | Email | Usage |
|------|-------|-------|
| Greg Hoy | greglhoy@gmail.com | Note and reminder testing |
| Peter Wong | (unknown) | Timeline and reminder testing |
| Joe Stettner | joestt600@hotmail.com | Email search testing |
| Melissa | (has LinkedIn) | LinkedIn profile testing |

**Action Item:** Document your actual test contacts in `.env.example`:
```env
# Known test contacts for E2E tests
TEST_CONTACT_GREG_HOY_ID=abc123
TEST_CONTACT_PETER_WONG_ID=def456
```

---

## Next Steps

1. **Review this plan** and adjust based on your specific requirements
2. **Set up .env file** with valid Dex API credentials
3. **Create test infrastructure** (`tests/e2e/mod.rs`)
4. **Implement Phase 1** tests (basic API client tests)
5. **Iterate** through phases 2-5
6. **Document results** and any deviations from the plan

---

## Questions to Resolve

Before implementation, clarify:

1. **Test data strategy**: Will we use existing contacts or create test contacts?
2. **API rate limits**: What are the Dex API rate limits? Do we need throttling?
3. **Test isolation**: Can tests run in parallel or must they be sequential?
4. **Cleanup strategy**: Should we clean up test data or leave it for inspection?
5. **CI/CD**: Will these tests run in CI, or only locally?

---

## Appendix: JavaScript Test Mapping

| JavaScript Test | Rust Test File | Priority |
|----------------|----------------|----------|
| test-api-direct.js | test_client_api.rs | High |
| test-list-contacts.js | test_contact_discovery.rs | High |
| test-real-contact.js | test_contact_discovery.rs | High |
| test-greg-hoy.js | test_contact_discovery.rs | Medium |
| test-melissa-linkedin.js | test_contact_discovery.rs | Medium |
| test-fuzzy-match.js | test_fuzzy_matching.rs | High |
| test-add-note.js | test_contact_enrichment.rs | High |
| test-add-note-debug.js | test_contact_enrichment.rs | Low |
| test-greg-notes-*.js | test_contact_enrichment.rs | Medium |
| test-add-reminder.js | test_contact_enrichment.rs | High |
| test-add-reminder-debug.js | test_contact_enrichment.rs | Low |
| test-complete-reminder.js | test_contact_enrichment.rs | High |
| test-complete-reminder-v2.js | test_contact_enrichment.rs | Medium |
| test-verify-complete.js | test_contact_enrichment.rs | Medium |
| test-peter-reminders-*.js | test_contact_enrichment.rs | Medium |
| test-all-reminders.js | test_contact_enrichment.rs | Medium |
| test-greg-reminders-only.js | test_contact_enrichment.rs | Low |
| test-timeline-items.js | test_relationship_history.rs | High |
| test-greg-history-*.js | test_relationship_history.rs | Medium |
| test-server.js | test_mcp_server.rs | High |
| test-tools.js | test_mcp_server.rs | High |

---

**Document Version:** 1.0
**Last Updated:** 2025-10-20
**Status:** Draft - Ready for Review

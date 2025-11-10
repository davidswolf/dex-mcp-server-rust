# CRUD Tests and Update Endpoint Fixes

## Overview

This document describes the comprehensive end-to-end CRUD tests implemented for Contacts, Notes, and Reminders, and the critical fixes required to make create and update operations work with the Dex API.

## Test Files

### 1. `tests/test_contacts_crud.rs`
Comprehensive end-to-end tests for Contact CRUD operations against the live Dex API.

**Tests:**
- ✅ `test_contact_crud_lifecycle` - Full CRUD cycle (Create → Read → Update → Delete)
- ✅ `test_contact_batch_create_and_cleanup` - Multiple contact creation with batch cleanup
- ✅ `test_contact_multiple_updates` - Sequential updates to verify persistence
- ✅ `test_contact_update_nonexistent` - Error handling for invalid contact IDs
- ✅ `test_contact_delete_nonexistent` - Error handling for delete operations
- ✅ `test_contact_minimal_create` - Creating contact with minimal information
- ✅ `test_contact_update_email_phone` - Updating email and phone numbers

### 2. `tests/test_notes_crud.rs`
Comprehensive end-to-end tests for Note CRUD operations against the live Dex API.

**Tests:**
- ✅ `test_note_crud_lifecycle` - Full CRUD cycle (Create → Read → Update → Delete)
- ✅ `test_note_batch_create_and_cleanup` - Multiple note creation with batch cleanup
- ✅ `test_note_multiple_updates` - Sequential updates to verify persistence
- ✅ `test_note_update_nonexistent` - Error handling for invalid note IDs
- ✅ `test_note_delete_nonexistent` - Error handling for delete operations

### 3. `tests/test_reminders_crud.rs`
Comprehensive end-to-end tests for Reminder CRUD operations against the live Dex API.

**Tests:**
- ✅ `test_reminder_crud_lifecycle` - Full CRUD cycle (Create → Read → Update → Delete)
- ✅ `test_reminder_completion_workflow` - Mark reminder as complete and update
- ✅ `test_reminder_batch_create_and_cleanup` - Batch operations with cleanup
- ✅ `test_reminder_update_due_date` - Due date modification
- ✅ `test_reminder_with_past_due_date` - Overdue reminder handling
- ✅ `test_reminder_update_nonexistent` - Error handling for invalid IDs
- ✅ `test_reminder_delete_nonexistent` - Error handling for deletes

## Automatic Cleanup with RAII Guards

Tests use RAII (Resource Acquisition Is Initialization) pattern to guarantee cleanup even if tests fail:

```rust
struct NoteGuard<'a> {
    client: &'a dex_mcp_server::DexClient,
    note_id: Option<String>,
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
```

## Critical Fixes for Update Operations

### Problem 1: Update Request Structure

**Issue:** The update endpoint was rejecting all fields with "Unexpected variable" errors.

**Root Cause:** The Dex API update endpoints expect a `changes` wrapper object, not raw field values.

**What We Were Sending (WRONG):**
```json
{
  "id": "...",
  "note": "...",
  "event_time": "..."
}
```

**What the API Expects (CORRECT):**
```json
{
  "changes": {
    "note": "..."
  }
}
```

**Solution:** Created dedicated update request structures:

#### Notes (`src/models/note.rs`):
```rust
#[derive(Debug, Clone, Serialize)]
struct NoteChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_time: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateNoteRequest {
    changes: NoteChanges,
}

impl From<&Note> for UpdateNoteRequest {
    fn from(note: &Note) -> Self {
        Self {
            changes: NoteChanges {
                note: if note.content.is_empty() {
                    None
                } else {
                    Some(note.content.clone())
                },
                event_time: None, // Don't update event_time
            },
        }
    }
}
```

#### Reminders (`src/models/reminder.rs`):
```rust
#[derive(Debug, Clone, Serialize)]
struct ReminderChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_complete: Option<Bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_at_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_at_time: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateReminderRequest {
    changes: ReminderChanges,
}
```

#### Contacts (`src/models/contact.rs`):
```rust
// Create structure
#[derive(Debug, Clone, Serialize)]
pub struct CreateContactRequest {
    contact: ContactPayload,
}

// Update structure
#[derive(Debug, Clone, Serialize)]
pub struct UpdateContactRequest {
    changes: ContactChanges,
    contact_emails: Option<Vec<ContactEmailUpdate>>,
    contact_phone_numbers: Option<Vec<ContactPhoneUpdate>>,
    update_contact_emails: bool,
    update_contact_phone_numbers: bool,
}
```

**Key differences for contacts:**
- Create uses `{ "contact": { ... } }` wrapper
- Update uses `{ "changes": { ... }, "update_contact_emails": bool, ... }` structure
- Emails and phones are managed through separate arrays with update flags

### Problem 2: Serializing Read-Only Fields

**Issue:** The API was rejecting `id` and `created_at` fields during updates.

**Root Cause:** These fields are read-only and should only be deserialized, never serialized.

**Solution:** Added `#[serde(skip_serializing)]` to read-only fields:

```rust
// In Note struct
#[serde(skip_serializing)]
pub id: String,

#[serde(rename = "event_time", skip_serializing)]
pub created_at: String,

// In Reminder struct
#[serde(skip_serializing)]
pub id: String,

#[serde(default = "default_timestamp", skip_serializing)]
pub created_at: String,
```

### Problem 3: Wrapped API Responses

**Issue:** Create and update operations return wrapped responses that don't directly deserialize to our models.

**Root Cause:** The Dex API wraps responses in mutation-specific objects.

**Response Structures:**

#### Note Creation Response:
```json
{
  "insert_timeline_items_one": {
    "id": "...",
    "note": "...",
    "timeline_items_contacts": [{"contact": {"id": "..."}}]
  }
}
```

#### Note Update Response:
```json
{
  "update_timeline_items_by_pk": {
    "id": "...",
    "note": "...",
    "contact_ids": [{"contact_id": "..."}]
  }
}
```

#### Reminder Creation Response:
```json
{
  "insert_reminders_one": {
    "id": "...",
    "body": "...",
    "due_at_date": "...",
    "is_complete": false,
    "contact_ids": [{"contact_id": "..."}]
  }
}
```

#### Reminder Update Response:
```json
{
  "update_reminders_by_pk": {
    "id": "...",
    "text": "...",
    "due_at_date": "...",
    "is_complete": false,
    "reminders_contacts": [{"contact_id": "..."}]
  }
}
```

#### Contact Creation Response:
```json
{
  "insert_contacts_one": {
    "id": "...",
    "first_name": "...",
    "last_name": "...",
    "emails": [{"email": "..."}],
    "phones": []
  }
}
```

#### Contact Update Response:
```json
{
  "delete_contact_emails": {"affected_rows": 1},
  "insert_contact_emails": {"affected_rows": 1},
  "update_contacts_by_pk": {
    "id": "...",
    "first_name": "...",
    "last_name": "...",
    "job_title": "...",
    "emails": [{"email": "..."}],
    "phones": []
  }
}
```

**Solution:** Manual JSON parsing to extract and map fields correctly.

Example for note updates (`src/client/mod.rs`):
```rust
pub fn update_note(&self, note_id: &str, note: &Note) -> DexApiResult<Note> {
    // ... send request ...

    // Parse the wrapped response
    let value: serde_json::Value = serde_json::from_str(&response_body)
        .map_err(DexApiError::JsonError)?;

    let timeline_item = value.get("update_timeline_items_by_pk")
        .ok_or_else(|| DexApiError::HttpError("Missing update_timeline_items_by_pk in API response".to_string()))?;

    // Extract and map fields
    let id = timeline_item.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let content = timeline_item.get("note")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Extract contact_id from nested array
    let contact_id = timeline_item.get("contact_ids")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("contact_id"))
        .and_then(|v| v.as_str())
        .unwrap_or(&note.contact_id)
        .to_string();

    // Build the Note object
    let updated_note = Note {
        id,
        contact_id,
        content,
        created_at: note.created_at.clone(),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        tags: note.tags.clone(),
        source: note.source.clone(),
    };

    Ok(updated_note)
}
```

### Problem 4: Reminder Title Field

**Issue:** API rejected reminder creation with error: "field 'title' not found in type: 'reminders_insert_input'"

**Root Cause:** The API doesn't support the `title` field during reminder creation, even though documentation suggested it.

**Solution:** Set `title` to `None` in CreateReminderRequest:

```rust
impl From<&Reminder> for CreateReminderRequest {
    fn from(reminder: &Reminder) -> Self {
        Self {
            reminder: ReminderPayload {
                title: None, // API doesn't support title field during creation
                text: reminder.text.clone(),
                is_complete: if reminder.completed { Some(true) } else { None },
                due_at_date: reminder.due_date.clone(),
                // ...
            },
        }
    }
}
```

## Running the Tests

### Prerequisites
Set environment variables:
```bash
export DEX_API_KEY="your-api-key-here"
export DEX_API_URL="https://api.getdex.com/graphql"
```

### Run All CRUD Tests
```bash
cargo test crud_lifecycle -- --nocapture
```

### Run Specific Tests
```bash
# Contacts only
cargo test test_contacts_crud -- --nocapture

# Notes only
cargo test test_notes_crud -- --nocapture

# Reminders only
cargo test test_reminders_crud -- --nocapture

# Specific lifecycle tests
cargo test test_contact_crud_lifecycle -- --nocapture
cargo test test_note_crud_lifecycle -- --nocapture
cargo test test_reminder_crud_lifecycle -- --nocapture
```

## Test Results

All three CRUD lifecycle tests passing:

```
✓ All CRUD operations completed successfully
test test_contact_crud_lifecycle ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 4.10s

✓ All CRUD operations completed successfully
test test_note_crud_lifecycle ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 1.57s

✓ All CRUD operations completed successfully
test test_reminder_crud_lifecycle ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 1.71s
```

Example output from contact CRUD test:

```
1. Testing CREATE contact...
  ✓ Contact created successfully
    Contact ID: 4646a188-aeb3-4fbe-a090-135d8690d969
    Name: E2ETest20251022002321 CRUDLifecycle
    Email: e2etest20251022002321@example.com

2. Testing READ contact...
  ✓ Retrieved 100 contacts
  ✓ Found our test contact in results

3. Testing UPDATE contact...
  ✓ Contact updated successfully
    New name: E2ETest20251022002321_UPDATED CRUDLifecycle
    New job title: Senior Engineer
  Verifying update persisted...
  ✓ Update verified

4. Testing DELETE contact...
  ✓ Contact deleted successfully
  Verifying deletion...
  ✓ Deletion verified - contact no longer exists

✓ All CRUD operations completed successfully
```

## Files Modified

1. **`src/models/contact.rs`**
   - Added `CreateContactRequest` structure with `contact` wrapper
   - Added `UpdateContactRequest` structure with `changes` wrapper and email/phone update flags
   - Email and phone handling through nested data structures

2. **`src/models/note.rs`**
   - Added `UpdateNoteRequest` structure with `changes` wrapper
   - Added `#[serde(skip_serializing)]` to `id` and `created_at`

3. **`src/models/reminder.rs`**
   - Added `UpdateReminderRequest` structure with `changes` wrapper
   - Added `#[serde(skip_serializing)]` to `id` and `created_at`
   - Set `title` to `None` in create requests

4. **`src/client/mod.rs`**
   - Updated `create_contact` to use `CreateContactRequest` and parse `insert_contacts_one` wrapper
   - Updated `update_contact` to use `UpdateContactRequest` and parse `update_contacts_by_pk` wrapper
   - Updated `create_note` to parse `insert_timeline_items_one` wrapper
   - Updated `update_note` to use `UpdateNoteRequest` and parse `update_timeline_items_by_pk`
   - Updated `create_reminder` to parse `insert_reminders_one` wrapper
   - Updated `update_reminder` to use `UpdateReminderRequest` and parse `update_reminders_by_pk`

5. **`tests/test_contacts_crud.rs`** - New file
   - 8 comprehensive test cases for contact CRUD operations

6. **`tests/test_notes_crud.rs`** - New file
   - 5 comprehensive test cases for note CRUD operations

7. **`tests/test_reminders_crud.rs`** - New file
   - 7 comprehensive test cases for reminder CRUD operations

8. **`tests/e2e/fixtures.rs`**
   - Updated `sample_contact` to accept first_name, last_name, and email parameters

9. **`Cargo.toml`**
   - Added `serial_test = "3.2"` dependency

## API Documentation References

The fixes were based on actual Dex API documentation:
- [Create Contact](https://getdex.com/docs/api-reference/contacts/post)
- [Update Contact](https://getdex.com/docs/api-reference/contacts/put)
- [Create Note](https://getdex.com/docs/api-reference/notes/post)
- [Update Note](https://getdex.com/docs/api-reference/notes/put)
- [Create Reminder](https://getdex.com/docs/api-reference/reminders/post)
- [Update Reminder](https://getdex.com/docs/api-reference/reminders/put)

## Key Takeaways

1. **Create endpoints require wrapper objects** - `contact`, `timeline_event`, or `reminder` wrapper
2. **Update endpoints require `changes` wrapper** - Don't send raw model fields
3. **Read-only fields must skip serialization** - Use `#[serde(skip_serializing)]`
4. **API responses are wrapped** - Manually parse JSON to extract actual data
5. **Field names differ between operations** - Create vs Update vs Read may use different names
6. **Contact emails/phones have special handling** - Separate arrays with update flags
7. **RAII guards ensure cleanup** - Tests clean up after themselves even on failure
8. **Sequential execution prevents conflicts** - Use `#[serial]` for live API tests

## Future Improvements

- [ ] Add tests for batch update operations
- [ ] Add tests for concurrent CRUD operations
- [ ] Add performance benchmarks for CRUD operations
- [ ] Add integration with mock API server for offline testing
- [ ] Add tests for edge cases (special characters, very long content, etc.)

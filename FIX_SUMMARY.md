# Fix Summary - Notes and Reminders Creation

## Problem

Notes and reminders were failing to create with the error:
```
"Unexpected variable due_at_date"
```

The verbose logging revealed that the request body structure was completely wrong for the Dex REST API.

## Root Cause

The original implementation was sending incorrectly formatted request bodies that didn't match the Dex API specification:

### What We Were Sending (WRONG)

**Notes:**
```json
{
  "contacts": [{"contact_id": "..."}],
  "note": "...",
}
```

**Reminders:**
```json
{
  "contact_ids": [{"contact_id": "..."}],
  "body": "...",
  "due_at_date": "..."
}
```

### What the API Expects (CORRECT)

**Notes:** (POST to `/api/rest/timeline_items`)
```json
{
  "timeline_event": {
    "note": "write your note here...",
    "event_time": "2023-05-19T01:03:27.083Z",
    "meeting_type": "note",
    "timeline_items_contacts": {
      "data": [{"contact_id": "75b7bc73-7be0-41a6-960a-183555c80976"}]
    }
  }
}
```

**Reminders:** (POST to `/api/rest/reminders`)
```json
{
  "reminder": {
    "title": "sample title",
    "text": "adding sample reminder",
    "is_complete": false,
    "due_at_date": "2023-06-01",
    "reminders_contacts": {
      "data": [{"contact_id": "2a6102bc-972b-46da-8d3d-4eea17a757ce"}]
    }
  }
}
```

## Changes Made

### 1. Rewrote CreateReminderRequest (src/models/reminder.rs)

**Before:**
- Flat structure with `contact_ids`, `body`, `due_at_date`

**After:**
- Nested structure with `reminder` wrapper
- Contains `ReminderPayload` with:
  - `title` - extracted from text
  - `text` - the reminder description
  - `is_complete` - completion status
  - `due_at_date` - the due date
  - `reminders_contacts.data` - array of contacts

### 2. Rewrote CreateNoteRequest (src/models/note.rs)

**Before:**
- Flat structure with `contacts`, `note`

**After:**
- Nested structure with `timeline_event` wrapper
- Contains `TimelineEventPayload` with:
  - `note` - the content
  - `event_time` - ISO 8601 timestamp
  - `meeting_type` - always "note"
  - `timeline_items_contacts.data` - array of contacts

### 3. Updated Note Creation Endpoint (src/client/mod.rs)

Changed from `/notes` to `/timeline_items` to match the correct API endpoint.

### 4. Added chrono Dependency (Cargo.toml)

Added `chrono = "0.4"` for timestamp handling in note creation.

### 5. Enhanced Logging

Added comprehensive logging throughout the creation pipeline:
- Request payloads (DEBUG level)
- Response bodies (DEBUG level)
- Success/failure messages (INFO/ERROR level)
- HTTP request/response details (DEBUG level)

## Testing

All 82 unit tests pass, including:
- `test_create_note_request_serialization` - Verifies correct note structure
- `test_create_reminder_request_serialization` - Verifies correct reminder structure

## Expected Behavior Now

### Creating a Reminder

**Input:**
```json
{
  "note": "Reach out to Sayee",
  "contact_id": "abb29721-d8c1-4a9f-a684-05c3ec7595ee",
  "reminder_date": "2025-10-22"
}
```

**Sent to API:**
```json
{
  "reminder": {
    "title": "Reach out to Sayee",
    "text": "Reach out to Sayee",
    "due_at_date": "2025-10-22",
    "reminders_contacts": {
      "data": [
        {"contact_id": "abb29721-d8c1-4a9f-a684-05c3ec7595ee"}
      ]
    }
  }
}
```

### Creating a Note

**Input:**
```json
{
  "note": "Reach out to Sayee",
  "contact_id": "abb29721-d8c1-4a9f-a684-05c3ec7595ee"
}
```

**Sent to API:**
```json
{
  "timeline_event": {
    "note": "Reach out to Sayee",
    "event_time": "2025-10-21T20:22:26.286277300+00:00",
    "meeting_type": "note",
    "timeline_items_contacts": {
      "data": [
        {"contact_id": "abb29721-d8c1-4a9f-a684-05c3ec7595ee"}
      ]
    }
  }
}
```

## How to Deploy

1. **Rebuild the server:**
   ```bash
   cargo build --release
   ```

2. **Update your Claude Desktop config** with the new binary:
   ```json
   {
     "mcpServers": {
       "dex-mcp-server": {
         "command": "C:\\Users\\david\\code\\DexMCPServerRust\\target\\release\\dex-mcp-server.exe",
         "env": {
           "DEX_API_KEY": "your-api-key-here",
           "DEX_API_URL": "https://api.getdex.com/graphql",
           "RUST_LOG": "info"
         }
       }
     }
   }
   ```

3. **Restart Claude Desktop**

4. **Test creating a note or reminder**

## Debugging

If you still encounter issues:

1. Enable debug logging: `"RUST_LOG": "debug"`
2. Check the logs for the exact request being sent
3. Verify the response from the API
4. See `LOGGING_GUIDE.md` for detailed troubleshooting steps

## Files Modified

- `src/models/reminder.rs` - Rewrote CreateReminderRequest structure
- `src/models/note.rs` - Rewrote CreateNoteRequest structure
- `src/client/mod.rs` - Updated endpoint and added logging
- `src/server/handlers.rs` - Added logging to MCP handlers
- `Cargo.toml` - Added chrono dependency

## API Documentation References

- [Create Reminder API](https://getdex.com/docs/api-reference/reminders/post)
- [Create Note API](https://getdex.com/docs/api-reference/notes/post)

## Success Indicators

When working correctly, you'll see logs like:

```
INFO Creating note for contact: abb29721-d8c1-4a9f-a684-05c3ec7595ee
DEBUG Note request payload: { "timeline_event": { ... } }
DEBUG POST https://api.getdex.com/api/rest/timeline_items - Success (status: 201)
INFO Note created successfully: id=timeline_item_123
```

```
INFO Creating reminder for contact: abb29721-d8c1-4a9f-a684-05c3ec7595ee, due: 2025-10-22
DEBUG Reminder request payload: { "reminder": { ... } }
DEBUG POST https://api.getdex.com/api/rest/reminders - Success (status: 201)
INFO Reminder created successfully: id=reminder_456
```

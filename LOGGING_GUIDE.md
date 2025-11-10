# Logging Guide - Troubleshooting Notes and Reminders

## Overview

This guide explains how to enable verbose logging to troubleshoot issues with creating notes and reminders in the Dex MCP Server.

## Quick Start - Enable Debug Logging

### Option 1: Set Environment Variable (Recommended for Claude Desktop)

Before starting the MCP server, set the `RUST_LOG` environment variable:

**Windows (PowerShell):**
```powershell
$env:RUST_LOG = "debug"
```

**Windows (Command Prompt):**
```cmd
set RUST_LOG=debug
```

**macOS/Linux:**
```bash
export RUST_LOG=debug
```

### Option 2: Update Claude Desktop Config

Edit your Claude Desktop configuration file to include the environment variable:

**Location:**
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

**Add to the MCP server configuration:**
```json
{
  "mcpServers": {
    "dex-mcp-server": {
      "command": "C:\\path\\to\\dex-mcp-server.exe",
      "env": {
        "DEX_API_KEY": "your-api-key-here",
        "DEX_API_URL": "https://api.getdex.com/graphql",
        "RUST_LOG": "debug"
      }
    }
  }
}
```

### Option 3: More Granular Logging

For even more detailed logging, use the `trace` level:
```bash
export RUST_LOG=trace
```

Or target specific modules:
```bash
export RUST_LOG=dex_mcp_server=debug,dex_mcp_server::client=trace
```

## Log Levels

- **error**: Only errors (default)
- **warn**: Warnings and errors
- **info**: Informational messages, warnings, and errors
- **debug**: Debug information (recommended for troubleshooting)
- **trace**: Very detailed trace information

## What to Look For in Logs

When creating a note or reminder, you should see log entries like this:

### Successful Note Creation

```
INFO MCP Handler: add_contact_note called
DEBUG Parameters: contact_id=abb29721-d8c1-4a9f-a684-05c3ec7595ee, content_len=19, tags=None
DEBUG Calling enrichment_tools.add_contact_note
INFO Creating note for contact: abb29721-d8c1-4a9f-a684-05c3ec7595ee
DEBUG Note request payload: {
  "contacts": [
    {
      "contact_id": "abb29721-d8c1-4a9f-a684-05c3ec7595ee"
    }
  ],
  "note": "Reach out to Sayee"
}
DEBUG POST https://api.getdex.com/graphql/notes
DEBUG Request body: { ... }
DEBUG POST https://api.getdex.com/graphql/notes - Success (status: 200)
DEBUG Note creation response: { ... }
INFO Note created successfully: id=note_abc123
```

### Successful Reminder Creation

```
INFO MCP Handler: create_contact_reminder called
DEBUG Parameters: contact_id=abb29721-d8c1-4a9f-a684-05c3ec7595ee, note=Reach out to Sayee again, reminder_date=2025-10-22, reminder_type=None
DEBUG Calling enrichment_tools.create_contact_reminder
INFO Creating reminder for contact: abb29721-d8c1-4a9f-a684-05c3ec7595ee, due: 2025-10-22
DEBUG Reminder request payload: {
  "contact_ids": [
    {
      "contact_id": "abb29721-d8c1-4a9f-a684-05c3ec7595ee"
    }
  ],
  "body": "Reach out to Sayee again",
  "due_at_date": "2025-10-22"
}
DEBUG POST https://api.getdex.com/graphql/reminders
DEBUG Request body: { ... }
DEBUG POST https://api.getdex.com/graphql/reminders - Success (status: 200)
DEBUG Reminder creation response: { ... }
INFO Reminder created successfully: id=reminder_xyz789
```

### Failed Creation (What to Look For)

If creation fails, you'll see error messages like:

```
ERROR POST https://api.getdex.com/graphql/notes - Error: HttpError("400 Bad Request")
ERROR Failed to create note: HttpError("400 Bad Request")
```

Or:

```
ERROR POST https://api.getdex.com/graphql/reminders - Error: ApiError("Invalid contact_id")
ERROR Failed to create reminder: ApiError("Invalid contact_id")
```

## Common Issues and Solutions

### Issue 1: Missing contact_ids or contacts field

**Log shows:**
```json
{
  "note": "Reach out to Sayee"
}
```

**Problem:** The `contacts` field is missing from the request.

**Solution:** This should now be fixed with the `CreateNoteRequest` struct. If you still see this, the fix wasn't applied correctly.

### Issue 2: Invalid contact_id

**Log shows:**
```
ERROR Failed to create note: ApiError("Contact not found")
```

**Problem:** The contact ID doesn't exist in Dex.

**Solution:** Verify the contact exists by searching for them first.

### Issue 3: Authentication Error

**Log shows:**
```
ERROR POST ... - Error: HttpError("401 Unauthorized")
```

**Problem:** The API key is invalid or missing.

**Solution:** Check your `DEX_API_KEY` environment variable.

### Issue 4: Network/Connection Error

**Log shows:**
```
ERROR POST ... - Error: HttpError("Connection refused")
```

**Problem:** Can't connect to the Dex API.

**Solution:** Check your network connection and `DEX_API_URL`.

## Viewing Logs in Claude Desktop

On Windows, logs are written to stderr and can be viewed in:
- **Windows Event Viewer** (if configured)
- **PowerShell output** if running from command line
- **Log files** if you redirect stderr

To capture logs to a file when testing manually:
```bash
dex-mcp-server.exe 2> debug.log
```

## Disabling Debug Logging

Once you're done troubleshooting, disable debug logging for better performance:

```bash
unset RUST_LOG  # macOS/Linux
$env:RUST_LOG = ""  # PowerShell
```

Or remove the `RUST_LOG` entry from your Claude Desktop config.

## Need More Help?

If you're still seeing issues:

1. Enable `trace` level logging: `RUST_LOG=trace`
2. Capture the full log output
3. Look for the request payload being sent
4. Check the HTTP response status and body
5. Compare with the expected format in this guide

The logs will show exactly what's being sent to the API and what the API is responding with.

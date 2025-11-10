//! MCP tool handlers for Dex server.
//!
//! This module implements all the MCP tools using the rmcp SDK's tool_router pattern.

use crate::client::AsyncDexClient;
use crate::repositories::{ContactRepository, NoteRepository, ReminderRepository};
use crate::tools::{
    ContactDiscoveryTools, ContactEnrichmentTools, RelationshipHistoryTools, SearchTools,
};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The Dex MCP server that exposes tools for interacting with Dex CRM.
#[derive(Clone)]
pub struct DexMcpServer {
    // Services provide business logic
    contact_service: Arc<dyn crate::services::ContactService>,
    note_service: Arc<dyn crate::services::NoteService>,
    reminder_service: Arc<dyn crate::services::ReminderService>,
    history_service: Arc<dyn crate::services::HistoryService>,
    #[allow(dead_code)] // Reserved for future direct API calls if needed
    client: Arc<dyn AsyncDexClient>,
    tool_router: ToolRouter<Self>,
}

// Implement ServerHandler using the tool_handler macro
#[tool_handler]
impl ServerHandler for DexMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities {
                tools: Some(Default::default()),
                ..Default::default()
            },
            server_info: Implementation {
                name: "dex-mcp-server".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: Some("MCP server for Dex Personal CRM - provides contact discovery, relationship history, and contact enrichment capabilities.".into()),
        }
    }
}

// Helper structs for tool parameters
#[derive(Debug, Deserialize, JsonSchema)]
struct SearchContactsParams {
    query: String,
    #[serde(default)]
    max_results: Option<usize>,
    #[serde(default)]
    min_confidence: Option<u8>,
    #[serde(default)]
    #[allow(dead_code)]
    include_types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FindContactToolParams {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    social_url: Option<String>,
    #[serde(default)]
    company: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ContactIdParams {
    contact_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetContactHistoryParams {
    contact_id: String,
    #[serde(default)]
    include_notes: Option<bool>,
    #[serde(default)]
    include_reminders: Option<bool>,
    #[serde(default)]
    date_from: Option<String>,
    #[serde(default)]
    date_to: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetContactNotesParams {
    contact_id: String,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    date_from: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetContactRemindersParams {
    contact_id: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    date_from: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EnrichContactToolParams {
    contact_id: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    social_profiles: Option<Vec<String>>,
    #[serde(default)]
    company: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AddContactNoteToolParams {
    contact_id: String,
    content: String,
    #[serde(default)]
    #[allow(dead_code)]
    date: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateContactReminderToolParams {
    contact_id: String,
    reminder_date: String,
    note: String,
    #[serde(default)]
    reminder_type: Option<String>,
}

// Helper function to convert errors to MCP errors
fn to_mcp_error(e: impl std::fmt::Display) -> McpError {
    McpError {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::from(e.to_string()),
        data: None,
    }
}

// Tool router implementation
#[tool_router]
impl DexMcpServer {
    /// Create a new Dex MCP server.
    pub fn new(
        contact_repo: Arc<dyn ContactRepository>,
        note_repo: Arc<dyn NoteRepository>,
        reminder_repo: Arc<dyn ReminderRepository>,
        client: Arc<dyn AsyncDexClient>,
        discovery_cache_ttl_secs: u64,
        search_cache_ttl_secs: u64,
    ) -> Self {
        // Construct all tools with repository dependencies
        let discovery_tools = Arc::new(RwLock::new(ContactDiscoveryTools::new(
            contact_repo.clone(),
            discovery_cache_ttl_secs,
        )));

        let history_tools = Arc::new(RelationshipHistoryTools::new(
            contact_repo.clone(),
            note_repo.clone(),
            reminder_repo.clone(),
        ));

        let enrichment_tools = Arc::new(ContactEnrichmentTools::new(
            contact_repo.clone(),
            note_repo.clone(),
            reminder_repo.clone(),
        ));

        let search_tools = SearchTools::new(
            contact_repo,
            note_repo,
            reminder_repo,
            search_cache_ttl_secs,
        );

        // Construct services from tools
        let contact_service = Arc::new(crate::services::ContactServiceImpl::new(
            discovery_tools,
            enrichment_tools.clone(),
            search_tools,
        )) as Arc<dyn crate::services::ContactService>;

        let note_service = Arc::new(crate::services::NoteServiceImpl::new(
            history_tools.clone(),
            enrichment_tools.clone(),
        )) as Arc<dyn crate::services::NoteService>;

        let reminder_service = Arc::new(crate::services::ReminderServiceImpl::new(
            history_tools.clone(),
            enrichment_tools,
        )) as Arc<dyn crate::services::ReminderService>;

        let history_service = Arc::new(crate::services::HistoryServiceImpl::new(history_tools))
            as Arc<dyn crate::services::HistoryService>;

        Self {
            contact_service,
            note_service,
            reminder_service,
            history_service,
            client,
            tool_router: Self::tool_router(),
        }
    }

    /// Search across all contact data including names, descriptions, notes, and reminders.
    #[tool(
        description = "Search across all contact data including names, descriptions, notes, and reminders using fuzzy matching. Returns ranked results with match context showing where the query was found."
    )]
    async fn search_contacts_full_text(
        &self,
        params: Parameters<SearchContactsParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        // Use ContactService with validation
        let response = self
            .contact_service
            .search_full_text(
                params.query.clone(),
                params.max_results,
                params.min_confidence,
            )
            .await
            .map_err(to_mcp_error)?;

        let results = response.results;

        // Format results as JSON
        let response = serde_json::json!({
            "query": params.query,
            "result_count": results.len(),
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "contact": {
                        "id": r.contact.id,
                        "name": format!("{} {}",
                            r.contact.first_name.as_deref().unwrap_or(""),
                            r.contact.last_name.as_deref().unwrap_or("")
                        ).trim(),
                        "email": r.contact.email.as_deref().unwrap_or(""),
                        "company": r.contact.company.as_deref().unwrap_or(""),
                    },
                    "confidence": r.confidence,
                    "matches": r.matches.iter().map(|mc| {
                        serde_json::json!({
                            "found_in": mc.field_type.display_name(),
                            "field": mc.field_type.display_name(),
                            "excerpt": mc.snippet,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).map_err(to_mcp_error)?,
        )]))
    }

    /// Find contacts using smart matching with fuzzy name search or exact matches.
    #[tool(
        description = "Find contacts using smart matching with fuzzy name search or exact matches on email/phone/social URLs. Returns top matches with confidence scores."
    )]
    async fn find_contact(
        &self,
        params: Parameters<FindContactToolParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        let response = self
            .contact_service
            .find_contact(
                params.name,
                params.email,
                params.phone,
                params.social_url,
                params.company,
            )
            .await
            .map_err(to_mcp_error)?;

        let json_response = serde_json::to_string_pretty(&serde_json::json!({
            "matches": response.matches.iter().map(|m| {
                serde_json::json!({
                    "contact": {
                        "id": m.contact.id,
                        "name": format!("{} {}",
                            m.contact.first_name.as_deref().unwrap_or(""),
                            m.contact.last_name.as_deref().unwrap_or("")
                        ).trim(),
                        "email": m.contact.email.as_deref().unwrap_or(""),
                        "phone": m.contact.phone.as_deref().unwrap_or(""),
                        "company": m.contact.company.as_deref().unwrap_or(""),
                    },
                    "confidence": m.confidence,
                    "match_type": format!("{:?}", m.match_type),
                })
            }).collect::<Vec<_>>(),
            "from_cache": response.from_cache,
        }))
        .map_err(to_mcp_error)?;

        Ok(CallToolResult::success(vec![Content::text(json_response)]))
    }

    /// Retrieve complete information for a specific contact by ID.
    #[tool(description = "Retrieve complete information for a specific contact by ID")]
    async fn get_contact_details(
        &self,
        params: Parameters<ContactIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        let contact = self
            .contact_service
            .get_contact_details(&params.contact_id)
            .await
            .map_err(to_mcp_error)?;

        let json_response = serde_json::to_string_pretty(&contact).map_err(to_mcp_error)?;

        Ok(CallToolResult::success(vec![Content::text(json_response)]))
    }

    /// Get the complete relationship timeline for a contact.
    #[tool(
        description = "Get the complete relationship timeline for a contact, including all notes and reminders in chronological order"
    )]
    async fn get_contact_history(
        &self,
        params: Parameters<GetContactHistoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        let history = self
            .history_service
            .get_contact_history(
                &params.contact_id,
                params.date_from,
                params.date_to,
                params.include_notes.unwrap_or(true),
                params.include_reminders.unwrap_or(true),
            )
            .await
            .map_err(to_mcp_error)?;

        let json_response = serde_json::to_string_pretty(&serde_json::json!({
            "contact": history.contact,
            "timeline": history.timeline.iter().map(|entry| {
                match entry {
                    crate::tools::TimelineEntry::Note(note) => {
                        serde_json::json!({
                            "type": "note",
                            "id": note.id,
                            "content": note.content,
                            "created_at": note.created_at,
                            "tags": note.tags,
                        })
                    },
                    crate::tools::TimelineEntry::Reminder(reminder) => {
                        serde_json::json!({
                            "type": "reminder",
                            "id": reminder.id,
                            "text": reminder.text,
                            "due_date": reminder.due_date,
                            "completed": reminder.completed,
                            "created_at": reminder.created_at,
                            "tags": reminder.tags,
                        })
                    }
                }
            }).collect::<Vec<_>>(),
            "total_entries": history.total_entries,
        }))
        .map_err(to_mcp_error)?;

        Ok(CallToolResult::success(vec![Content::text(json_response)]))
    }

    /// Get all notes for a specific contact.
    #[tool(
        description = "Get all notes for a specific contact, sorted by date (most recent first)"
    )]
    async fn get_contact_notes(
        &self,
        params: Parameters<GetContactNotesParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        let notes = self
            .note_service
            .get_contact_notes(&params.contact_id, params.date_from, params.limit)
            .await
            .map_err(to_mcp_error)?;

        let json_response = serde_json::to_string_pretty(&notes).map_err(to_mcp_error)?;

        Ok(CallToolResult::success(vec![Content::text(json_response)]))
    }

    /// Get all reminders for a specific contact.
    #[tool(description = "Get all reminders for a specific contact")]
    async fn get_contact_reminders(
        &self,
        params: Parameters<GetContactRemindersParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        // Convert status string to ReminderStatus
        let status = params.status.as_ref().map(|s| {
            s.parse::<crate::services::ReminderStatus>()
                .unwrap_or(crate::services::ReminderStatus::All)
        });

        let reminders = self
            .reminder_service
            .get_contact_reminders(&params.contact_id, params.date_from, status)
            .await
            .map_err(to_mcp_error)?;

        let json_response = serde_json::to_string_pretty(&reminders).map_err(to_mcp_error)?;

        Ok(CallToolResult::success(vec![Content::text(json_response)]))
    }

    /// Add or update information for an existing contact.
    #[tool(
        description = "Add or update information for an existing contact. Intelligently merges new data without overwriting existing information."
    )]
    async fn enrich_contact(
        &self,
        params: Parameters<EnrichContactToolParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        let enrich_params = crate::services::ContactEnrichParams {
            contact_id: params.contact_id.clone(),
            email: params.email,
            phone: params.phone,
            company: params.company,
            title: params.title,
            notes: params.notes,
            tags: params.tags,
            social_profiles: params.social_profiles,
        };

        let updated_contact = self
            .contact_service
            .enrich_contact(enrich_params)
            .await
            .map_err(to_mcp_error)?;

        let json_response = serde_json::to_string_pretty(&updated_contact).map_err(to_mcp_error)?;

        Ok(CallToolResult::success(vec![Content::text(json_response)]))
    }

    /// Create a new note for a contact.
    #[tool(
        description = "Create a new note for a contact to track interactions and important information"
    )]
    async fn add_contact_note(
        &self,
        params: Parameters<AddContactNoteToolParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        tracing::info!("MCP Handler: add_contact_note called");
        tracing::debug!(
            "Parameters: contact_id={}, content_len={}, tags={:?}",
            params.contact_id,
            params.content.len(),
            params.tags
        );

        let note = self
            .note_service
            .create_note(
                params.contact_id.clone(),
                params.content.clone(),
                params.tags.clone(),
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to create note: {:?}", e);
                to_mcp_error(e)
            })?;

        tracing::info!("Note created successfully: id={}", note.id);
        let json_response = serde_json::to_string_pretty(&note).map_err(to_mcp_error)?;

        Ok(CallToolResult::success(vec![Content::text(json_response)]))
    }

    /// Set a reminder for future follow-up with a contact.
    #[tool(description = "Set a reminder for future follow-up with a contact")]
    async fn create_contact_reminder(
        &self,
        params: Parameters<CreateContactReminderToolParams>,
    ) -> Result<CallToolResult, McpError> {
        let params = params.0;

        tracing::info!("MCP Handler: create_contact_reminder called");
        tracing::debug!(
            "Parameters: contact_id={}, note={}, reminder_date={}, reminder_type={:?}",
            params.contact_id,
            params.note,
            params.reminder_date,
            params.reminder_type
        );

        let reminder = self
            .reminder_service
            .create_reminder(
                params.contact_id.clone(),
                params.note.clone(),
                params.reminder_date.clone(),
                params.reminder_type.clone(),
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to create reminder: {:?}", e);
                to_mcp_error(e)
            })?;

        tracing::info!("Reminder created successfully: id={}", reminder.id);
        let json_response = serde_json::to_string_pretty(&reminder).map_err(to_mcp_error)?;

        Ok(CallToolResult::success(vec![Content::text(json_response)]))
    }
}

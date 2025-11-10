//! Application service layer.
//!
//! Services contain business logic and orchestrate interactions between
//! repositories and tools. They provide a clean boundary between the
//! MCP handlers and the data access layer.

mod contact_service;
mod history_service;
mod note_service;
mod reminder_service;

pub use contact_service::{ContactEnrichParams, ContactService, ContactServiceImpl};
pub use history_service::{HistoryService, HistoryServiceImpl};
pub use note_service::{NoteService, NoteServiceImpl};
pub use reminder_service::{ReminderService, ReminderServiceImpl, ReminderStatus};

// Re-export common types used by services
pub use crate::models::{Contact, Note, Reminder};
pub use crate::tools::{
    ContactDiscoveryTools, ContactEnrichmentTools, EnrichContactParams, CreateNoteParams,
    CreateReminderParams, FindContactParams, FindContactResponse, RelationshipHistoryTools,
    SearchTools, HistoryFilterParams, ContactHistoryResponse,
};

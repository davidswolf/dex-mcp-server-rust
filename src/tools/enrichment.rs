//! Contact enrichment tools.
//!
//! Provides tools for updating contacts, adding notes, and creating reminders
//! with smart data merging.

use crate::error::DexApiResult;
use crate::models::{Contact, Note, Reminder, SocialProfile};
use crate::repositories::{ContactRepository, NoteRepository, ReminderRepository};
use std::sync::Arc;

/// Contact enrichment tools for updating contact data.
pub struct ContactEnrichmentTools {
    contact_repo: Arc<dyn ContactRepository>,
    note_repo: Arc<dyn NoteRepository>,
    reminder_repo: Arc<dyn ReminderRepository>,
}

/// Parameters for enriching a contact.
#[derive(Debug, Clone, Default)]
pub struct EnrichContactParams {
    /// Contact ID to enrich
    pub contact_id: String,

    /// New or updated fields
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub birthday: Option<String>,
    pub notes: Option<String>,

    /// Additional emails (will be merged with existing)
    pub additional_emails: Option<Vec<String>>,

    /// Additional phones (will be merged with existing)
    pub additional_phones: Option<Vec<String>>,

    /// Tags (will be merged with existing)
    pub tags: Option<Vec<String>>,

    /// Social profiles (will be merged with existing)
    pub social_profiles: Option<Vec<SocialProfile>>,
}

/// Parameters for creating a note.
#[derive(Debug, Clone)]
pub struct CreateNoteParams {
    /// Contact ID
    pub contact_id: String,

    /// Note content (may include HTML)
    pub content: String,

    /// Optional tags
    pub tags: Option<Vec<String>>,

    /// Optional source (e.g., "manual", "email", "meeting")
    pub source: Option<String>,
}

/// Parameters for creating a reminder.
#[derive(Debug, Clone)]
pub struct CreateReminderParams {
    /// Contact ID
    pub contact_id: String,

    /// Reminder text
    pub text: String,

    /// Due date (ISO 8601 format)
    pub due_date: String,

    /// Optional tags
    pub tags: Option<Vec<String>>,

    /// Optional priority (e.g., "high", "medium", "low")
    pub priority: Option<String>,
}

impl ContactEnrichmentTools {
    /// Create new contact enrichment tools.
    ///
    /// # Arguments
    /// * `contact_repo` - ContactRepository for contact data access
    /// * `note_repo` - NoteRepository for note data access
    /// * `reminder_repo` - ReminderRepository for reminder data access
    pub fn new(
        contact_repo: Arc<dyn ContactRepository>,
        note_repo: Arc<dyn NoteRepository>,
        reminder_repo: Arc<dyn ReminderRepository>,
    ) -> Self {
        Self {
            contact_repo,
            note_repo,
            reminder_repo,
        }
    }

    /// Enrich a contact with new data.
    ///
    /// This method performs smart merging:
    /// - Simple fields: Overwrites if new value provided
    /// - Array fields (emails, phones, tags, social profiles): Merges with existing,
    ///   removing duplicates
    ///
    /// # Arguments
    /// * `params` - Enrichment parameters
    ///
    /// # Returns
    /// The updated contact
    pub async fn enrich_contact(&self, params: EnrichContactParams) -> DexApiResult<Contact> {
        // Fetch existing contact
        let mut contact = self.contact_repo.get(&params.contact_id).await?;

        // Update simple fields
        if let Some(first_name) = params.first_name {
            contact.first_name = Some(first_name);
        }
        if let Some(last_name) = params.last_name {
            contact.last_name = Some(last_name);
        }
        if let Some(email) = params.email {
            contact.email = Some(email);
        }
        if let Some(phone) = params.phone {
            contact.phone = Some(phone);
        }
        if let Some(company) = params.company {
            contact.company = Some(company);
        }
        if let Some(title) = params.title {
            contact.title = Some(title);
        }
        if let Some(website) = params.website {
            contact.website = Some(website);
        }
        if let Some(location) = params.location {
            contact.location = Some(location);
        }
        if let Some(birthday) = params.birthday {
            contact.birthday = Some(birthday);
        }
        if let Some(notes) = params.notes {
            contact.notes = Some(notes);
        }

        // Merge array fields (remove duplicates)
        if let Some(mut additional_emails) = params.additional_emails {
            contact.emails.append(&mut additional_emails);
            contact.emails.sort();
            contact.emails.dedup();
        }

        if let Some(mut additional_phones) = params.additional_phones {
            contact.phones.append(&mut additional_phones);
            contact.phones.sort();
            contact.phones.dedup();
        }

        if let Some(mut tags) = params.tags {
            contact.tags.append(&mut tags);
            contact.tags.sort();
            contact.tags.dedup();
        }

        if let Some(mut social_profiles) = params.social_profiles {
            // Merge social profiles by URL (avoid duplicates)
            for new_profile in social_profiles.drain(..) {
                if !contact
                    .social_profiles
                    .iter()
                    .any(|p| p.url == new_profile.url)
                {
                    contact.social_profiles.push(new_profile);
                }
            }
        }

        // Update contact via repository
        self.contact_repo.update(&params.contact_id, &contact).await
    }

    /// Add a note to a contact.
    ///
    /// # Arguments
    /// * `params` - Note creation parameters
    ///
    /// # Returns
    /// The created note
    pub async fn add_contact_note(&self, params: CreateNoteParams) -> DexApiResult<Note> {
        // Build note object
        let note = Note {
            id: String::new(), // Will be assigned by the API
            contact_id: params.contact_id,
            content: params.content,
            created_at: String::new(), // Will be assigned by the API
            updated_at: None,
            tags: params.tags.unwrap_or_default(),
            source: params.source,
        };

        self.note_repo.create(&note).await
    }

    /// Create a reminder for a contact.
    ///
    /// # Arguments
    /// * `params` - Reminder creation parameters
    ///
    /// # Returns
    /// The created reminder
    pub async fn create_contact_reminder(&self, params: CreateReminderParams) -> DexApiResult<Reminder> {
        // Build reminder object
        let reminder = Reminder {
            id: String::new(), // Will be assigned by the API
            contact_id: params.contact_id,
            text: params.text,
            due_date: params.due_date,
            completed: false,
            completed_at: None,
            created_at: String::new(), // Will be assigned by the API
            updated_at: None,
            tags: params.tags.unwrap_or_default(),
            priority: params.priority,
        };

        self.reminder_repo.create(&reminder).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::client::{AsyncDexClient, AsyncDexClientImpl, DexClient};
    use crate::repositories::{DexContactRepository, DexNoteRepository, DexReminderRepository};

    #[test]
    fn test_contact_enrichment_tools_creation() {
        let config = Config::default();
        let sync_client = DexClient::new(&config);
        let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

        let contact_repo = Arc::new(DexContactRepository::new(client.clone()));
        let note_repo = Arc::new(DexNoteRepository::new(client.clone()));
        let reminder_repo = Arc::new(DexReminderRepository::new(client));

        let _tools = ContactEnrichmentTools::new(contact_repo, note_repo, reminder_repo);
        // Just verify it constructs without panic
    }

    #[test]
    fn test_enrich_contact_params_default() {
        let params = EnrichContactParams {
            contact_id: "test123".to_string(),
            ..Default::default()
        };
        assert_eq!(params.contact_id, "test123");
        assert!(params.first_name.is_none());
        assert!(params.email.is_none());
    }

    #[test]
    fn test_create_note_params() {
        let params = CreateNoteParams {
            contact_id: "contact1".to_string(),
            content: "Test note".to_string(),
            tags: Some(vec!["meeting".to_string()]),
            source: Some("manual".to_string()),
        };
        assert_eq!(params.contact_id, "contact1");
        assert_eq!(params.content, "Test note");
        assert_eq!(params.tags.unwrap()[0], "meeting");
    }

    #[test]
    fn test_create_reminder_params() {
        let params = CreateReminderParams {
            contact_id: "contact1".to_string(),
            text: "Follow up".to_string(),
            due_date: "2024-02-01T10:00:00Z".to_string(),
            tags: Some(vec!["important".to_string()]),
            priority: Some("high".to_string()),
        };
        assert_eq!(params.contact_id, "contact1");
        assert_eq!(params.text, "Follow up");
        assert_eq!(params.priority.unwrap(), "high");
    }

    // Note: More comprehensive tests for enrich_contact, add_contact_note,
    // and create_contact_reminder would require mocking the DexClient
    // These integration tests should be in tests/ directory with mockito
}

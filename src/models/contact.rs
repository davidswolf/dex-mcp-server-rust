//! Contact model representing a person in Dex Personal CRM.

use serde::{Deserialize, Deserializer, Serialize};
use std::sync::Arc;

/// Shared reference to a Contact for memory-efficient storage.
///
/// This type is used throughout the application to avoid unnecessary cloning
/// of Contact data, particularly in caches and search results where the same
/// contact may be referenced multiple times.
///
/// ⚠️ **IMPORTANT:** Do not create reference cycles by adding `Arc<Contact>` fields
/// inside Contact. Use `Weak<Contact>` if bidirectional references are needed.
pub type ContactRef = Arc<Contact>;

/// Email address entry for a contact (from API).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(default)]
pub struct EmailEntry {
    /// The email address
    pub email: String,
}

/// Phone number entry for a contact (from API).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(default)]
pub struct PhoneEntry {
    /// The phone number
    #[serde(rename = "phone_number")]
    pub phone: String,
}

/// Custom deserializer for emails that converts from API format to Vec<String>
fn deserialize_emails<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let entries: Vec<EmailEntry> = Vec::deserialize(deserializer)?;
    Ok(entries.into_iter().map(|e| e.email).collect())
}

/// Custom deserializer for phones that converts from API format to Vec<String>
fn deserialize_phones<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let entries: Vec<PhoneEntry> = Vec::deserialize(deserializer)?;
    Ok(entries.into_iter().map(|e| e.phone).collect())
}

/// A contact in the Dex Personal CRM system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Contact {
    /// Unique identifier for the contact
    pub id: String,

    /// Full name of the contact (computed from first/last name)
    #[serde(skip_serializing, default = "String::new")]
    pub name: String,

    /// First name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// Last name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// Primary email address (computed from emails vec)
    #[serde(skip_serializing, default)]
    pub email: Option<String>,

    /// Email addresses (from API as array of {email: string} objects, deserialized to strings)
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_emails"
    )]
    pub emails: Vec<String>,

    /// Primary phone number (computed from phones vec)
    #[serde(skip_serializing, default)]
    pub phone: Option<String>,

    /// Phone numbers (from API as array of {phone: string} objects, deserialized to strings)
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_phones"
    )]
    pub phones: Vec<String>,

    /// Job title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_title: Option<String>,

    /// Job title alias for compatibility
    #[serde(skip_serializing, default, rename = "title")]
    pub title: Option<String>,

    /// Company/organization (not in API currently)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,

    /// Description/notes about the contact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Tags associated with the contact
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Social media profiles
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub social_profiles: Vec<SocialProfile>,

    /// Education information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education: Option<String>,

    /// Website URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Profile image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,

    /// LinkedIn username/profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin: Option<String>,

    /// Facebook username/profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facebook: Option<String>,

    /// Twitter username/profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter: Option<String>,

    /// Instagram username/profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instagram: Option<String>,

    /// Telegram username/profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telegram: Option<String>,

    /// Birthday (API field: birthday_current_year)
    #[serde(skip_serializing_if = "Option::is_none", rename = "birthday_current_year")]
    pub birthday: Option<String>,

    /// Location/address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Custom notes (different from description)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Last time contact was seen/interacted with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_at: Option<String>,

    /// Next reminder date/time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_reminder_at: Option<String>,

    /// Whether the contact is archived
    #[serde(default)]
    pub is_archived: bool,

    /// When the contact was created (ISO 8601 timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// When the contact was last updated (ISO 8601 timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// A social media profile associated with a contact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SocialProfile {
    /// Type of social media (e.g., "twitter", "linkedin", "github")
    #[serde(rename = "type")]
    pub profile_type: String,

    /// Username or handle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Full URL to the profile
    pub url: String,
}

impl Contact {
    /// Create a new contact with minimal required fields.
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            first_name: None,
            last_name: None,
            email: None,
            emails: Vec::new(),
            phone: None,
            phones: Vec::new(),
            job_title: None,
            title: None,
            company: None,
            description: None,
            tags: Vec::new(),
            social_profiles: Vec::new(),
            education: None,
            website: None,
            image_url: None,
            linkedin: None,
            facebook: None,
            twitter: None,
            instagram: None,
            telegram: None,
            birthday: None,
            location: None,
            notes: None,
            last_seen_at: None,
            next_reminder_at: None,
            is_archived: false,
            created_at: None,
            updated_at: None,
        }
    }

    /// Populate computed fields from API data after deserialization.
    pub fn populate_computed_fields(&mut self) {
        // Populate name from first_name and last_name
        self.name = match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => String::new(),
        };

        // Populate email from emails vec
        self.email = self.emails.first().cloned();

        // Populate phone from phones vec
        self.phone = self.phones.first().cloned();

        // Populate title from job_title if not set
        if self.title.is_none() {
            self.title = self.job_title.clone();
        }
    }

    /// Get all email addresses for this contact.
    pub fn all_emails(&self) -> Vec<String> {
        let mut emails = Vec::new();
        if let Some(ref email) = self.email {
            emails.push(email.clone());
        }
        emails.extend(self.emails.iter().cloned());
        emails.dedup();
        emails
    }

    /// Get all phone numbers for this contact.
    pub fn all_phones(&self) -> Vec<String> {
        let mut phones = Vec::new();
        if let Some(ref phone) = self.phone {
            phones.push(phone.clone());
        }
        phones.extend(self.phones.iter().cloned());
        phones.dedup();
        phones
    }
}

impl Default for Contact {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

impl SocialProfile {
    /// Create a new social profile.
    pub fn new(profile_type: String, url: String) -> Self {
        Self {
            profile_type,
            username: None,
            url,
        }
    }
}


/// Inner contact payload for contact creation matching Dex API structure
#[derive(Debug, Clone, Serialize)]
struct ContactPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    job_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact_emails: Option<ContactEmailsData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact_phone_numbers: Option<ContactPhonesData>,
}

/// Wrapper for contact emails data
#[derive(Debug, Clone, Serialize)]
struct ContactEmailsData {
    data: Vec<EmailEntry>,
}

/// Wrapper for contact phone numbers data
#[derive(Debug, Clone, Serialize)]
struct ContactPhonesData {
    data: Vec<PhoneEntryWithLabel>,
}

/// Phone entry with label for API
#[derive(Debug, Clone, Serialize)]
struct PhoneEntryWithLabel {
    phone_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

/// Request payload for creating a new contact.
/// This matches the Dex API structure: { "contact": { ... } }
#[derive(Debug, Clone, Serialize)]
pub struct CreateContactRequest {
    contact: ContactPayload,
}

impl From<&Contact> for CreateContactRequest {
    fn from(contact: &Contact) -> Self {
        let contact_emails = if !contact.emails.is_empty() {
            Some(ContactEmailsData {
                data: contact.emails.iter().map(|email| EmailEntry {
                    email: email.clone(),
                }).collect(),
            })
        } else {
            None
        };

        let contact_phone_numbers = if !contact.phones.is_empty() {
            Some(ContactPhonesData {
                data: contact.phones.iter().map(|phone| PhoneEntryWithLabel {
                    phone_number: phone.clone(),
                    label: Some("mobile".to_string()),
                }).collect(),
            })
        } else {
            None
        };

        Self {
            contact: ContactPayload {
                first_name: contact.first_name.clone(),
                last_name: contact.last_name.clone(),
                job_title: contact.job_title.clone(),
                description: contact.description.clone(),
                contact_emails,
                contact_phone_numbers,
            },
        }
    }
}

/// Changes object for updating a contact
#[derive(Debug, Clone, Serialize)]
struct ContactChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    job_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

/// Request payload for updating a contact.
/// This matches the Dex API structure: { "changes": { ... } }
#[derive(Debug, Clone, Serialize)]
pub struct UpdateContactRequest {
    changes: ContactChanges,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact_emails: Option<Vec<ContactEmailUpdate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact_phone_numbers: Option<Vec<ContactPhoneUpdate>>,
    #[serde(skip_serializing_if = "is_false")]
    update_contact_emails: bool,
    #[serde(skip_serializing_if = "is_false")]
    update_contact_phone_numbers: bool,
}

/// Email update entry
#[derive(Debug, Clone, Serialize)]
struct ContactEmailUpdate {
    contact_id: String,
    email: String,
}

/// Phone update entry
#[derive(Debug, Clone, Serialize)]
struct ContactPhoneUpdate {
    contact_id: String,
    phone_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

/// Helper function for serde skip_serializing_if
fn is_false(b: &bool) -> bool {
    !b
}

impl UpdateContactRequest {
    pub fn from_contact(contact: &Contact, contact_id: &str) -> Self {
        let contact_emails = if !contact.emails.is_empty() {
            Some(contact.emails.iter().map(|email| ContactEmailUpdate {
                contact_id: contact_id.to_string(),
                email: email.clone(),
            }).collect())
        } else {
            None
        };

        let contact_phone_numbers = if !contact.phones.is_empty() {
            Some(contact.phones.iter().map(|phone| ContactPhoneUpdate {
                contact_id: contact_id.to_string(),
                phone_number: phone.clone(),
                label: Some("mobile".to_string()),
            }).collect())
        } else {
            None
        };

        let update_emails = contact_emails.is_some();
        let update_phones = contact_phone_numbers.is_some();

        Self {
            changes: ContactChanges {
                first_name: contact.first_name.clone(),
                last_name: contact.last_name.clone(),
                job_title: contact.job_title.clone(),
                description: contact.description.clone(),
            },
            contact_emails,
            contact_phone_numbers,
            update_contact_emails: update_emails,
            update_contact_phone_numbers: update_phones,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_new() {
        let contact = Contact::new("123".to_string(), "John Doe".to_string());
        assert_eq!(contact.id, "123");
        assert_eq!(contact.name, "John Doe");
        assert!(contact.email.is_none());
    }

    #[test]
    fn test_contact_all_emails() {
        let mut contact = Contact::new("123".to_string(), "John Doe".to_string());
        contact.email = Some("john@example.com".to_string());
        contact.emails = vec!["john.doe@work.com".to_string()];

        let all_emails = contact.all_emails();
        assert_eq!(all_emails.len(), 2);
        assert!(all_emails.contains(&"john@example.com".to_string()));
        assert!(all_emails.contains(&"john.doe@work.com".to_string()));
    }

    #[test]
    fn test_contact_all_phones() {
        let mut contact = Contact::new("123".to_string(), "John Doe".to_string());
        contact.phone = Some("+1234567890".to_string());
        contact.phones = vec!["+0987654321".to_string()];

        let all_phones = contact.all_phones();
        assert_eq!(all_phones.len(), 2);
    }

    #[test]
    fn test_contact_serialization() {
        let contact = Contact::new("123".to_string(), "John Doe".to_string());
        let json = serde_json::to_string(&contact).unwrap();
        assert!(json.contains("\"id\":\"123\""));
        // Note: 'name' field is marked as skip_serializing, so it won't appear in JSON
        // It's a computed field derived from first_name and last_name
    }

    #[test]
    fn test_contact_deserialization() {
        let json = r#"{"id":"123","first_name":"John","last_name":"Doe","emails":[{"email":"john@example.com"}]}"#;
        let mut contact: Contact = serde_json::from_str(json).unwrap();
        assert_eq!(contact.id, "123");
        contact.populate_computed_fields();
        assert_eq!(contact.name, "John Doe");
        assert_eq!(contact.email, Some("john@example.com".to_string()));
    }

    #[test]
    fn test_social_profile() {
        let profile = SocialProfile::new(
            "twitter".to_string(),
            "https://twitter.com/johndoe".to_string(),
        );
        assert_eq!(profile.profile_type, "twitter");
        assert_eq!(profile.url, "https://twitter.com/johndoe");
    }
}

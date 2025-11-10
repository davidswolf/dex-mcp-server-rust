//! ContactId value object.

use super::errors::ValidationError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A type-safe wrapper for contact IDs.
///
/// This ensures that contact IDs are validated at construction time
/// and cannot be empty.
///
/// # Example
///
/// ```
/// use dex_mcp_server::domain::ContactId;
///
/// let id = ContactId::new("contact_123").unwrap();
/// assert_eq!(id.as_str(), "contact_123");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContactId(String);

impl ContactId {
    /// Create a new ContactId, validating that it's not empty.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::EmptyId` if the provided ID is empty.
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ValidationError::EmptyId);
        }
        Ok(Self(id))
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the underlying String.
    pub fn into_inner(self) -> String {
        self.0
    }
}

// Serde support - serialize as string
impl Serialize for ContactId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

// Serde support - deserialize from string with validation
impl<'de> Deserialize<'de> for ContactId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ContactId::new(s).map_err(serde::de::Error::custom)
    }
}

// Display support
impl fmt::Display for ContactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_id_valid() {
        let id = ContactId::new("contact_123").unwrap();
        assert_eq!(id.as_str(), "contact_123");
    }

    #[test]
    fn test_contact_id_rejects_empty() {
        assert!(ContactId::new("").is_err());
    }

    #[test]
    fn test_contact_id_display() {
        let id = ContactId::new("contact_123").unwrap();
        assert_eq!(format!("{}", id), "contact_123");
    }

    #[test]
    fn test_contact_id_serialization() {
        let id = ContactId::new("contact_123").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"contact_123\"");
    }

    #[test]
    fn test_contact_id_deserialization() {
        let id: ContactId = serde_json::from_str("\"contact_123\"").unwrap();
        assert_eq!(id.as_str(), "contact_123");
    }

    #[test]
    fn test_contact_id_deserialization_empty_fails() {
        let result: Result<ContactId, _> = serde_json::from_str("\"\"");
        assert!(result.is_err());
    }
}

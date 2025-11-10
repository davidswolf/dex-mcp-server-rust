//! EmailAddress value object.

use super::errors::ValidationError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A type-safe wrapper for email addresses.
///
/// This ensures that email addresses are validated at construction time.
/// The validation is basic and checks for the presence of '@' and a domain part.
///
/// # Example
///
/// ```
/// use dex_mcp_server::domain::EmailAddress;
///
/// let email = EmailAddress::new("user@example.com").unwrap();
/// assert_eq!(email.as_str(), "user@example.com");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Create a new EmailAddress, validating the format.
    ///
    /// # Validation Rules
    ///
    /// - Must contain exactly one '@' symbol
    /// - Must have a local part before '@'
    /// - Must have a domain part after '@' with at least one '.'
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::InvalidEmail` if the email format is invalid.
    pub fn new(email: impl Into<String>) -> Result<Self, ValidationError> {
        let email = email.into();

        if !Self::is_valid(&email) {
            return Err(ValidationError::InvalidEmail(email));
        }

        Ok(Self(email))
    }

    /// Validate email format.
    fn is_valid(email: &str) -> bool {
        // Basic validation: check for '@' and a domain part
        let parts: Vec<&str> = email.split('@').collect();

        if parts.len() != 2 {
            return false;
        }

        let local = parts[0];
        let domain = parts[1];

        // Local part must not be empty
        if local.is_empty() {
            return false;
        }

        // Domain must have at least one '.' and not be empty
        if domain.is_empty() || !domain.contains('.') {
            return false;
        }

        // Domain parts must not be empty
        for part in domain.split('.') {
            if part.is_empty() {
                return false;
            }
        }

        true
    }

    /// Get the email address as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the underlying String.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Get the local part (before '@').
    pub fn local_part(&self) -> &str {
        // SAFETY: Constructor validates exactly one '@' exists
        self.0
            .split('@')
            .next()
            .expect("email validated to contain '@'")
    }

    /// Get the domain part (after '@').
    pub fn domain(&self) -> &str {
        // SAFETY: Constructor validates exactly one '@' exists
        self.0
            .split('@')
            .nth(1)
            .expect("email validated to contain '@'")
    }
}

// Serde support - serialize as string
impl Serialize for EmailAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

// Serde support - deserialize from string with validation
impl<'de> Deserialize<'de> for EmailAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        EmailAddress::new(s).map_err(serde::de::Error::custom)
    }
}

// Display support
impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_valid() {
        let email = EmailAddress::new("user@example.com").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_email_validates_format() {
        assert!(EmailAddress::new("invalid").is_err());
        assert!(EmailAddress::new("@example.com").is_err());
        assert!(EmailAddress::new("user@").is_err());
        assert!(EmailAddress::new("user@domain").is_err());
        assert!(EmailAddress::new("user@@example.com").is_err());
        assert!(EmailAddress::new("valid@example.com").is_ok());
        assert!(EmailAddress::new("user.name+tag@example.co.uk").is_ok());
    }

    #[test]
    fn test_email_parts() {
        let email = EmailAddress::new("user@example.com").unwrap();
        assert_eq!(email.local_part(), "user");
        assert_eq!(email.domain(), "example.com");
    }

    #[test]
    fn test_email_display() {
        let email = EmailAddress::new("user@example.com").unwrap();
        assert_eq!(format!("{}", email), "user@example.com");
    }

    #[test]
    fn test_email_serialization() {
        let email = EmailAddress::new("user@example.com").unwrap();
        let json = serde_json::to_string(&email).unwrap();
        assert_eq!(json, "\"user@example.com\"");
    }

    #[test]
    fn test_email_deserialization() {
        let email: EmailAddress = serde_json::from_str("\"user@example.com\"").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_email_deserialization_invalid_fails() {
        let result: Result<EmailAddress, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }
}

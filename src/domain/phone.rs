//! PhoneNumber value object.

use super::errors::ValidationError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A type-safe wrapper for phone numbers.
///
/// This ensures that phone numbers are validated at construction time.
/// The validation is basic and checks that the number contains at least
/// some digits and optional formatting characters.
///
/// # Example
///
/// ```
/// use dex_mcp_server::domain::PhoneNumber;
///
/// let phone = PhoneNumber::new("+1-555-1234").unwrap();
/// assert_eq!(phone.as_str(), "+1-555-1234");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// Create a new PhoneNumber, validating the format.
    ///
    /// # Validation Rules
    ///
    /// - Must contain at least one digit
    /// - Can contain: digits, spaces, hyphens, parentheses, plus sign, periods
    /// - Must not be empty
    ///
    /// # Errors
    ///
    /// Returns `ValidationError::InvalidPhone` if the phone format is invalid.
    pub fn new(phone: impl Into<String>) -> Result<Self, ValidationError> {
        let phone = phone.into();

        if !Self::is_valid(&phone) {
            return Err(ValidationError::InvalidPhone(phone));
        }

        Ok(Self(phone))
    }

    /// Validate phone format.
    fn is_valid(phone: &str) -> bool {
        if phone.is_empty() {
            return false;
        }

        // Must contain at least one digit
        if !phone.chars().any(|c| c.is_ascii_digit()) {
            return false;
        }

        // All characters must be valid phone number characters
        phone.chars().all(|c| {
            c.is_ascii_digit()
                || c == ' '
                || c == '-'
                || c == '('
                || c == ')'
                || c == '+'
                || c == '.'
        })
    }

    /// Get the phone number as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the underlying String.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Get the phone number with only digits (no formatting).
    pub fn digits_only(&self) -> String {
        self.0.chars().filter(|c| c.is_ascii_digit()).collect()
    }
}

// Serde support - serialize as string
impl Serialize for PhoneNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

// Serde support - deserialize from string with validation
impl<'de> Deserialize<'de> for PhoneNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PhoneNumber::new(s).map_err(serde::de::Error::custom)
    }
}

// Display support
impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_valid() {
        let phone = PhoneNumber::new("+1-555-1234").unwrap();
        assert_eq!(phone.as_str(), "+1-555-1234");
    }

    #[test]
    fn test_phone_validates_format() {
        assert!(PhoneNumber::new("").is_err());
        assert!(PhoneNumber::new("no digits").is_err());
        assert!(PhoneNumber::new("123-456-7890").is_ok());
        assert!(PhoneNumber::new("+1 (555) 123-4567").is_ok());
        assert!(PhoneNumber::new("555.123.4567").is_ok());
        assert!(PhoneNumber::new("+14155551234").is_ok());
        assert!(PhoneNumber::new("invalid@phone").is_err());
    }

    #[test]
    fn test_phone_digits_only() {
        let phone = PhoneNumber::new("+1 (555) 123-4567").unwrap();
        assert_eq!(phone.digits_only(), "15551234567");
    }

    #[test]
    fn test_phone_display() {
        let phone = PhoneNumber::new("+1-555-1234").unwrap();
        assert_eq!(format!("{}", phone), "+1-555-1234");
    }

    #[test]
    fn test_phone_serialization() {
        let phone = PhoneNumber::new("+1-555-1234").unwrap();
        let json = serde_json::to_string(&phone).unwrap();
        assert_eq!(json, "\"+1-555-1234\"");
    }

    #[test]
    fn test_phone_deserialization() {
        let phone: PhoneNumber = serde_json::from_str("\"+1-555-1234\"").unwrap();
        assert_eq!(phone.as_str(), "+1-555-1234");
    }

    #[test]
    fn test_phone_deserialization_invalid_fails() {
        let result: Result<PhoneNumber, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }
}

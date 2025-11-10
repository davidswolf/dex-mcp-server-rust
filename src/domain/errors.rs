//! Domain validation errors.

use std::fmt;

/// Errors that can occur during domain value object validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The provided ID is empty.
    EmptyId,

    /// The provided email address is invalid.
    InvalidEmail(String),

    /// The provided phone number is invalid.
    InvalidPhone(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => write!(f, "ID cannot be empty"),
            Self::InvalidEmail(email) => write!(f, "Invalid email address: {}", email),
            Self::InvalidPhone(phone) => write!(f, "Invalid phone number: {}", phone),
        }
    }
}

impl std::error::Error for ValidationError {}

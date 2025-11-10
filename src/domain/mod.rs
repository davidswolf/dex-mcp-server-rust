//! Domain value objects and types.
//!
//! This module contains type-safe wrappers for domain concepts like
//! contact IDs, email addresses, and phone numbers. These value objects
//! provide validation at construction time and prevent invalid data from
//! being represented in the system.

pub mod contact_id;
pub mod email;
pub mod errors;
pub mod phone;

pub use contact_id::ContactId;
pub use email::EmailAddress;
pub use errors::ValidationError;
pub use phone::PhoneNumber;

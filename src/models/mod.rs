//! Data models for Dex Personal CRM entities.
//!
//! This module contains the data structures representing contacts, notes, reminders,
//! and other entities from the Dex Personal CRM system.

pub mod contact;
pub mod note;
pub mod reminder;

pub use contact::{Contact, SocialProfile};
pub use note::{CreateNoteRequest, Note};
pub use reminder::{CreateReminderRequest, Reminder};

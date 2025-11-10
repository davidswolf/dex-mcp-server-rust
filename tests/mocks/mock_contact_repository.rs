use async_trait::async_trait;
use dex_mcp_server::error::{DexApiError, DexApiResult};
use dex_mcp_server::models::Contact;
use dex_mcp_server::repositories::ContactRepository;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock contact repository for testing.
///
/// Provides an in-memory implementation of ContactRepository that can be
/// easily configured with test data and tracks method calls for verification.
#[allow(dead_code)]
#[derive(Clone)]
pub struct MockContactRepository {
    contacts: Arc<Mutex<HashMap<String, Contact>>>,
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

#[allow(dead_code)]
impl MockContactRepository {
    /// Create a new empty MockContactRepository.
    pub fn new() -> Self {
        Self {
            contacts: Arc::new(Mutex::new(HashMap::new())),
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a contact to the mock repository.
    pub fn add_contact(&self, contact: Contact) {
        let mut contacts = self.contacts.lock().unwrap();
        contacts.insert(contact.id.clone(), contact);
    }

    /// Add multiple contacts to the mock repository.
    pub fn add_contacts(&self, contacts_list: Vec<Contact>) {
        let mut contacts = self.contacts.lock().unwrap();
        for contact in contacts_list {
            contacts.insert(contact.id.clone(), contact);
        }
    }

    /// Get the number of times a method was called.
    pub fn get_call_count(&self, method: &str) -> usize {
        let counts = self.call_counts.lock().unwrap();
        *counts.get(method).unwrap_or(&0)
    }

    /// Reset all call counts.
    pub fn reset_call_counts(&self) {
        let mut counts = self.call_counts.lock().unwrap();
        counts.clear();
    }

    /// Clear all contacts from the repository.
    pub fn clear(&self) {
        let mut contacts = self.contacts.lock().unwrap();
        contacts.clear();
    }

    fn track_call(&self, method: &str) {
        let mut counts = self.call_counts.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
}

impl Default for MockContactRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContactRepository for MockContactRepository {
    async fn get(&self, id: &str) -> DexApiResult<Contact> {
        self.track_call("get");

        let contacts = self.contacts.lock().unwrap();
        contacts
            .get(id)
            .cloned()
            .ok_or_else(|| DexApiError::NotFound(format!("Contact {} not found", id)))
    }

    async fn list(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>> {
        self.track_call("list");

        let contacts = self.contacts.lock().unwrap();
        let result: Vec<Contact> = contacts
            .values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn search_by_email(
        &self,
        email: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Contact>> {
        self.track_call("search_by_email");

        let contacts = self.contacts.lock().unwrap();
        let email_lower = email.to_lowercase();

        let result: Vec<Contact> = contacts
            .values()
            .filter(|contact| {
                contact
                    .emails
                    .iter()
                    .any(|e| e.to_lowercase().contains(&email_lower))
            })
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        Ok(result)
    }

    async fn search_by_name(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Contact>> {
        self.track_call("search_by_name");

        let contacts = self.contacts.lock().unwrap();
        let query_lower = query.to_lowercase();

        let result: Vec<Contact> = contacts
            .values()
            .filter(|contact| {
                let first_match = contact
                    .first_name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);
                let last_match = contact
                    .last_name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);

                first_match || last_match
            })
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        Ok(result)
    }

    async fn create(&self, contact: &Contact) -> DexApiResult<Contact> {
        self.track_call("create");

        let mut contacts = self.contacts.lock().unwrap();

        // Check if contact with this ID already exists
        if contacts.contains_key(&contact.id) {
            return Err(DexApiError::InvalidRequest(format!(
                "Contact with ID {} already exists",
                contact.id
            )));
        }

        contacts.insert(contact.id.clone(), contact.clone());
        Ok(contact.clone())
    }

    async fn update(&self, id: &str, contact: &Contact) -> DexApiResult<Contact> {
        self.track_call("update");

        let mut contacts = self.contacts.lock().unwrap();

        // Check if contact exists
        if !contacts.contains_key(id) {
            return Err(DexApiError::NotFound(format!("Contact {} not found", id)));
        }

        contacts.insert(id.to_string(), contact.clone());
        Ok(contact.clone())
    }

    async fn delete(&self, id: &str) -> DexApiResult<()> {
        self.track_call("delete");

        let mut contacts = self.contacts.lock().unwrap();

        // Check if contact exists
        if !contacts.contains_key(id) {
            return Err(DexApiError::NotFound(format!("Contact {} not found", id)));
        }

        contacts.remove(id);
        Ok(())
    }
}

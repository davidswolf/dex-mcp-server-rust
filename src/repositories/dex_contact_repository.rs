use async_trait::async_trait;
use std::sync::Arc;
use crate::client::AsyncDexClient;
use crate::repositories::traits::ContactRepository;
use crate::models::Contact;
use crate::error::DexApiResult;

/// Contact repository implementation using Dex API client.
///
/// This repository delegates all operations to the AsyncDexClient,
/// providing a clean abstraction layer between business logic and
/// the underlying HTTP client.
pub struct DexContactRepository {
    client: Arc<dyn AsyncDexClient>,
}

impl DexContactRepository {
    /// Create a new DexContactRepository with the given client.
    pub fn new(client: Arc<dyn AsyncDexClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ContactRepository for DexContactRepository {
    async fn get(&self, id: &str) -> DexApiResult<Contact> {
        self.client.get_contact(id).await
    }

    async fn list(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>> {
        self.client.get_contacts(limit, offset).await
    }

    async fn search_by_email(
        &self,
        email: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Contact>> {
        // Note: Current AsyncDexClient doesn't support limit/offset for email search
        // Get all results and apply pagination in-memory
        let mut results = self.client.search_contacts_by_email(email).await?;

        // Apply offset and limit
        if offset >= results.len() {
            return Ok(Vec::new());
        }

        results.drain(..offset);
        results.truncate(limit);
        Ok(results)
    }

    async fn search_by_name(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Contact>> {
        // Current AsyncDexClient doesn't have search_by_name
        // We need to fetch all contacts and filter client-side
        // This is inefficient but maintains the abstraction
        // TODO: Add search_by_name to AsyncDexClient for better performance

        let all_contacts = self.fetch_all_contacts().await?;
        let query_lower = query.to_lowercase();

        let matches: Vec<Contact> = all_contacts
            .into_iter()
            .filter(|contact| {
                // Search in first name, last name, and full name
                let first_match = contact.first_name.as_ref()
                    .map(|n| n.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);
                let last_match = contact.last_name.as_ref()
                    .map(|n| n.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);

                first_match || last_match
            })
            .skip(offset)
            .take(limit)
            .collect();

        Ok(matches)
    }

    async fn create(&self, contact: &Contact) -> DexApiResult<Contact> {
        self.client.create_contact(contact).await
    }

    async fn update(&self, id: &str, contact: &Contact) -> DexApiResult<Contact> {
        self.client.update_contact(id, contact).await
    }

    async fn delete(&self, id: &str) -> DexApiResult<()> {
        self.client.delete_contact(id).await
    }
}

impl DexContactRepository {
    /// Helper to fetch all contacts with pagination.
    /// This is inefficient but necessary until we have better search APIs.
    async fn fetch_all_contacts(&self) -> DexApiResult<Vec<Contact>> {
        const PAGE_SIZE: usize = 100;
        let mut all_contacts = Vec::new();
        let mut offset = 0;

        loop {
            let contacts = self.client.get_contacts(PAGE_SIZE, offset).await?;
            let count = contacts.len();
            all_contacts.extend(contacts);

            if count < PAGE_SIZE {
                break;
            }
            offset += PAGE_SIZE;
        }

        Ok(all_contacts)
    }
}

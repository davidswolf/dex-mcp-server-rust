use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use dex_mcp_server::repositories::ReminderRepository;
use dex_mcp_server::models::Reminder;
use dex_mcp_server::error::{DexApiResult, DexApiError};

/// Mock reminder repository for testing.
#[allow(dead_code)]
#[derive(Clone)]
pub struct MockReminderRepository {
    reminders: Arc<Mutex<HashMap<String, Reminder>>>,
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

#[allow(dead_code)]
impl MockReminderRepository {
    pub fn new() -> Self {
        Self {
            reminders: Arc::new(Mutex::new(HashMap::new())),
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_reminder(&self, reminder: Reminder) {
        let mut reminders = self.reminders.lock().unwrap();
        reminders.insert(reminder.id.clone(), reminder);
    }

    pub fn add_reminders(&self, reminders_list: Vec<Reminder>) {
        let mut reminders = self.reminders.lock().unwrap();
        for reminder in reminders_list {
            reminders.insert(reminder.id.clone(), reminder);
        }
    }

    pub fn get_call_count(&self, method: &str) -> usize {
        let counts = self.call_counts.lock().unwrap();
        *counts.get(method).unwrap_or(&0)
    }

    pub fn reset_call_counts(&self) {
        let mut counts = self.call_counts.lock().unwrap();
        counts.clear();
    }

    pub fn clear(&self) {
        let mut reminders = self.reminders.lock().unwrap();
        reminders.clear();
    }

    fn track_call(&self, method: &str) {
        let mut counts = self.call_counts.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
}

impl Default for MockReminderRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReminderRepository for MockReminderRepository {
    async fn get_for_contact(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Reminder>> {
        self.track_call("get_for_contact");

        let reminders = self.reminders.lock().unwrap();
        let result: Vec<Reminder> = reminders
            .values()
            .filter(|reminder| reminder.contact_id == contact_id)
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn create(&self, reminder: &Reminder) -> DexApiResult<Reminder> {
        self.track_call("create");

        let mut reminders = self.reminders.lock().unwrap();

        if reminders.contains_key(&reminder.id) {
            return Err(DexApiError::InvalidRequest(format!(
                "Reminder with ID {} already exists",
                reminder.id
            )));
        }

        reminders.insert(reminder.id.clone(), reminder.clone());
        Ok(reminder.clone())
    }

    async fn update(&self, id: &str, reminder: &Reminder) -> DexApiResult<Reminder> {
        self.track_call("update");

        let mut reminders = self.reminders.lock().unwrap();

        if !reminders.contains_key(id) {
            return Err(DexApiError::NotFound(format!(
                "Reminder {} not found",
                id
            )));
        }

        reminders.insert(id.to_string(), reminder.clone());
        Ok(reminder.clone())
    }

    async fn delete(&self, id: &str) -> DexApiResult<()> {
        self.track_call("delete");

        let mut reminders = self.reminders.lock().unwrap();

        if !reminders.contains_key(id) {
            return Err(DexApiError::NotFound(format!(
                "Reminder {} not found",
                id
            )));
        }

        reminders.remove(id);
        Ok(())
    }
}

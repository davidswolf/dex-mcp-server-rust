use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Reminder {
    pub id: String,

    #[serde(skip_serializing)]
    pub contact_id: String,

    #[serde(rename = "body")]
    pub text: String,

    #[serde(rename = "due_at_date")]
    pub due_date: String,
}

fn main() {
    let reminder = Reminder {
        id: String::new(),
        contact_id: "abb29721-d8c1-4a9f-a684-05c3ec7595ee".to_string(),
        text: "Reach out to Sayee again".to_string(),
        due_date: "2025-10-22".to_string(),
    };

    let json = serde_json::to_string_pretty(&reminder).unwrap();
    println!("Serialized Reminder:\n{}", json);
    println!("\nNotice: contact_id is missing! This is the bug.");
}

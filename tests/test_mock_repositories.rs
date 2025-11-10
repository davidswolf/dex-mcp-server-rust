mod mocks;

use mocks::MockContactRepository;
use dex_mcp_server::repositories::ContactRepository;
use dex_mcp_server::models::Contact;

fn sample_contact(id: &str, email: &str, first_name: &str, last_name: &str) -> Contact {
    Contact {
        id: id.to_string(),
        first_name: Some(first_name.to_string()),
        last_name: Some(last_name.to_string()),
        emails: vec![email.to_string()],
        ..Default::default()
    }
}

#[tokio::test]
async fn test_mock_repository_get() {
    let repo = MockContactRepository::new();
    let contact = sample_contact("123", "john@example.com", "John", "Doe");
    repo.add_contact(contact.clone());

    let result = repo.get("123").await.unwrap();
    assert_eq!(result.id, "123");
    assert_eq!(repo.get_call_count("get"), 1);
}

#[tokio::test]
async fn test_mock_repository_get_not_found() {
    let repo = MockContactRepository::new();
    let result = repo.get("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mock_repository_list() {
    let repo = MockContactRepository::new();
    repo.add_contact(sample_contact("1", "a@example.com", "Alice", "Smith"));
    repo.add_contact(sample_contact("2", "b@example.com", "Bob", "Jones"));
    repo.add_contact(sample_contact("3", "c@example.com", "Carol", "Brown"));

    let result = repo.list(2, 0).await.unwrap();
    assert_eq!(result.len(), 2);

    let result = repo.list(2, 1).await.unwrap();
    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_mock_repository_search_by_email() {
    let repo = MockContactRepository::new();
    repo.add_contact(sample_contact("1", "john@example.com", "John", "Doe"));
    repo.add_contact(sample_contact("2", "jane@example.com", "Jane", "Doe"));

    let result = repo.search_by_email("john", 10, 0).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "1");
}

#[tokio::test]
async fn test_mock_repository_search_by_name() {
    let repo = MockContactRepository::new();
    repo.add_contact(sample_contact("1", "john@example.com", "John", "Doe"));
    repo.add_contact(sample_contact("2", "jane@example.com", "Jane", "Smith"));

    let result = repo.search_by_name("Jane", 10, 0).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "2");
}

#[tokio::test]
async fn test_mock_repository_create() {
    let repo = MockContactRepository::new();
    let contact = sample_contact("123", "new@example.com", "New", "Contact");

    let result = repo.create(&contact).await.unwrap();
    assert_eq!(result.id, "123");

    // Verify it was actually stored
    let stored = repo.get("123").await.unwrap();
    assert_eq!(stored.id, "123");
}

#[tokio::test]
async fn test_mock_repository_update() {
    let repo = MockContactRepository::new();
    let contact = sample_contact("123", "old@example.com", "Old", "Name");
    repo.add_contact(contact);

    let updated = sample_contact("123", "new@example.com", "New", "Name");
    let result = repo.update("123", &updated).await.unwrap();
    assert_eq!(result.first_name, Some("New".to_string()));

    // Verify it was actually updated
    let stored = repo.get("123").await.unwrap();
    assert_eq!(stored.first_name, Some("New".to_string()));
}

#[tokio::test]
async fn test_mock_repository_delete() {
    let repo = MockContactRepository::new();
    let contact = sample_contact("123", "test@example.com", "Test", "User");
    repo.add_contact(contact);

    repo.delete("123").await.unwrap();

    // Verify it was deleted
    let result = repo.get("123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_count_tracking() {
    let repo = MockContactRepository::new();
    let contact = sample_contact("123", "test@example.com", "Test", "User");
    repo.add_contact(contact);

    assert_eq!(repo.get_call_count("get"), 0);

    repo.get("123").await.unwrap();
    assert_eq!(repo.get_call_count("get"), 1);

    repo.get("123").await.unwrap();
    assert_eq!(repo.get_call_count("get"), 2);

    repo.reset_call_counts();
    assert_eq!(repo.get_call_count("get"), 0);
}

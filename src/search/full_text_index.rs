//! Full-text search index implementation.
//!
//! This module provides fuzzy full-text search across contacts, notes, and reminders,
//! with snippet generation and match context extraction.

use crate::models::{Contact, Note, Reminder};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Maximum snippet length in characters
const MAX_SNIPPET_LENGTH: usize = 150;

/// Context characters to show before and after match
const CONTEXT_CHARS: usize = 50;

/// A searchable document extracted from contact data.
#[derive(Debug, Clone)]
pub struct SearchableDocument {
    /// Contact ID this document belongs to
    pub contact_id: String,

    /// Contact name for display
    pub contact_name: String,

    /// Field type (e.g., "name", "email", "note", "reminder")
    pub field_type: SearchableField,

    /// Searchable text content
    pub content: String,

    /// Original item ID (for notes/reminders)
    pub item_id: Option<String>,
}

/// Type of searchable field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchableField {
    /// Contact name
    Name,
    /// Contact email
    Email,
    /// Contact phone
    Phone,
    /// Contact company
    Company,
    /// Contact job title
    JobTitle,
    /// Note content
    Note,
    /// Reminder content
    Reminder,
}

impl SearchableField {
    /// Get display name for the field type.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Email => "email",
            Self::Phone => "phone",
            Self::Company => "company",
            Self::JobTitle => "job title",
            Self::Note => "note",
            Self::Reminder => "reminder",
        }
    }
}

/// A match found in a document with context.
#[derive(Debug, Clone)]
pub struct MatchContext {
    /// Field where the match was found
    pub field_type: SearchableField,

    /// Snippet showing the match with context
    pub snippet: String,

    /// Confidence score (0-100)
    pub confidence: u8,

    /// Item ID (for notes/reminders)
    pub item_id: Option<String>,
}

/// A search result for a contact with all matches.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The contact
    pub contact: Contact,

    /// All matches found for this contact
    pub matches: Vec<MatchContext>,

    /// Overall confidence score (0-100), boosted for multiple matches
    pub confidence: u8,
}

/// Full-text search index for contacts and related data.
pub struct FullTextSearchIndex {
    /// All searchable documents
    documents: Vec<SearchableDocument>,
}

impl FullTextSearchIndex {
    /// Create a new empty search index.
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    /// Index a contact and all its related data.
    ///
    /// This extracts searchable documents from the contact's fields,
    /// notes, and reminders.
    pub fn index_contact(&mut self, contact: &Contact, notes: &[Note], reminders: &[Reminder]) {
        let contact_id = &contact.id;
        let contact_name = &contact.name;

        // Index contact name
        self.documents.push(SearchableDocument {
            contact_id: contact_id.clone(),
            contact_name: contact_name.clone(),
            field_type: SearchableField::Name,
            content: contact.name.clone(),
            item_id: None,
        });

        // Index primary email
        if let Some(ref email) = contact.email {
            self.documents.push(SearchableDocument {
                contact_id: contact_id.clone(),
                contact_name: contact_name.clone(),
                field_type: SearchableField::Email,
                content: email.clone(),
                item_id: None,
            });
        }

        // Index additional emails
        for email in &contact.emails {
            self.documents.push(SearchableDocument {
                contact_id: contact_id.clone(),
                contact_name: contact_name.clone(),
                field_type: SearchableField::Email,
                content: email.clone(),
                item_id: None,
            });
        }

        // Index primary phone
        if let Some(ref phone) = contact.phone {
            self.documents.push(SearchableDocument {
                contact_id: contact_id.clone(),
                contact_name: contact_name.clone(),
                field_type: SearchableField::Phone,
                content: phone.clone(),
                item_id: None,
            });
        }

        // Index additional phones
        for phone in &contact.phones {
            self.documents.push(SearchableDocument {
                contact_id: contact_id.clone(),
                contact_name: contact_name.clone(),
                field_type: SearchableField::Phone,
                content: phone.clone(),
                item_id: None,
            });
        }

        // Index company
        if let Some(ref company) = contact.company {
            self.documents.push(SearchableDocument {
                contact_id: contact_id.clone(),
                contact_name: contact_name.clone(),
                field_type: SearchableField::Company,
                content: company.clone(),
                item_id: None,
            });
        }

        // Index job title
        if let Some(ref title) = contact.title {
            self.documents.push(SearchableDocument {
                contact_id: contact_id.clone(),
                contact_name: contact_name.clone(),
                field_type: SearchableField::JobTitle,
                content: title.clone(),
                item_id: None,
            });
        }

        // Index notes
        for note in notes {
            let plain_text = strip_html(&note.content);
            if !plain_text.trim().is_empty() {
                self.documents.push(SearchableDocument {
                    contact_id: contact_id.clone(),
                    contact_name: contact_name.clone(),
                    field_type: SearchableField::Note,
                    content: plain_text,
                    item_id: Some(note.id.clone()),
                });
            }
        }

        // Index reminders
        for reminder in reminders {
            if !reminder.text.trim().is_empty() {
                self.documents.push(SearchableDocument {
                    contact_id: contact_id.clone(),
                    contact_name: contact_name.clone(),
                    field_type: SearchableField::Reminder,
                    content: reminder.text.clone(),
                    item_id: Some(reminder.id.clone()),
                });
            }
        }
    }

    /// Search the index for a query string.
    ///
    /// Returns results grouped by contact, sorted by confidence.
    ///
    /// # Arguments
    /// * `query` - The search query
    /// * `max_results` - Maximum number of contacts to return
    /// * `min_confidence` - Minimum confidence threshold (0-100)
    pub fn search(
        &self,
        contacts: &[Contact],
        query: &str,
        max_results: usize,
        min_confidence: u8,
    ) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut matches_by_contact: HashMap<String, Vec<MatchContext>> = HashMap::new();

        // Search all documents
        for doc in &self.documents {
            if let Some(match_ctx) = self.find_match(doc, &query_lower) {
                if match_ctx.confidence >= min_confidence {
                    matches_by_contact
                        .entry(doc.contact_id.clone())
                        .or_default()
                        .push(match_ctx);
                }
            }
        }

        // Build search results
        let mut results: Vec<SearchResult> = Vec::new();

        for (contact_id, matches) in matches_by_contact {
            // Find the contact
            if let Some(contact) = contacts.iter().find(|c| c.id == contact_id) {
                // Calculate overall confidence (boost for multiple matches)
                let max_confidence = matches.iter().map(|m| m.confidence).max().unwrap_or(0);
                let match_count_boost = (matches.len().saturating_sub(1) * 5).min(15) as u8;
                let overall_confidence = (max_confidence + match_count_boost).min(100);

                if overall_confidence >= min_confidence {
                    results.push(SearchResult {
                        contact: contact.clone(),
                        matches,
                        confidence: overall_confidence,
                    });
                }
            }
        }

        // Sort by confidence (highest first)
        results.sort_by(|a, b| b.confidence.cmp(&a.confidence));

        // Limit results
        results.truncate(max_results);

        results
    }

    /// Find a match in a document and generate context.
    fn find_match(&self, doc: &SearchableDocument, query_lower: &str) -> Option<MatchContext> {
        let content_lower = doc.content.to_lowercase();

        // Calculate match confidence
        let confidence = self.calculate_match_confidence(query_lower, &content_lower)?;

        // Generate snippet
        let snippet = self.generate_snippet(&doc.content, &content_lower, query_lower);

        Some(MatchContext {
            field_type: doc.field_type.clone(),
            snippet,
            confidence,
            item_id: doc.item_id.clone(),
        })
    }

    /// Calculate match confidence for a query against content.
    ///
    /// Returns None if no match, Some(confidence) if matched.
    fn calculate_match_confidence(&self, query: &str, content: &str) -> Option<u8> {
        if query.is_empty() || content.is_empty() {
            return None;
        }

        // Exact substring match (highest confidence)
        if content.contains(query) {
            let ratio = query.len() as f64 / content.len() as f64;
            let confidence = (85.0 * ratio + 10.0).min(95.0) as u8;
            return Some(confidence);
        }

        // Fuzzy word matching - split query and content into words
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let content_words: Vec<&str> = content.split_whitespace().collect();

        if query_words.is_empty() || content_words.is_empty() {
            return None;
        }

        // Check if all query words have fuzzy matches in content
        let mut total_score = 0;
        let mut matches = 0;

        for query_word in &query_words {
            let mut best_word_score = 0;

            for content_word in &content_words {
                if content_word.contains(query_word) {
                    // Substring match within word
                    best_word_score = 85;
                    break;
                }

                // Fuzzy match using Levenshtein distance
                let distance = levenshtein_distance(query_word, content_word);
                let max_len = query_word.len().max(content_word.len());

                if max_len > 0 && distance as f64 / max_len as f64 <= 0.4 {
                    // Allow 40% difference for fuzzy match
                    let similarity = 1.0 - (distance as f64 / max_len as f64);
                    let score = (similarity * 75.0) as u8;
                    best_word_score = best_word_score.max(score);
                }
            }

            if best_word_score > 0 {
                total_score += best_word_score as usize;
                matches += 1;
            }
        }

        // Require at least 50% of query words to match
        if matches >= query_words.len().div_ceil(2) {
            let avg_score = (total_score / query_words.len()).min(90) as u8;
            Some(avg_score)
        } else {
            None
        }
    }

    /// Generate a snippet with context around the match.
    fn generate_snippet(&self, original: &str, content_lower: &str, query: &str) -> String {
        // Find the position of the query in the content
        let pos = if let Some(idx) = content_lower.find(query) {
            idx
        } else {
            // If exact match not found, find the first word from query
            let first_word = query.split_whitespace().next().unwrap_or(query);
            content_lower
                .split_whitespace()
                .enumerate()
                .find(|(_, word)| word.contains(first_word))
                .map(|(i, _)| {
                    content_lower
                        .split_whitespace()
                        .take(i)
                        .map(|w| w.len() + 1)
                        .sum::<usize>()
                })
                .unwrap_or(0)
        };

        // Calculate snippet boundaries
        let start = pos.saturating_sub(CONTEXT_CHARS);
        let end = (pos + query.len() + CONTEXT_CHARS).min(original.len());

        // Extract snippet
        let mut snippet = original[start..end].to_string();

        // Add ellipsis if truncated
        if start > 0 {
            snippet = format!("...{}", snippet);
        }
        if end < original.len() {
            snippet = format!("{}...", snippet);
        }

        // Truncate if too long
        if snippet.len() > MAX_SNIPPET_LENGTH {
            snippet.truncate(MAX_SNIPPET_LENGTH - 3);
            snippet.push_str("...");
        }

        snippet
    }

    /// Clear all documents from the index.
    pub fn clear(&mut self) {
        self.documents.clear();
    }

    /// Get the number of indexed documents.
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }
}

impl Default for FullTextSearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Strip HTML tags from text, preserving content.
pub fn strip_html(html: &str) -> String {
    static HTML_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]*>").unwrap());

    // Remove HTML tags
    let text = HTML_TAG_RE.replace_all(html, " ");

    // Collapse multiple spaces and trim
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Calculate Levenshtein distance between two strings.
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix: Vec<Vec<usize>> = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first column
    for (i, row) in matrix.iter_mut().enumerate() {
        row[0] = i;
    }
    // Initialize first row
    for (j, cell) in matrix[0].iter_mut().enumerate() {
        *cell = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for (i, c1) in s1_chars.iter().enumerate() {
        for (j, c2) in s2_chars.iter().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_contact(id: &str, name: &str) -> Contact {
        Contact::new(id.to_string(), name.to_string())
    }

    #[test]
    fn test_strip_html() {
        assert_eq!(strip_html("<p>Hello world</p>"), "Hello world");
        assert_eq!(
            strip_html("<div>Hello <strong>world</strong></div>"),
            "Hello world"
        );
        assert_eq!(strip_html("No HTML here"), "No HTML here");
        assert_eq!(
            strip_html("<p>  Multiple   spaces  </p>"),
            "Multiple spaces"
        );
    }

    #[test]
    fn test_searchable_field_display_name() {
        assert_eq!(SearchableField::Name.display_name(), "name");
        assert_eq!(SearchableField::Email.display_name(), "email");
        assert_eq!(SearchableField::Note.display_name(), "note");
    }

    #[test]
    fn test_index_contact_basic() {
        let mut index = FullTextSearchIndex::new();
        let mut contact = create_test_contact("1", "John Doe");
        contact.email = Some("john@example.com".to_string());
        contact.company = Some("Acme Corp".to_string());

        index.index_contact(&contact, &[], &[]);

        assert_eq!(index.document_count(), 3); // name, email, company
    }

    #[test]
    fn test_index_contact_with_notes() {
        let mut index = FullTextSearchIndex::new();
        let contact = create_test_contact("1", "John Doe");
        let notes = vec![Note {
            id: "note1".to_string(),
            contact_id: "1".to_string(),
            content: "<p>Meeting notes</p>".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: Some("2024-01-01T00:00:00Z".to_string()),
            tags: Vec::new(),
            source: None,
        }];

        index.index_contact(&contact, &notes, &[]);

        assert_eq!(index.document_count(), 2); // name + note
    }

    #[test]
    fn test_index_contact_with_reminders() {
        let mut index = FullTextSearchIndex::new();
        let contact = create_test_contact("1", "John Doe");
        let reminders = vec![Reminder {
            id: "rem1".to_string(),
            contact_id: "1".to_string(),
            text: "Follow up next week".to_string(),
            due_date: "2024-01-15".to_string(),
            completed: false,
            completed_at: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: Some("2024-01-01T00:00:00Z".to_string()),
            tags: Vec::new(),
            priority: None,
        }];

        index.index_contact(&contact, &[], &reminders);

        assert_eq!(index.document_count(), 2); // name + reminder
    }

    #[test]
    fn test_search_exact_match() {
        let mut index = FullTextSearchIndex::new();
        let mut contact = create_test_contact("1", "John Doe");
        contact.email = Some("john@example.com".to_string());

        let contacts = vec![contact.clone()];
        index.index_contact(&contact, &[], &[]);

        let results = index.search(&contacts, "john", 10, 0);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].contact.id, "1");
        assert!(results[0].confidence > 0);
        assert!(!results[0].matches.is_empty());
    }

    #[test]
    fn test_search_in_notes() {
        let mut index = FullTextSearchIndex::new();
        let contact = create_test_contact("1", "John Doe");
        let notes = vec![Note {
            id: "note1".to_string(),
            contact_id: "1".to_string(),
            content: "Discussed the project timeline".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: Some("2024-01-01T00:00:00Z".to_string()),
            tags: Vec::new(),
            source: None,
        }];

        let contacts = vec![contact.clone()];
        index.index_contact(&contact, &notes, &[]);

        let results = index.search(&contacts, "timeline", 10, 0);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 1);
        assert_eq!(results[0].matches[0].field_type, SearchableField::Note);
    }

    #[test]
    fn test_search_multiple_matches_boost() {
        let mut index = FullTextSearchIndex::new();
        let mut contact = create_test_contact("1", "John Smith");
        contact.company = Some("John's Company".to_string());

        let contacts = vec![contact.clone()];
        index.index_contact(&contact, &[], &[]);

        let results = index.search(&contacts, "john", 10, 0);

        assert_eq!(results.len(), 1);
        // Should have matches in both name and company
        assert!(results[0].matches.len() >= 2);
        // Confidence should be boosted for multiple matches
        // With 2 matches: one in name (~44) and one in company (~32)
        // Overall: max(44) + (2-1)*5 = 49
        assert!(
            results[0].confidence >= 45,
            "Expected confidence >= 45, got {}",
            results[0].confidence
        );
    }

    #[test]
    fn test_search_confidence_threshold() {
        let mut index = FullTextSearchIndex::new();
        let contact = create_test_contact("1", "John Doe");

        let contacts = vec![contact.clone()];
        index.index_contact(&contact, &[], &[]);

        // With high threshold, fuzzy matches may be filtered out
        let results_high = index.search(&contacts, "jon", 10, 90);
        let results_low = index.search(&contacts, "jon", 10, 0);

        // Low threshold should return more results
        assert!(results_low.len() >= results_high.len());
    }

    #[test]
    fn test_search_max_results() {
        let mut index = FullTextSearchIndex::new();

        let contacts: Vec<Contact> = (0..10)
            .map(|i| create_test_contact(&i.to_string(), &format!("Contact {}", i)))
            .collect();

        for contact in &contacts {
            index.index_contact(contact, &[], &[]);
        }

        let results = index.search(&contacts, "contact", 3, 0);

        assert!(results.len() <= 3);
    }

    #[test]
    fn test_snippet_generation() {
        let index = FullTextSearchIndex::new();
        let original =
            "This is a long text with many words to test snippet generation functionality";
        let snippet = index.generate_snippet(original, &original.to_lowercase(), "snippet");

        assert!(snippet.contains("snippet"));
        assert!(snippet.len() <= MAX_SNIPPET_LENGTH);
    }

    #[test]
    fn test_snippet_with_ellipsis() {
        let index = FullTextSearchIndex::new();
        let original = "Start of text. This is the middle section with the important keyword that we are searching for. End of text with more content.";
        let snippet = index.generate_snippet(original, &original.to_lowercase(), "keyword");

        assert!(snippet.contains("keyword"));
        // Should have ellipsis since we're extracting from the middle
        assert!(snippet.contains("..."));
    }

    #[test]
    fn test_clear_index() {
        let mut index = FullTextSearchIndex::new();
        let contact = create_test_contact("1", "John Doe");

        index.index_contact(&contact, &[], &[]);
        assert!(index.document_count() > 0);

        index.clear();
        assert_eq!(index.document_count(), 0);
    }

    #[test]
    fn test_fuzzy_word_matching() {
        let mut index = FullTextSearchIndex::new();
        let mut contact = create_test_contact("1", "John Doe");
        contact.company = Some("Software Engineering Company".to_string());

        let contacts = vec![contact.clone()];
        index.index_contact(&contact, &[], &[]);

        // Fuzzy match - slightly misspelled
        let results = index.search(&contacts, "sofware", 10, 0);

        // Should still find a match
        assert!(!results.is_empty());
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
        assert_eq!(levenshtein_distance("", "test"), 4);
        assert_eq!(levenshtein_distance("test", ""), 4);
        assert_eq!(levenshtein_distance("same", "same"), 0);
    }
}

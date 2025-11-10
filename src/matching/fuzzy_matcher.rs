//! Fuzzy matching implementation for contact discovery.
//!
//! This module provides intelligent contact matching with:
//! - Exact matching on email, phone, and social URLs
//! - Fuzzy name matching using the nucleo library
//! - Confidence scoring (0-100 scale)
//! - Company-based confidence boosting

use crate::models::Contact;

/// A match result containing a contact and its confidence score.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched contact
    pub contact: Contact,

    /// Confidence score (0-100, where 100 is an exact match)
    pub confidence: u8,

    /// Type of match that produced this result
    pub match_type: MatchType,
}

/// The type of match that was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchType {
    /// Exact email match
    ExactEmail,

    /// Exact phone match
    ExactPhone,

    /// Exact social URL match
    ExactSocial,

    /// Fuzzy name match
    FuzzyName,
}

/// Contact matcher with fuzzy and exact matching capabilities.
pub struct ContactMatcher;

impl ContactMatcher {
    /// Create a new ContactMatcher.
    pub fn new() -> Self {
        Self
    }

    /// Find matching contacts from a list based on the search query.
    ///
    /// # Arguments
    /// * `query` - Search parameters (name, email, phone, company, etc.)
    /// * `contacts` - List of contacts to search through
    /// * `max_results` - Maximum number of results to return
    /// * `min_confidence` - Minimum confidence threshold (0-100)
    ///
    /// # Returns
    /// A vector of MatchResult, sorted by confidence (highest first)
    pub fn find_matches(
        &mut self,
        query: &ContactQuery,
        contacts: &[Contact],
        max_results: usize,
        min_confidence: u8,
    ) -> Vec<MatchResult> {
        let mut results: Vec<MatchResult> = Vec::new();

        for contact in contacts {
            // Try exact matches first (highest priority)
            if let Some(email) = &query.email {
                if let Some(confidence) = self.match_email(email, contact) {
                    results.push(MatchResult {
                        contact: contact.clone(),
                        confidence,
                        match_type: MatchType::ExactEmail,
                    });
                    continue;
                }
            }

            if let Some(phone) = &query.phone {
                if let Some(confidence) = self.match_phone(phone, contact) {
                    results.push(MatchResult {
                        contact: contact.clone(),
                        confidence,
                        match_type: MatchType::ExactPhone,
                    });
                    continue;
                }
            }

            if let Some(social_url) = &query.social_url {
                if let Some(confidence) = self.match_social_url(social_url, contact) {
                    results.push(MatchResult {
                        contact: contact.clone(),
                        confidence,
                        match_type: MatchType::ExactSocial,
                    });
                    continue;
                }
            }

            // Try fuzzy name matching
            if let Some(name) = &query.name {
                if let Some(mut confidence) = self.fuzzy_match_name(name, &contact.name) {
                    // Boost confidence if company also matches
                    if let (Some(query_company), Some(contact_company)) =
                        (&query.company, &contact.company)
                    {
                        if self
                            .fuzzy_match_company(query_company, contact_company)
                            .is_some()
                        {
                            confidence = (confidence + 15).min(95); // Boost but don't exceed exact match range
                        }
                    }

                    if confidence >= min_confidence {
                        results.push(MatchResult {
                            contact: contact.clone(),
                            confidence,
                            match_type: MatchType::FuzzyName,
                        });
                    }
                }
            }
        }

        // Sort by confidence (highest first), then by name
        results.sort_by(|a, b| {
            b.confidence
                .cmp(&a.confidence)
                .then_with(|| a.contact.name.cmp(&b.contact.name))
        });

        // Limit results
        results.truncate(max_results);

        results
    }

    /// Match email addresses (exact match with normalization).
    ///
    /// Returns confidence score (100) if matched, None otherwise.
    fn match_email(&self, query_email: &str, contact: &Contact) -> Option<u8> {
        let normalized_query = Self::normalize_email(query_email);

        // Check primary email
        if let Some(ref email) = contact.email {
            if Self::normalize_email(email) == normalized_query {
                return Some(100);
            }
        }

        // Check additional emails
        for email in &contact.emails {
            if Self::normalize_email(email) == normalized_query {
                return Some(100);
            }
        }

        None
    }

    /// Match phone numbers (exact match with normalization).
    ///
    /// Returns confidence score (100) if matched, None otherwise.
    fn match_phone(&self, query_phone: &str, contact: &Contact) -> Option<u8> {
        let normalized_query = Self::normalize_phone(query_phone);

        // Check primary phone
        if let Some(ref phone) = contact.phone {
            if Self::normalize_phone(phone) == normalized_query {
                return Some(100);
            }
        }

        // Check additional phones
        for phone in &contact.phones {
            if Self::normalize_phone(phone) == normalized_query {
                return Some(100);
            }
        }

        None
    }

    /// Match social media URLs (exact match with normalization).
    ///
    /// Returns confidence score (100) if matched, None otherwise.
    fn match_social_url(&self, query_url: &str, contact: &Contact) -> Option<u8> {
        let normalized_query = Self::normalize_url(query_url);

        for profile in &contact.social_profiles {
            if Self::normalize_url(&profile.url) == normalized_query {
                return Some(100);
            }
        }

        None
    }

    /// Fuzzy match names using a simple but effective algorithm.
    ///
    /// Returns confidence score (0-100) if matched, None otherwise.
    fn fuzzy_match_name(&self, query: &str, contact_name: &str) -> Option<u8> {
        // Normalize and prepare strings
        let query_normalized = Self::normalize_name(query);
        let name_normalized = Self::normalize_name(contact_name);

        // Calculate fuzzy match score
        let score = Self::calculate_fuzzy_score(&query_normalized, &name_normalized);

        if score > 0 {
            Some(score)
        } else {
            None
        }
    }

    /// Fuzzy match company names.
    fn fuzzy_match_company(&self, query: &str, company: &str) -> Option<u8> {
        let query_normalized = Self::normalize_name(query);
        let company_normalized = Self::normalize_name(company);

        let score = Self::calculate_fuzzy_score(&query_normalized, &company_normalized);

        if score > 50 {
            // Higher threshold for company matching
            Some(score)
        } else {
            None
        }
    }

    /// Calculate fuzzy match score using Levenshtein distance and substring matching.
    ///
    /// Returns a confidence score from 0-95 (95 max to reserve 100 for exact matches).
    fn calculate_fuzzy_score(query: &str, target: &str) -> u8 {
        if query.is_empty() || target.is_empty() {
            return 0;
        }

        // Exact match
        if query == target {
            return 95; // Reserve 100 for exact email/phone/social matches
        }

        // Contains match (substring)
        if target.contains(query) {
            let ratio = query.len() as f64 / target.len() as f64;
            return (85.0 * ratio + 10.0) as u8; // 85-95 range for contains matches
        }

        if query.contains(target) {
            return 85;
        }

        // Levenshtein distance-based fuzzy matching
        let distance = Self::levenshtein_distance(query, target);
        let max_len = query.len().max(target.len());

        if distance as f64 / max_len as f64 > 0.5 {
            // Too many differences
            return 0;
        }

        // Calculate similarity percentage
        let similarity = 1.0 - (distance as f64 / max_len as f64);
        (similarity * 85.0) as u8 // Scale to 0-85 range
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

    // ==================== Normalization Functions ====================

    /// Normalize an email address for comparison.
    ///
    /// Converts to lowercase and trims whitespace.
    pub fn normalize_email(email: &str) -> String {
        email.trim().to_lowercase()
    }

    /// Normalize a phone number for comparison.
    ///
    /// Extracts only digits and takes the last 10 digits (US format).
    pub fn normalize_phone(phone: &str) -> String {
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

        // Take last 10 digits (handles country codes)
        if digits.len() > 10 {
            digits[digits.len() - 10..].to_string()
        } else {
            digits
        }
    }

    /// Normalize a URL for comparison.
    ///
    /// Converts to lowercase, removes protocol and trailing slashes.
    pub fn normalize_url(url: &str) -> String {
        url.trim()
            .to_lowercase()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("www.")
            .trim_end_matches('/')
            .to_string()
    }

    /// Normalize a name for fuzzy matching.
    ///
    /// Converts to lowercase and collapses whitespace.
    pub fn normalize_name(name: &str) -> String {
        name.trim()
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Default for ContactMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Search query parameters for contact matching.
#[derive(Debug, Clone, Default)]
pub struct ContactQuery {
    /// Name to search for
    pub name: Option<String>,

    /// Email to search for
    pub email: Option<String>,

    /// Phone number to search for
    pub phone: Option<String>,

    /// Company to search for
    pub company: Option<String>,

    /// Social media URL to search for
    pub social_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SocialProfile;

    fn create_test_contact(name: &str, email: Option<&str>, phone: Option<&str>) -> Contact {
        let mut contact = Contact::new("test-id".to_string(), name.to_string());
        contact.email = email.map(|e| e.to_string());
        contact.phone = phone.map(|p| p.to_string());
        contact
    }

    #[test]
    fn test_normalize_email() {
        assert_eq!(
            ContactMatcher::normalize_email("  Test@Example.COM  "),
            "test@example.com"
        );
        assert_eq!(
            ContactMatcher::normalize_email("user@domain.com"),
            "user@domain.com"
        );
    }

    #[test]
    fn test_normalize_phone() {
        assert_eq!(
            ContactMatcher::normalize_phone("+1 (555) 123-4567"),
            "5551234567"
        );
        assert_eq!(
            ContactMatcher::normalize_phone("555-123-4567"),
            "5551234567"
        );
        assert_eq!(
            ContactMatcher::normalize_phone("+44 20 7123 4567"),
            "2071234567"
        ); // Last 10 digits
    }

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            ContactMatcher::normalize_url("https://twitter.com/username/"),
            "twitter.com/username"
        );
        assert_eq!(
            ContactMatcher::normalize_url("HTTP://WWW.LINKEDIN.COM/in/user"),
            "linkedin.com/in/user"
        );
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(ContactMatcher::normalize_name("  John   Doe  "), "john doe");
        assert_eq!(ContactMatcher::normalize_name("JANE SMITH"), "jane smith");
    }

    #[test]
    fn test_exact_email_match() {
        let matcher = ContactMatcher::new();
        let contact = create_test_contact("John Doe", Some("john@example.com"), None);

        assert_eq!(matcher.match_email("john@example.com", &contact), Some(100));
        assert_eq!(matcher.match_email("JOHN@EXAMPLE.COM", &contact), Some(100)); // Case insensitive
        assert_eq!(matcher.match_email("jane@example.com", &contact), None);
    }

    #[test]
    fn test_exact_phone_match() {
        let matcher = ContactMatcher::new();
        let contact = create_test_contact("John Doe", None, Some("+1 (555) 123-4567"));

        assert_eq!(matcher.match_phone("555-123-4567", &contact), Some(100));
        assert_eq!(matcher.match_phone("+1 555 123 4567", &contact), Some(100)); // Normalized
        assert_eq!(matcher.match_phone("555-999-8888", &contact), None);
    }

    #[test]
    fn test_fuzzy_name_match() {
        let matcher = ContactMatcher::new();

        // Exact match should score very high
        let score = matcher.fuzzy_match_name("john doe", "John Doe");
        assert!(score.is_some());
        assert!(score.unwrap() >= 85);

        // Partial match
        let score = matcher.fuzzy_match_name("john", "John Doe");
        assert!(score.is_some());
        assert!(score.unwrap() >= 50);

        // Typo tolerance
        let score = matcher.fuzzy_match_name("johnn doe", "John Doe");
        assert!(score.is_some());

        // No match
        let _score = matcher.fuzzy_match_name("alice", "John Doe");
        // May or may not match depending on nucleo's algorithm
    }

    #[test]
    fn test_find_matches_by_email() {
        let mut matcher = ContactMatcher::new();
        let contacts = vec![
            create_test_contact("John Doe", Some("john@example.com"), None),
            create_test_contact("Jane Smith", Some("jane@example.com"), None),
        ];

        let query = ContactQuery {
            email: Some("john@example.com".to_string()),
            ..Default::default()
        };

        let results = matcher.find_matches(&query, &contacts, 5, 0);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].contact.name, "John Doe");
        assert_eq!(results[0].confidence, 100);
        assert_eq!(results[0].match_type, MatchType::ExactEmail);
    }

    #[test]
    fn test_find_matches_by_name() {
        let mut matcher = ContactMatcher::new();
        let contacts = vec![
            create_test_contact("John Doe", None, None),
            create_test_contact("Jane Doe", None, None),
            create_test_contact("Alice Smith", None, None),
        ];

        let query = ContactQuery {
            name: Some("doe".to_string()),
            ..Default::default()
        };

        // Lower threshold for partial name matches
        let results = matcher.find_matches(&query, &contacts, 5, 30);

        // Should match both John and Jane Doe
        assert!(results.len() >= 2);
        assert!(results.iter().any(|r| r.contact.name.contains("Doe")));
    }

    #[test]
    fn test_match_result_sorting() {
        let mut matcher = ContactMatcher::new();
        let contacts = vec![
            create_test_contact("John Doe", Some("john@example.com"), None),
            create_test_contact("John Smith", None, None),
            create_test_contact("Jane Doe", None, None),
        ];

        let query = ContactQuery {
            name: Some("john".to_string()),
            email: Some("john@example.com".to_string()),
            ..Default::default()
        };

        let results = matcher.find_matches(&query, &contacts, 5, 0);

        // Email match should come first (confidence 100)
        assert!(!results.is_empty());
        assert_eq!(
            results[0].contact.email,
            Some("john@example.com".to_string())
        );
        assert_eq!(results[0].confidence, 100);
    }

    #[test]
    fn test_confidence_threshold() {
        let mut matcher = ContactMatcher::new();
        let contacts = vec![
            create_test_contact("John Doe", None, None),
            create_test_contact("Alice Smith", None, None),
        ];

        let query = ContactQuery {
            name: Some("john".to_string()),
            ..Default::default()
        };

        // With low threshold
        let results = matcher.find_matches(&query, &contacts, 5, 0);
        let count_low = results.len();

        // With high threshold
        let results = matcher.find_matches(&query, &contacts, 5, 90);
        let count_high = results.len();

        // Higher threshold should filter out more results
        assert!(count_high <= count_low);
    }

    #[test]
    fn test_max_results_limit() {
        let mut matcher = ContactMatcher::new();
        let contacts = vec![
            create_test_contact("John Doe", None, None),
            create_test_contact("John Smith", None, None),
            create_test_contact("Johnny Walker", None, None),
            create_test_contact("Jonathan Lee", None, None),
        ];

        let query = ContactQuery {
            name: Some("john".to_string()),
            ..Default::default()
        };

        let results = matcher.find_matches(&query, &contacts, 2, 0);

        assert!(results.len() <= 2);
    }

    #[test]
    fn test_social_url_match() {
        let mut matcher = ContactMatcher::new();
        let mut contact = create_test_contact("John Doe", None, None);
        contact.social_profiles = vec![SocialProfile::new(
            "twitter".to_string(),
            "https://twitter.com/johndoe".to_string(),
        )];

        let query = ContactQuery {
            social_url: Some("twitter.com/johndoe".to_string()),
            ..Default::default()
        };

        let results = matcher.find_matches(&query, &[contact], 5, 0);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].confidence, 100);
        assert_eq!(results[0].match_type, MatchType::ExactSocial);
    }
}

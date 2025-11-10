# DexMCPServerRust - Architecture Improvements Work Plan

**Created:** 2025-11-09
**Based on:** architecture-review.md
**Estimated Timeline:** 10 weeks
**Team Size:** 1-2 developers

---

## Executive Summary

This work plan addresses critical architectural issues identified in the DexMCPServerRust codebase. Issues are prioritized by impact on production viability, with critical performance bottlenecks addressed first, followed by architectural improvements, and finally code quality enhancements.

### Success Criteria

| Metric | Current | Target | Priority |
|--------|---------|--------|----------|
| Search response time | >5s | <200ms (cached) | CRITICAL |
| Request throughput | <10 req/s | >100 req/s | CRITICAL |
| Memory usage (10k contacts) | ~500MB | <100MB | HIGH |
| Test coverage | ~40% | >80% | MEDIUM |
| Error rate | Unknown | <0.1% | HIGH |

---

## Phase 1: Critical Performance Fixes (Weeks 1-4)

**Goal:** Make the system production-ready by fixing fundamental async/blocking issues and eliminating catastrophic performance bottlenecks.

**Risk Level:** 🔴 **CRITICAL** - Current system will fail under production load

---

### Milestone 1.1: Async/Blocking Foundation (Week 1-2)

#### Task 1.1.1: Add Characterization Tests
**Priority:** CRITICAL
**Effort:** 2 days
**Assignee:** Developer 1
**Dependencies:** None

**Description:** Create tests that document current behavior before refactoring.

**Acceptance Criteria:**
- [ ] Integration tests for each tool (discovery, enrichment, history)
- [ ] Tests use mockito for HTTP mocking
- [ ] Tests cover happy path and common error cases
- [ ] Baseline performance measurements documented
- [ ] All tests pass with current implementation

**Implementation Steps:**
1. Create `tests/characterization/` directory
2. Add tests for `ContactDiscoveryTools`:
   - `test_find_contact_by_email()`
   - `test_find_contact_by_name()`
   - `test_find_contact_fuzzy_match()`
3. Add tests for `ContactEnrichmentTools`:
   - `test_enrich_contact_basic()`
   - `test_enrich_contact_merge_fields()`
4. Add tests for `HistoryTools`:
   - `test_get_timeline()`
   - `test_get_reminders()`
5. Document baseline performance in `BASELINE_PERFORMANCE.md`

**Files Created:**
- `tests/characterization/test_discovery.rs`
- `tests/characterization/test_enrichment.rs`
- `tests/characterization/test_history.rs`
- `BASELINE_PERFORMANCE.md`

**Validation:**
```bash
cargo test --test characterization
```

---

#### Task 1.1.2: Create AsyncDexClient Wrapper
**Priority:** CRITICAL
**Effort:** 3 days
**Assignee:** Developer 1
**Dependencies:** Task 1.1.1

**Description:** Create async wrapper around synchronous DexClient using `tokio::task::spawn_blocking`.

**Acceptance Criteria:**
- [ ] `AsyncDexClient` trait defined with all operations
- [ ] `AsyncDexClientImpl` wraps `DexClient` with `spawn_blocking`
- [ ] All methods return async futures
- [ ] Error handling preserves context
- [ ] Unit tests verify non-blocking behavior
- [ ] Documentation explains async/sync boundary

**Implementation Steps:**

1. Create trait definition:

```rust
// src/client/async_wrapper.rs

use async_trait::async_trait;
use std::sync::Arc;
use crate::client::DexClient;
use crate::models::*;
use crate::error::DexApiResult;

/// Async wrapper trait for CRM client operations.
///
/// This trait provides async versions of all DexClient methods,
/// internally using `tokio::task::spawn_blocking` to avoid
/// blocking the async runtime with synchronous HTTP calls.
#[async_trait]
pub trait AsyncDexClient: Send + Sync {
    async fn get_contact(&self, id: &str) -> DexApiResult<Contact>;
    async fn get_contacts(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>>;
    async fn search_contacts_by_email(&self, email: &str, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>>;
    async fn search_contacts_by_name(&self, query: &str, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>>;

    async fn get_contact_notes(&self, contact_id: &str, limit: usize, offset: usize) -> DexApiResult<Vec<Note>>;
    async fn get_contact_reminders(&self, contact_id: &str, limit: usize, offset: usize) -> DexApiResult<Vec<Reminder>>;

    async fn create_contact(&self, contact: &Contact) -> DexApiResult<Contact>;
    async fn update_contact(&self, id: &str, contact: &Contact) -> DexApiResult<Contact>;
    async fn delete_contact(&self, id: &str) -> DexApiResult<()>;

    async fn create_note(&self, note: &Note) -> DexApiResult<Note>;
    async fn update_note(&self, id: &str, note: &Note) -> DexApiResult<Note>;
    async fn delete_note(&self, id: &str) -> DexApiResult<()>;

    async fn create_reminder(&self, reminder: &Reminder) -> DexApiResult<Reminder>;
    async fn update_reminder(&self, id: &str, reminder: &Reminder) -> DexApiResult<Reminder>;
    async fn delete_reminder(&self, id: &str) -> DexApiResult<()>;
}
```

2. Implement wrapper:

```rust
/// Async wrapper around synchronous DexClient.
///
/// Uses `tokio::task::spawn_blocking` to run synchronous HTTP
/// operations on a dedicated thread pool, preventing blocking
/// the async runtime.
#[derive(Clone)]
pub struct AsyncDexClientImpl {
    client: Arc<DexClient>,
}

impl AsyncDexClientImpl {
    pub fn new(client: DexClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

#[async_trait]
impl AsyncDexClient for AsyncDexClientImpl {
    async fn get_contact(&self, id: &str) -> DexApiResult<Contact> {
        let client = self.client.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            client.get_contact(&id)
        })
        .await
        .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    async fn get_contacts(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>> {
        let client = self.client.clone();

        tokio::task::spawn_blocking(move || {
            client.get_contacts(limit, offset)
        })
        .await
        .map_err(|e| DexApiError::HttpError(format!("Task join error: {}", e)))?
    }

    // ... implement remaining methods similarly
}
```

3. Add to `src/client/mod.rs`:

```rust
mod async_wrapper;
pub use async_wrapper::{AsyncDexClient, AsyncDexClientImpl};
```

4. Update `Cargo.toml`:

```toml
[dependencies]
async-trait = "0.1"
```

**Files Modified/Created:**
- `src/client/async_wrapper.rs` (new)
- `src/client/mod.rs` (export async_wrapper)
- `Cargo.toml` (add async-trait dependency)

**Unit Tests:**

```rust
// tests/unit/test_async_client.rs

#[tokio::test]
async fn test_async_client_does_not_block_runtime() {
    // Create slow mock client
    let client = DexClient::new(/* config with slow endpoint */);
    let async_client = AsyncDexClientImpl::new(client);

    // Launch multiple concurrent requests
    let handles: Vec<_> = (0..10).map(|i| {
        let client = async_client.clone();
        tokio::spawn(async move {
            client.get_contact(&format!("contact_{}", i)).await
        })
    }).collect();

    // All should complete without blocking each other
    let start = std::time::Instant::now();
    for handle in handles {
        let _ = handle.await;
    }
    let duration = start.elapsed();

    // Should take ~1 RTT, not 10 RTTs
    assert!(duration.as_secs() < 5, "Requests blocked each other");
}
```

**Validation:**
```bash
cargo test test_async_client
cargo clippy -- -D warnings
```

---

#### Task 1.1.3: Update Tools to Use AsyncDexClient
**Priority:** CRITICAL
**Effort:** 2 days
**Assignee:** Developer 2
**Dependencies:** Task 1.1.2

**Description:** Refactor all tool implementations to use async client.

**Acceptance Criteria:**
- [ ] All tools accept `Arc<dyn AsyncDexClient>` instead of `Arc<DexClient>`
- [ ] All HTTP calls use `.await`
- [ ] No `spawn_blocking` in tool code (handled by client)
- [ ] All characterization tests still pass
- [ ] No performance regression

**Files Modified:**
- `src/tools/discovery.rs`
- `src/tools/enrichment.rs`
- `src/tools/history.rs`
- `src/server/handlers.rs` (update instantiation)

**Implementation Pattern:**

```rust
// Before:
pub struct ContactDiscoveryTools {
    client: Arc<DexClient>,
    // ...
}

// After:
pub struct ContactDiscoveryTools {
    client: Arc<dyn AsyncDexClient>,
    // ...
}

// Before:
let contacts = self.client.get_contacts(PAGE_SIZE, offset)?;

// After:
let contacts = self.client.get_contacts(PAGE_SIZE, offset).await?;
```

**Validation:**
```bash
cargo test --test characterization
cargo run --release  # Manual smoke test
```

---

#### Task 1.1.4: Add Basic Metrics
**Priority:** HIGH
**Effort:** 1 day
**Assignee:** Developer 1
**Dependencies:** Task 1.1.3

**Description:** Add basic metrics to measure performance improvements.

**Acceptance Criteria:**
- [ ] HTTP request duration tracked
- [ ] HTTP request count tracked
- [ ] Error count tracked
- [ ] Metrics logged to tracing
- [ ] Documentation on viewing metrics

**Implementation:**

```rust
// src/client/async_wrapper.rs

#[async_trait]
impl AsyncDexClient for AsyncDexClientImpl {
    async fn get_contact(&self, id: &str) -> DexApiResult<Contact> {
        let start = std::time::Instant::now();

        let result = /* ... spawn_blocking ... */;

        let duration = start.elapsed();
        tracing::info!(
            method = "get_contact",
            duration_ms = duration.as_millis(),
            success = result.is_ok(),
        );

        result
    }
}
```

**Files Modified:**
- `src/client/async_wrapper.rs`

**Validation:**
```bash
RUST_LOG=info cargo run 2>&1 | grep "method="
```

---

### Milestone 1.2: Search Performance Optimization (Week 3)

#### Task 1.2.1: Extract SearchTools with Cached Index
**Priority:** CRITICAL
**Effort:** 3 days
**Assignee:** Developer 1
**Dependencies:** Milestone 1.1 complete

**Description:** Move full-text search logic from handlers to dedicated SearchTools with cached index.

**Acceptance Criteria:**
- [ ] `SearchTools` struct created
- [ ] Full-text index cached with TTL
- [ ] Cache invalidation on contact modifications
- [ ] Search latency < 200ms for cached index
- [ ] Tests verify caching behavior
- [ ] Metrics track cache hits/misses

**Implementation:**

```rust
// src/tools/search.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::cache::TimedCache;
use crate::search::FullTextSearchIndex;
use crate::client::AsyncDexClient;
use crate::models::*;
use crate::error::DexApiResult;

pub struct SearchTools {
    client: Arc<dyn AsyncDexClient>,
    /// Cached search index
    index_cache: Arc<RwLock<TimedCache<String, Arc<FullTextSearchIndex>>>>,
    cache_ttl_secs: u64,
}

impl SearchTools {
    pub fn new(
        client: Arc<dyn AsyncDexClient>,
        cache_ttl_secs: u64,
    ) -> Self {
        Self {
            client,
            index_cache: Arc::new(RwLock::new(TimedCache::new())),
            cache_ttl_secs,
        }
    }

    pub async fn search_full_text(
        &self,
        query: &str,
        max_results: usize,
        min_confidence: u8,
    ) -> DexApiResult<Vec<SearchResult>> {
        let index = self.get_or_build_index().await?;

        // Search using cached index
        Ok(index.search(query, max_results, min_confidence))
    }

    async fn get_or_build_index(&self) -> DexApiResult<Arc<FullTextSearchIndex>> {
        let cache_key = "full_index".to_string();

        // Try to get from cache
        {
            let cache = self.index_cache.read().await;
            if let Some(cached_index) = cache.get(&cache_key) {
                tracing::debug!("Using cached search index");
                return Ok(cached_index);
            }
        }

        // Build new index
        tracing::info!("Building search index");
        let start = std::time::Instant::now();
        let index = self.build_index().await?;
        let duration = start.elapsed();
        tracing::info!(
            "Search index built in {}ms",
            duration.as_millis()
        );

        let index_arc = Arc::new(index);

        // Cache the index
        {
            let mut cache = self.index_cache.write().await;
            cache.insert_with_ttl(
                cache_key,
                index_arc.clone(),
                self.cache_ttl_secs,
            );
        }

        Ok(index_arc)
    }

    async fn build_index(&self) -> DexApiResult<FullTextSearchIndex> {
        // Fetch all contacts
        let contacts = self.fetch_all_contacts().await?;

        let mut search_index = FullTextSearchIndex::new();

        // For now, index contacts without notes/reminders
        // Will be optimized in Task 1.2.2
        for contact in &contacts {
            search_index.index_contact(contact, &[], &[]);
        }

        Ok(search_index)
    }

    async fn fetch_all_contacts(&self) -> DexApiResult<Vec<Contact>> {
        const PAGE_SIZE: usize = 100;
        let mut all_contacts = Vec::new();
        let mut offset = 0;

        loop {
            let contacts = self.client
                .get_contacts(PAGE_SIZE, offset)
                .await?;

            let count = contacts.len();
            all_contacts.extend(contacts);

            if count < PAGE_SIZE {
                break;
            }
            offset += PAGE_SIZE;
        }

        Ok(all_contacts)
    }

    pub fn invalidate_cache(&self) {
        // For now, synchronous clear
        // Will be improved in Phase 2
        tokio::task::block_in_place(|| {
            self.index_cache.blocking_write().clear();
        });
    }
}
```

**Files Created/Modified:**
- `src/tools/search.rs` (new)
- `src/tools/mod.rs` (export search)
- `src/server/handlers.rs` (use SearchTools)
- `src/cache/timed_cache.rs` (add `insert_with_ttl` if needed)

**Tests:**

```rust
// tests/unit/test_search_tools.rs

#[tokio::test]
async fn test_search_index_cached() {
    let mock_client = Arc::new(MockAsyncDexClient::new());
    let search_tools = SearchTools::new(mock_client.clone(), 300);

    // First search - builds index
    let results1 = search_tools.search_full_text("john", 10, 50).await.unwrap();
    assert_eq!(mock_client.get_contacts_call_count(), 1);

    // Second search - uses cached index
    let results2 = search_tools.search_full_text("john", 10, 50).await.unwrap();
    assert_eq!(mock_client.get_contacts_call_count(), 1); // No additional calls
}
```

**Validation:**
```bash
cargo test test_search_tools
RUST_LOG=debug cargo run  # Verify cache hit logs
```

---

#### Task 1.2.2: Parallelize Note/Reminder Fetching
**Priority:** HIGH
**Effort:** 2 days
**Assignee:** Developer 2
**Dependencies:** Task 1.2.1

**Description:** Fetch notes and reminders concurrently with bounded parallelism.

**Acceptance Criteria:**
- [ ] Notes/reminders fetched in parallel (max 20 concurrent)
- [ ] Index building time reduced by >10x
- [ ] Errors in individual fetches don't fail entire build
- [ ] Metrics track parallel fetch performance
- [ ] Rate limiting handled gracefully

**Implementation:**

```rust
// src/tools/search.rs

use futures::stream::{self, StreamExt};

impl SearchTools {
    async fn build_index(&self) -> DexApiResult<FullTextSearchIndex> {
        let contacts = self.fetch_all_contacts().await?;

        tracing::info!(
            "Fetching notes/reminders for {} contacts",
            contacts.len()
        );

        // Fetch notes/reminders in parallel with bounded concurrency
        let results = stream::iter(contacts.iter())
            .map(|contact| {
                let client = self.client.clone();
                let contact_id = contact.id.clone();

                async move {
                    // Fetch both concurrently
                    let (notes_result, reminders_result) = tokio::join!(
                        client.get_contact_notes(&contact_id, 100, 0),
                        client.get_contact_reminders(&contact_id, 100, 0),
                    );

                    // Don't fail entire build if one contact fails
                    let notes = notes_result.unwrap_or_else(|e| {
                        tracing::warn!(
                            "Failed to fetch notes for contact {}: {}",
                            contact_id,
                            e
                        );
                        Vec::new()
                    });

                    let reminders = reminders_result.unwrap_or_else(|e| {
                        tracing::warn!(
                            "Failed to fetch reminders for contact {}: {}",
                            contact_id,
                            e
                        );
                        Vec::new()
                    });

                    (contact.clone(), notes, reminders)
                }
            })
            .buffer_unordered(20)  // Max 20 concurrent contact fetches
            .collect::<Vec<_>>()
            .await;

        // Build index from results
        let mut search_index = FullTextSearchIndex::new();
        for (contact, notes, reminders) in results {
            search_index.index_contact(&contact, &notes, &reminders);
        }

        Ok(search_index)
    }
}
```

**Files Modified:**
- `src/tools/search.rs`
- `Cargo.toml` (add futures dependency)

**Cargo.toml:**
```toml
[dependencies]
futures = "0.3"
```

**Validation:**
```bash
cargo test test_search_parallel_fetch
# Measure index build time
RUST_LOG=info cargo run -- search-contacts-full-text
```

---

### Milestone 1.3: Performance Validation (Week 4)

#### Task 1.3.1: Performance Benchmarks
**Priority:** MEDIUM
**Effort:** 2 days
**Assignee:** Developer 1
**Dependencies:** Milestone 1.2 complete

**Description:** Create benchmarks to measure and validate performance improvements.

**Acceptance Criteria:**
- [ ] Benchmarks for search with varying dataset sizes
- [ ] Benchmarks for cache hit vs miss
- [ ] Memory usage profiling
- [ ] Results compared to baseline
- [ ] Documentation of improvements

**Implementation:**

```rust
// benches/search_benchmarks.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use dex_mcp_server_rust::*;

fn bench_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_client = create_mock_client_with_contacts(1000);
    let search_tools = SearchTools::new(Arc::new(mock_client), 300);

    let mut group = c.benchmark_group("search");

    // Benchmark first search (cache miss)
    group.bench_function("search_cache_miss", |b| {
        b.to_async(&rt).iter(|| async {
            search_tools.invalidate_cache();
            search_tools.search_full_text("john", 10, 50).await.unwrap()
        });
    });

    // Benchmark subsequent search (cache hit)
    group.bench_function("search_cache_hit", |b| {
        b.to_async(&rt).iter(|| async {
            search_tools.search_full_text("john", 10, 50).await.unwrap()
        });
    });

    group.finish();
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
```

**Files Created:**
- `benches/search_benchmarks.rs`
- `PERFORMANCE_RESULTS.md`

**Cargo.toml:**
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }

[[bench]]
name = "search_benchmarks"
harness = false
```

**Validation:**
```bash
cargo bench
cat PERFORMANCE_RESULTS.md
```

---

#### Task 1.3.2: Integration Testing
**Priority:** HIGH
**Effort:** 2 days
**Assignee:** Developer 2
**Dependencies:** Milestone 1.2 complete

**Description:** Comprehensive integration tests for refactored code.

**Acceptance Criteria:**
- [ ] End-to-end tests for all MCP tools
- [ ] Tests verify async behavior
- [ ] Tests verify caching behavior
- [ ] Tests verify error handling
- [ ] All tests pass consistently

**Files Created:**
- `tests/integration/test_e2e_search.rs`
- `tests/integration/test_concurrent_requests.rs`

**Validation:**
```bash
cargo test --test integration
```

---

#### Task 1.3.3: Documentation Update
**Priority:** MEDIUM
**Effort:** 1 day
**Assignee:** Developer 1
**Dependencies:** Tasks 1.3.1, 1.3.2

**Description:** Update documentation to reflect Phase 1 changes.

**Acceptance Criteria:**
- [ ] README updated with performance characteristics
- [ ] Architecture diagrams updated
- [ ] CHANGELOG.md entry added
- [ ] Migration guide for downstream users

**Files Modified/Created:**
- `README.md`
- `CHANGELOG.md`
- `docs/PHASE1_IMPROVEMENTS.md`

---

## Phase 2: Architectural Improvements (Weeks 5-8)

**Goal:** Introduce proper architectural boundaries with repository pattern and service layer to improve testability and maintainability.

**Risk Level:** 🟡 **MEDIUM** - Requires careful refactoring

---

### Milestone 2.1: Repository Pattern (Week 5-6)

#### Task 2.1.1: Define Repository Traits
**Priority:** HIGH
**Effort:** 2 days
**Assignee:** Developer 1
**Dependencies:** Phase 1 complete

**Description:** Define clean repository trait abstractions.

**Acceptance Criteria:**
- [ ] Repository traits defined for all entities
- [ ] Traits are async with proper error types
- [ ] Documentation includes usage examples
- [ ] Traits support all current operations
- [ ] `async_trait` used consistently

**Implementation:**

```rust
// src/repositories/traits.rs

use async_trait::async_trait;
use crate::models::*;
use crate::error::DexApiResult;

/// Repository for managing contacts.
///
/// Provides abstraction over contact storage and retrieval,
/// enabling different implementations (API client, mock, cached).
#[async_trait]
pub trait ContactRepository: Send + Sync {
    /// Retrieve a single contact by ID.
    async fn get(&self, id: &str) -> DexApiResult<Contact>;

    /// Retrieve multiple contacts with pagination.
    async fn list(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>>;

    /// Search contacts by email address.
    async fn search_by_email(&self, email: &str, limit: usize, offset: usize)
        -> DexApiResult<Vec<Contact>>;

    /// Search contacts by name.
    async fn search_by_name(&self, query: &str, limit: usize, offset: usize)
        -> DexApiResult<Vec<Contact>>;

    /// Create a new contact.
    async fn create(&self, contact: &Contact) -> DexApiResult<Contact>;

    /// Update an existing contact.
    async fn update(&self, id: &str, contact: &Contact) -> DexApiResult<Contact>;

    /// Delete a contact.
    async fn delete(&self, id: &str) -> DexApiResult<()>;
}

/// Repository for managing notes.
#[async_trait]
pub trait NoteRepository: Send + Sync {
    /// Get notes for a specific contact.
    async fn get_for_contact(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Note>>;

    /// Create a new note.
    async fn create(&self, note: &Note) -> DexApiResult<Note>;

    /// Update an existing note.
    async fn update(&self, id: &str, note: &Note) -> DexApiResult<Note>;

    /// Delete a note.
    async fn delete(&self, id: &str) -> DexApiResult<()>;
}

/// Repository for managing reminders.
#[async_trait]
pub trait ReminderRepository: Send + Sync {
    /// Get reminders for a specific contact.
    async fn get_for_contact(
        &self,
        contact_id: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Reminder>>;

    /// Create a new reminder.
    async fn create(&self, reminder: &Reminder) -> DexApiResult<Reminder>;

    /// Update an existing reminder.
    async fn update(&self, id: &str, reminder: &Reminder) -> DexApiResult<Reminder>;

    /// Delete a reminder.
    async fn delete(&self, id: &str) -> DexApiResult<()>;
}
```

**Files Created:**
- `src/repositories/mod.rs`
- `src/repositories/traits.rs`

**Validation:**
```bash
cargo check
cargo doc --no-deps --open  # Verify trait documentation
```

---

#### Task 2.1.2: Implement DexClient Repositories
**Priority:** HIGH
**Effort:** 3 days
**Assignee:** Developer 2
**Dependencies:** Task 2.1.1

**Description:** Implement repository traits using AsyncDexClient.

**Acceptance Criteria:**
- [ ] All repository traits implemented
- [ ] Repositories delegate to AsyncDexClient
- [ ] No business logic in repositories
- [ ] Error handling consistent
- [ ] Unit tests with mock client

**Implementation:**

```rust
// src/repositories/dex_contact_repository.rs

use async_trait::async_trait;
use std::sync::Arc;
use crate::client::AsyncDexClient;
use crate::repositories::traits::ContactRepository;
use crate::models::Contact;
use crate::error::DexApiResult;

/// Contact repository implementation using Dex API client.
pub struct DexContactRepository {
    client: Arc<dyn AsyncDexClient>,
}

impl DexContactRepository {
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
        self.client.search_contacts_by_email(email, limit, offset).await
    }

    async fn search_by_name(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> DexApiResult<Vec<Contact>> {
        self.client.search_contacts_by_name(query, limit, offset).await
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
```

**Files Created:**
- `src/repositories/dex_contact_repository.rs`
- `src/repositories/dex_note_repository.rs`
- `src/repositories/dex_reminder_repository.rs`

**Tests:**

```rust
// tests/unit/test_repositories.rs

#[tokio::test]
async fn test_contact_repository_get() {
    let mock_client = Arc::new(MockAsyncDexClient::new());
    let repo = DexContactRepository::new(mock_client.clone());

    mock_client.expect_get_contact("123", Ok(sample_contact()));

    let contact = repo.get("123").await.unwrap();
    assert_eq!(contact.id, "123");
}
```

**Validation:**
```bash
cargo test test_repositories
```

---

#### Task 2.1.3: Create Mock Repositories for Testing
**Priority:** HIGH
**Effort:** 2 days
**Assignee:** Developer 1
**Dependencies:** Task 2.1.1

**Description:** Create mock repository implementations for testing.

**Acceptance Criteria:**
- [ ] Mock implementations for all repository traits
- [ ] Mocks support configurable responses
- [ ] Mocks track method calls
- [ ] Easy to use in tests
- [ ] Example tests demonstrating usage

**Implementation:**

```rust
// tests/mocks/mock_contact_repository.rs

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use dex_mcp_server_rust::repositories::traits::ContactRepository;
use dex_mcp_server_rust::models::Contact;
use dex_mcp_server_rust::error::{DexApiResult, DexApiError};

/// Mock contact repository for testing.
pub struct MockContactRepository {
    contacts: Arc<Mutex<HashMap<String, Contact>>>,
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl MockContactRepository {
    pub fn new() -> Self {
        Self {
            contacts: Arc::new(Mutex::new(HashMap::new())),
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_contact(&self, contact: Contact) {
        let mut contacts = self.contacts.lock().unwrap();
        contacts.insert(contact.id.clone(), contact);
    }

    pub fn get_call_count(&self, method: &str) -> usize {
        let counts = self.call_counts.lock().unwrap();
        *counts.get(method).unwrap_or(&0)
    }

    fn track_call(&self, method: &str) {
        let mut counts = self.call_counts.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
}

#[async_trait]
impl ContactRepository for MockContactRepository {
    async fn get(&self, id: &str) -> DexApiResult<Contact> {
        self.track_call("get");

        let contacts = self.contacts.lock().unwrap();
        contacts.get(id)
            .cloned()
            .ok_or_else(|| DexApiError::NotFound(format!("Contact {} not found", id)))
    }

    async fn list(&self, limit: usize, offset: usize) -> DexApiResult<Vec<Contact>> {
        self.track_call("list");

        let contacts = self.contacts.lock().unwrap();
        let result: Vec<Contact> = contacts.values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(result)
    }

    // ... implement other methods
}
```

**Files Created:**
- `tests/mocks/mod.rs`
- `tests/mocks/mock_contact_repository.rs`
- `tests/mocks/mock_note_repository.rs`
- `tests/mocks/mock_reminder_repository.rs`

**Example Test:**

```rust
// tests/unit/test_with_mocks.rs

#[tokio::test]
async fn test_service_with_mock_repository() {
    let mock_repo = Arc::new(MockContactRepository::new());
    mock_repo.add_contact(sample_contact());

    let service = ContactService::new(mock_repo.clone());

    let contact = service.find_by_id("123").await.unwrap();

    assert_eq!(mock_repo.get_call_count("get"), 1);
}
```

**Validation:**
```bash
cargo test test_with_mocks
```

---

#### Task 2.1.4: Refactor Tools to Use Repositories
**Priority:** HIGH
**Effort:** 3 days
**Assignee:** Developer 2
**Dependencies:** Tasks 2.1.2, 2.1.3

**Description:** Update all tool implementations to use repository traits.

**Acceptance Criteria:**
- [ ] All tools depend on repository traits, not concrete types
- [ ] No direct AsyncDexClient usage in tools
- [ ] All tests updated to use mock repositories
- [ ] No functionality regression
- [ ] Integration tests still pass

**Before/After:**

```rust
// Before:
pub struct ContactDiscoveryTools {
    client: Arc<dyn AsyncDexClient>,
    // ...
}

// After:
pub struct ContactDiscoveryTools {
    contact_repo: Arc<dyn ContactRepository>,
    note_repo: Arc<dyn NoteRepository>,
    // ...
}
```

**Files Modified:**
- `src/tools/discovery.rs`
- `src/tools/enrichment.rs`
- `src/tools/history.rs`
- `src/tools/search.rs`
- `src/server/handlers.rs`

**Validation:**
```bash
cargo test
cargo clippy -- -D warnings
```

---

### Milestone 2.2: Service Layer (Week 7-8)

#### Task 2.2.1: Create Application Services
**Priority:** MEDIUM
**Effort:** 4 days
**Assignee:** Developer 1
**Dependencies:** Milestone 2.1 complete

**Description:** Extract business logic into application service layer.

**Acceptance Criteria:**
- [ ] Service classes for each major use case
- [ ] Services depend on repository traits
- [ ] No business logic in handlers
- [ ] Services are testable with mock repositories
- [ ] Comprehensive service unit tests

**Implementation:**

```rust
// src/application/contact_service.rs

use std::sync::Arc;
use crate::repositories::traits::{ContactRepository, NoteRepository};
use crate::error::DexApiResult;
use crate::models::*;
use crate::matching::fuzzy_matcher::FuzzyMatcher;

/// Application service for contact-related operations.
///
/// Orchestrates business logic using repositories.
pub struct ContactService {
    contact_repo: Arc<dyn ContactRepository>,
    note_repo: Arc<dyn NoteRepository>,
    matcher: FuzzyMatcher,
}

impl ContactService {
    pub fn new(
        contact_repo: Arc<dyn ContactRepository>,
        note_repo: Arc<dyn NoteRepository>,
    ) -> Self {
        Self {
            contact_repo,
            note_repo,
            matcher: FuzzyMatcher::new(),
        }
    }

    /// Find contacts matching the given criteria.
    pub async fn find_contacts(
        &self,
        params: FindContactParams,
    ) -> DexApiResult<FindContactResult> {
        // Business logic for finding contacts

        // Try exact email match first
        if let Some(email) = &params.email {
            let results = self.contact_repo
                .search_by_email(email, 10, 0)
                .await?;

            if !results.is_empty() {
                return Ok(FindContactResult {
                    matches: results.into_iter()
                        .map(|c| MatchResult {
                            contact: c,
                            confidence: 100,
                            match_type: MatchType::EmailExact,
                        })
                        .collect(),
                    from_cache: false,
                });
            }
        }

        // Fall back to fuzzy name search
        if let Some(name) = &params.name {
            let all_contacts = self.fetch_all_contacts().await?;
            let matches = self.matcher.find_matches(&all_contacts, name, params.min_confidence);

            return Ok(FindContactResult {
                matches,
                from_cache: false,
            });
        }

        Ok(FindContactResult {
            matches: Vec::new(),
            from_cache: false,
        })
    }

    async fn fetch_all_contacts(&self) -> DexApiResult<Vec<Contact>> {
        // Pagination logic
        const PAGE_SIZE: usize = 100;
        let mut all_contacts = Vec::new();
        let mut offset = 0;

        loop {
            let contacts = self.contact_repo.list(PAGE_SIZE, offset).await?;
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

pub struct FindContactParams {
    pub email: Option<String>,
    pub name: Option<String>,
    pub min_confidence: u8,
}

pub struct FindContactResult {
    pub matches: Vec<MatchResult>,
    pub from_cache: bool,
}

pub struct MatchResult {
    pub contact: Contact,
    pub confidence: u8,
    pub match_type: MatchType,
}

pub enum MatchType {
    EmailExact,
    NameFuzzy,
}
```

**Files Created:**
- `src/application/mod.rs`
- `src/application/contact_service.rs`
- `src/application/search_service.rs`
- `src/application/enrichment_service.rs`

**Tests:**

```rust
// tests/unit/test_contact_service.rs

#[tokio::test]
async fn test_find_contacts_by_email() {
    let mock_contact_repo = Arc::new(MockContactRepository::new());
    let mock_note_repo = Arc::new(MockNoteRepository::new());

    mock_contact_repo.add_contact(sample_contact_with_email("john@example.com"));

    let service = ContactService::new(mock_contact_repo, mock_note_repo);

    let result = service.find_contacts(FindContactParams {
        email: Some("john@example.com".to_string()),
        name: None,
        min_confidence: 50,
    }).await.unwrap();

    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].confidence, 100);
}
```

**Validation:**
```bash
cargo test test_contact_service
```

---

#### Task 2.2.2: Refactor Handlers to Use Services
**Priority:** MEDIUM
**Effort:** 3 days
**Assignee:** Developer 2
**Dependencies:** Task 2.2.1

**Description:** Make MCP handlers thin adapters that delegate to services.

**Acceptance Criteria:**
- [ ] Handlers only handle protocol concerns
- [ ] All business logic in services
- [ ] Handlers format service responses to MCP format
- [ ] No duplication between handlers
- [ ] Integration tests still pass

**Before/After:**

```rust
// Before: Business logic in handler
impl DexMcpServer {
    async fn search_contacts_full_text(...) -> Result<CallToolResult, McpError> {
        // Build index
        let mut search_index = FullTextSearchIndex::new();
        let contacts = self.fetch_all_contacts().await?;
        for contact in &contacts {
            let notes = self.client.get_contact_notes(&contact.id, 100, 0)?;
            // ... more business logic
        }
        // Search
        let results = search_index.search(...);
        // Format
        let response = format_results(results);
        Ok(CallToolResult::success(vec![Content::text(response)]))
    }
}

// After: Thin protocol adapter
impl DexMcpServer {
    async fn search_contacts_full_text(...) -> Result<CallToolResult, McpError> {
        let params = params.0;

        // Delegate to service
        let results = self.search_service
            .search_full_text(&params.query, params.max_results, params.min_confidence)
            .await
            .map_err(to_mcp_error)?;

        // Format to MCP response
        let response = format_search_results(results);
        Ok(CallToolResult::success(vec![Content::text(response)]))
    }
}
```

**Files Modified:**
- `src/server/handlers.rs`
- `src/server/formatters.rs` (new - extract formatting logic)

**Validation:**
```bash
cargo test --test integration
```

---

#### Task 2.2.3: Improve Error Handling
**Priority:** MEDIUM
**Effort:** 2 days
**Assignee:** Developer 1
**Dependencies:** Task 2.2.2

**Description:** Map specific errors to appropriate MCP error codes.

**Acceptance Criteria:**
- [ ] Each DexApiError maps to specific MCP error code
- [ ] Error context preserved
- [ ] Structured error data included
- [ ] Client can distinguish error types
- [ ] Error handling documented

**Implementation:**

```rust
// src/error.rs - Add method

impl DexApiError {
    pub fn to_mcp_error_code(&self) -> i64 {
        match self {
            Self::NotFound(_) => -32001,
            Self::Unauthorized(_) => -32002,
            Self::RateLimited { .. } => -32003,
            Self::ValidationError(_) => -32004,
            Self::HttpError(_) => ErrorCode::INTERNAL_ERROR,
            Self::SerializationError(_) => ErrorCode::PARSE_ERROR,
        }
    }

    pub fn to_mcp_error_data(&self) -> Option<serde_json::Value> {
        match self {
            Self::RateLimited { retry_after } => {
                Some(serde_json::json!({
                    "retry_after": retry_after
                }))
            }
            Self::ValidationError(msg) => {
                Some(serde_json::json!({
                    "validation_error": msg
                }))
            }
            _ => None,
        }
    }
}
```

```rust
// src/server/handlers.rs - Update error mapper

fn to_mcp_error(e: DexApiError) -> McpError {
    McpError {
        code: e.to_mcp_error_code(),
        message: Cow::from(e.to_string()),
        data: e.to_mcp_error_data(),
    }
}
```

**Files Modified:**
- `src/error.rs`
- `src/server/handlers.rs`

**Tests:**

```rust
// tests/unit/test_error_mapping.rs

#[test]
fn test_not_found_maps_correctly() {
    let error = DexApiError::NotFound("Contact not found".to_string());
    assert_eq!(error.to_mcp_error_code(), -32001);
}

#[test]
fn test_rate_limit_includes_retry_after() {
    let error = DexApiError::RateLimited { retry_after: 60 };
    let data = error.to_mcp_error_data().unwrap();
    assert_eq!(data["retry_after"], 60);
}
```

**Validation:**
```bash
cargo test test_error_mapping
```

---

## Phase 3: Code Quality & Optimization (Weeks 9-10)

**Goal:** Reduce memory usage, improve type safety, and add production monitoring.

**Risk Level:** 🟢 **LOW** - Incremental improvements

---

### Milestone 3.1: Memory Optimization (Week 9)

#### Task 3.1.1: Use Arc for Shared Contact References
**Priority:** MEDIUM
**Effort:** 3 days
**Assignee:** Developer 1
**Dependencies:** Phase 2 complete

**Description:** Replace Contact clones with Arc references.

**Acceptance Criteria:**
- [ ] SearchResult uses Arc<Contact>
- [ ] Cache stores Arc<Contact>
- [ ] Service results use Arc<Contact>
- [ ] Memory usage reduced by >30%
- [ ] No performance regression

**Implementation:**

```rust
// src/models/contact.rs
pub type ContactRef = Arc<Contact>;

// src/search/full_text_index.rs
pub struct SearchResult {
    pub contact: ContactRef,  // Instead of Contact
    pub matches: Vec<MatchContext>,
    pub confidence: u8,
}

// src/cache/timed_cache.rs
pub struct TimedCache<K, V> {
    cache: Arc<RwLock<HashMap<K, CacheEntry<Arc<V>>>>>,  // Store Arc<V>
    ttl_secs: u64,
}

impl<K, V> TimedCache<K, V> {
    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        // Return Arc, no clone
    }

    pub fn insert(&self, key: K, value: Arc<V>) {
        // Store Arc
    }
}
```

**Files Modified:**
- `src/models/contact.rs`
- `src/search/full_text_index.rs`
- `src/cache/timed_cache.rs`
- `src/application/*.rs`
- `src/server/handlers.rs`

**Memory Profiling:**

```bash
# Before
cargo run --release &
PID=$!
pmap $PID | tail -1

# After
# Compare memory usage
```

**Validation:**
```bash
cargo test
# Memory profiling comparison
```

---

#### Task 3.1.2: Add Value Objects
**Priority:** LOW
**Effort:** 2 days
**Assignee:** Developer 2
**Dependencies:** Task 3.1.1

**Description:** Create type-safe value objects for domain concepts.

**Acceptance Criteria:**
- [ ] ContactId newtype with validation
- [ ] EmailAddress newtype with validation
- [ ] PhoneNumber newtype with validation
- [ ] Contact uses value objects
- [ ] Validation at construction
- [ ] Serde integration works

**Implementation:**

```rust
// src/domain/contact_id.rs

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContactId(String);

impl ContactId {
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ValidationError::EmptyId);
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Serde support
impl Serialize for ContactId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ContactId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ContactId::new(s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for ContactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

**Files Created:**
- `src/domain/mod.rs`
- `src/domain/contact_id.rs`
- `src/domain/email.rs`
- `src/domain/phone.rs`
- `src/domain/errors.rs`

**Tests:**

```rust
#[test]
fn test_contact_id_rejects_empty() {
    assert!(ContactId::new("").is_err());
}

#[test]
fn test_email_validates_format() {
    assert!(EmailAddress::new("invalid").is_err());
    assert!(EmailAddress::new("valid@example.com").is_ok());
}
```

**Validation:**
```bash
cargo test test_value_objects
```

---

### Milestone 3.2: Production Monitoring (Week 10)

#### Task 3.2.1: Add Comprehensive Metrics
**Priority:** MEDIUM
**Effort:** 2 days
**Assignee:** Developer 1
**Dependencies:** None

**Description:** Instrument code with production-grade metrics.

**Acceptance Criteria:**
- [ ] HTTP request metrics (count, duration, errors)
- [ ] Cache metrics (hits, misses, size)
- [ ] Search metrics (queries, results, duration)
- [ ] Repository operation metrics
- [ ] Metrics exportable to Prometheus

**Implementation:**

```rust
// Cargo.toml
[dependencies]
metrics = "0.21"
metrics-exporter-prometheus = "0.12"

// src/observability/metrics.rs

use metrics::{counter, histogram, gauge};

pub fn track_http_request(method: &str, duration_ms: u128, success: bool) {
    counter!("dex_api_requests_total", "method" => method).increment(1);

    histogram!(
        "dex_api_request_duration_seconds",
        "method" => method
    ).record(duration_ms as f64 / 1000.0);

    if !success {
        counter!("dex_api_errors_total", "method" => method).increment(1);
    }
}

pub fn track_cache_access(cache_type: &str, hit: bool) {
    if hit {
        counter!("cache_hits_total", "cache" => cache_type).increment(1);
    } else {
        counter!("cache_misses_total", "cache" => cache_type).increment(1);
    }
}

pub fn track_search_query(duration_ms: u128, result_count: usize) {
    counter!("search_queries_total").increment(1);
    histogram!("search_duration_seconds").record(duration_ms as f64 / 1000.0);
    histogram!("search_result_count").record(result_count as f64);
}

pub fn update_cache_size(cache_type: &str, size: usize) {
    gauge!("cache_size_entries", "cache" => cache_type).set(size as f64);
}
```

**Files Created:**
- `src/observability/mod.rs`
- `src/observability/metrics.rs`

**Files Modified:**
- `src/client/async_wrapper.rs` (add metrics)
- `src/cache/timed_cache.rs` (add metrics)
- `src/application/*.rs` (add metrics)

**Validation:**
```bash
# Run server with metrics
cargo run --release

# Check metrics endpoint
curl http://localhost:9090/metrics
```

---

#### Task 3.2.2: Add Structured Logging
**Priority:** LOW
**Effort:** 1 day
**Assignee:** Developer 2
**Dependencies:** None

**Description:** Improve logging with structured fields.

**Acceptance Criteria:**
- [ ] All log statements use structured fields
- [ ] Log levels used appropriately
- [ ] No sensitive data logged
- [ ] Correlation IDs for request tracing
- [ ] Log format documented

**Implementation:**

```rust
// Use structured logging throughout

// Before:
tracing::info!("Fetching contact {}", id);

// After:
tracing::info!(
    contact_id = %id,
    "Fetching contact"
);

// With context:
tracing::error!(
    error = %e,
    contact_id = %id,
    operation = "enrich_contact",
    "Failed to enrich contact"
);
```

**Validation:**
```bash
RUST_LOG=debug cargo run 2>&1 | grep "contact_id="
```

---

#### Task 3.2.3: Documentation and Deployment Guide
**Priority:** MEDIUM
**Effort:** 2 days
**Assignee:** Developer 1
**Dependencies:** All previous tasks

**Description:** Comprehensive documentation for deployment and operations.

**Acceptance Criteria:**
- [ ] Deployment guide with examples
- [ ] Configuration reference
- [ ] Monitoring guide
- [ ] Troubleshooting guide
- [ ] Performance tuning guide
- [ ] API examples

**Files Created:**
- `docs/DEPLOYMENT.md`
- `docs/CONFIGURATION.md`
- `docs/MONITORING.md`
- `docs/TROUBLESHOOTING.md`
- `docs/PERFORMANCE_TUNING.md`
- `docs/API_EXAMPLES.md`

**Validation:**
- [ ] Follow deployment guide on clean machine
- [ ] Verify all examples work

---

#### Task 3.2.4: Final Testing and Validation
**Priority:** HIGH
**Effort:** 2 days
**Assignee:** All developers
**Dependencies:** All previous tasks

**Description:** Comprehensive testing of all improvements.

**Acceptance Criteria:**
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Performance benchmarks meet targets
- [ ] Memory usage meets targets
- [ ] Load testing completed
- [ ] Documentation reviewed

**Test Plan:**

1. **Unit Tests:**
   ```bash
   cargo test --lib
   ```

2. **Integration Tests:**
   ```bash
   cargo test --test integration
   ```

3. **Performance Tests:**
   ```bash
   cargo bench
   ```

4. **Load Testing:**
   ```bash
   # Use k6 or similar
   k6 run load_test.js
   ```

5. **Memory Profiling:**
   ```bash
   valgrind --tool=massif cargo run --release
   ```

**Success Criteria Validation:**

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Search response time (cached) | <200ms | TBD | |
| Request throughput | >100 req/s | TBD | |
| Memory usage (10k contacts) | <100MB | TBD | |
| Test coverage | >80% | TBD | |
| Error rate | <0.1% | TBD | |

---

## Rollback Strategy

Each phase is designed to be independently reversible:

### Phase 1 Rollback
- Revert to original handlers
- Remove AsyncDexClient wrapper
- Keep characterization tests

### Phase 2 Rollback
- Keep repository traits
- Revert tools to use AsyncDexClient directly
- Remove service layer
- Keep improved error handling

### Phase 3 Rollback
- Revert Arc changes
- Remove value objects
- Keep metrics and logging

### Git Strategy
- Each phase in separate branch
- Merge to main only after validation
- Tag releases: `v1.0-phase1`, `v1.0-phase2`, `v1.0-phase3`

---

## Risk Mitigation

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| spawn_blocking overhead | Low | Medium | Benchmark early; adjust if needed |
| Lifetime complexity with Arc | Medium | Low | Use Arc everywhere; avoid lifetimes |
| Cache invalidation bugs | Medium | High | Comprehensive tests; conservative TTL |
| Breaking downstream users | Low | High | Maintain backward compatibility |
| Performance regression | Low | High | Continuous benchmarking |

### Schedule Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Underestimated complexity | Medium | Medium | 20% buffer built in |
| Developer unavailability | Low | High | Cross-training; documentation |
| Scope creep | Medium | Medium | Strict phase boundaries |
| Integration issues | Low | Medium | Early integration testing |

---

## Dependencies and Prerequisites

### Before Starting

- [ ] All team members familiar with async Rust
- [ ] Development environment set up
- [ ] CI/CD pipeline configured
- [ ] Testing infrastructure ready
- [ ] Monitoring system available

### External Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| tokio | 1.x | Async runtime |
| async-trait | 0.1 | Async trait support |
| futures | 0.3 | Stream utilities |
| metrics | 0.21 | Metrics collection |
| criterion | 0.5 | Benchmarking |

---

## Success Metrics and KPIs

### Performance KPIs

| Metric | Baseline | Phase 1 Target | Phase 2 Target | Final Target |
|--------|----------|----------------|----------------|--------------|
| P50 search latency | >5s | <500ms | <300ms | <200ms |
| P99 search latency | >10s | <2s | <1s | <500ms |
| Throughput | <10 req/s | >50 req/s | >75 req/s | >100 req/s |
| Memory (10k contacts) | ~500MB | <200MB | <150MB | <100MB |
| Cache hit rate | 0% | >80% | >90% | >95% |

### Quality KPIs

| Metric | Baseline | Target |
|--------|----------|--------|
| Test coverage | ~40% | >80% |
| Clippy warnings | TBD | 0 |
| Documentation coverage | ~50% | >90% |
| TODO/FIXME comments | TBD | <10 |

### Development KPIs

| Metric | Target |
|--------|--------|
| Code review turnaround | <24h |
| CI pipeline duration | <10min |
| Test execution time | <2min |
| Build time (clean) | <5min |

---

## Communication Plan

### Weekly Status Updates

Every Friday:
- Progress against plan
- Completed tasks
- Blockers and risks
- Next week's goals

### Milestone Reviews

After each milestone:
- Demo of functionality
- Performance measurements
- Test coverage report
- Updated risk assessment

### Stakeholder Communication

- **Daily:** Team standup
- **Weekly:** Status report to stakeholders
- **Bi-weekly:** Demo to product team
- **Phase completion:** Architecture review

---

## Appendix

### A. Glossary

- **MCP:** Model Context Protocol
- **spawn_blocking:** Tokio mechanism for running blocking code
- **Repository Pattern:** Abstraction over data access
- **Service Layer:** Business logic orchestration
- **Value Object:** Immutable type with validation

### B. References

- [Architecture Review](architecture-review.md)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [MCP Specification](https://modelcontextprotocol.io/)
- [Tokio Documentation](https://tokio.rs/)

### C. Template Files

See `templates/` directory for:
- Test template
- Service template
- Repository template
- Benchmark template

---

**Plan Created:** 2025-11-09
**Plan Owner:** Development Team
**Review Date:** End of each phase
**Next Review:** End of Week 2 (Milestone 1.1)

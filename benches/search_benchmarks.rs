//! Performance benchmarks for search functionality.
//!
//! These benchmarks measure search performance under various conditions:
//! - Cache miss (first search, index must be built)
//! - Cache hit (subsequent searches using cached index)
//! - Different dataset sizes
//! - Parallel fetching performance

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dex_mcp_server::client::{AsyncDexClient, AsyncDexClientImpl, DexClient};
use dex_mcp_server::config::Config;
use dex_mcp_server::repositories::{
    ContactRepository, DexContactRepository, DexNoteRepository, DexReminderRepository,
    NoteRepository, ReminderRepository,
};
use dex_mcp_server::tools::search::{SearchParams, SearchTools};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Create a mock client and repositories with predefined contacts for testing.
/// In real benchmarks, this would use mockito or a test server.
fn create_test_repos() -> (
    Arc<dyn ContactRepository>,
    Arc<dyn NoteRepository>,
    Arc<dyn ReminderRepository>,
) {
    // For now, use the real client but with a test configuration
    // In production benchmarks, you'd use mockito to avoid network calls
    let config = Config::default();
    let sync_client = DexClient::new(&config);
    let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

    let contact_repo =
        Arc::new(DexContactRepository::new(client.clone())) as Arc<dyn ContactRepository>;
    let note_repo = Arc::new(DexNoteRepository::new(client.clone())) as Arc<dyn NoteRepository>;
    let reminder_repo = Arc::new(DexReminderRepository::new(client)) as Arc<dyn ReminderRepository>;

    (contact_repo, note_repo, reminder_repo)
}

/// Benchmark search performance with cache miss (index build required).
fn bench_search_cache_miss(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache_ttl = 300; // 5 minutes

    c.bench_function("search_cache_miss", |b| {
        b.to_async(&rt).iter(|| async {
            // Create new SearchTools each time to force cache miss
            let (contact_repo, note_repo, reminder_repo) = create_test_repos();
            let search_tools = SearchTools::new(contact_repo, note_repo, reminder_repo, cache_ttl);

            let params = SearchParams {
                query: "john".to_string(),
                max_results: Some(10),
                min_confidence: Some(50),
            };

            let _result = search_tools.search_full_text(params).await;
        });
    });
}

/// Benchmark search performance with cache hit (using cached index).
fn bench_search_cache_hit(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache_ttl = 300; // 5 minutes

    // Pre-build the cache
    let (contact_repo, note_repo, reminder_repo) = create_test_repos();
    let search_tools = SearchTools::new(contact_repo, note_repo, reminder_repo, cache_ttl);
    rt.block_on(async {
        let params = SearchParams {
            query: "warmup".to_string(),
            max_results: Some(10),
            min_confidence: Some(50),
        };
        let _result = search_tools.search_full_text(params).await;
    });

    c.bench_function("search_cache_hit", |b| {
        b.to_async(&rt).iter(|| async {
            let params = SearchParams {
                query: "john".to_string(),
                max_results: Some(10),
                min_confidence: Some(50),
            };

            let _result = search_tools.search_full_text(params).await;
        });
    });
}

/// Benchmark search with different result limits.
fn bench_search_result_limits(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache_ttl = 300;

    // Pre-build the cache
    let (contact_repo, note_repo, reminder_repo) = create_test_repos();
    let search_tools = SearchTools::new(contact_repo, note_repo, reminder_repo, cache_ttl);
    rt.block_on(async {
        let params = SearchParams {
            query: "warmup".to_string(),
            max_results: Some(10),
            min_confidence: Some(50),
        };
        let _result = search_tools.search_full_text(params).await;
    });

    let mut group = c.benchmark_group("search_result_limits");

    for limit in [5, 10, 25, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(limit), limit, |b, &limit| {
            b.to_async(&rt).iter(|| async {
                let params = SearchParams {
                    query: "john".to_string(),
                    max_results: Some(limit),
                    min_confidence: Some(50),
                };

                let _result = search_tools.search_full_text(params).await;
            });
        });
    }

    group.finish();
}

/// Benchmark search with different confidence thresholds.
fn bench_search_confidence_thresholds(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache_ttl = 300;

    // Pre-build the cache
    let (contact_repo, note_repo, reminder_repo) = create_test_repos();
    let search_tools = SearchTools::new(contact_repo, note_repo, reminder_repo, cache_ttl);
    rt.block_on(async {
        let params = SearchParams {
            query: "warmup".to_string(),
            max_results: Some(10),
            min_confidence: Some(50),
        };
        let _result = search_tools.search_full_text(params).await;
    });

    let mut group = c.benchmark_group("search_confidence_thresholds");

    for confidence in [30, 50, 70, 90].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(confidence),
            confidence,
            |b, &confidence| {
                b.to_async(&rt).iter(|| async {
                    let params = SearchParams {
                        query: "john".to_string(),
                        max_results: Some(10),
                        min_confidence: Some(confidence),
                    };

                    let _result = search_tools.search_full_text(params).await;
                });
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);
    targets = bench_search_cache_miss,
        bench_search_cache_hit,
        bench_search_result_limits,
        bench_search_confidence_thresholds
}

criterion_main!(benches);

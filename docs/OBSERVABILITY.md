# Observability Guide

## Overview

The Dex MCP Server provides comprehensive observability features for production monitoring, including metrics tracking and structured logging.

## Metrics Tracking

### MetricsTracker

The `MetricsTracker` provides real-time metrics collection for monitoring application health and performance.

#### Basic Usage

```rust
use dex_mcp_server::observability::MetricsTracker;

let metrics = MetricsTracker::new();

// Track HTTP requests
metrics.track_http_request("get_contact", 150, true);

// Track cache access
metrics.track_cache_access("contact_cache", true);

// Track search queries
metrics.track_search_query(200, 5);
```

#### Viewing Metrics

```rust
// Get individual metrics
let total_requests = metrics.http_requests_total();
let cache_hit_rate = metrics.cache_hit_rate();
let error_rate = metrics.http_error_rate();

// Print summary
println!("{}", metrics.summary());
```

Example output:

```
Metrics Summary:
HTTP Requests: 1250
HTTP Errors: 3 (0.24% error rate)
Cache Hits: 980
Cache Misses: 120
Cache Hit Rate: 89.09%
Search Queries: 45
```

### Timer Utility

Track operation duration with the `Timer` helper:

```rust
use dex_mcp_server::observability::Timer;

async fn my_operation() {
    let timer = Timer::new("my_operation");

    // ... do work ...

    let duration_ms = timer.finish();
    println!("Operation took {}ms", duration_ms);
}
```

With status tracking:

```rust
let timer = Timer::new("risky_operation");

let result = risky_operation().await;

let duration_ms = timer.finish_with_status(result.is_ok());
```

## Structured Logging

### Log Levels

The application uses `tracing` for structured logging with the following levels:

- `ERROR` - Errors that require immediate attention
- `WARN` - Warning conditions that should be investigated
- `INFO` - Informational messages about normal operation
- `DEBUG` - Detailed diagnostic information
- `TRACE` - Very detailed diagnostic information

### Enabling Logs

Set the `RUST_LOG` environment variable:

```bash
# Show all logs
RUST_LOG=debug cargo run

# Show only info and above
RUST_LOG=info cargo run

# Filter by module
RUST_LOG=dex_mcp_server::search=debug cargo run
```

### Structured Fields

Logs include structured fields for easy filtering and analysis:

```rust
tracing::info!(
    contact_id = %id,
    duration_ms = duration,
    success = true,
    "Contact enrichment completed"
);
```

This produces output like:

```
2025-11-10T10:30:45.123Z INFO dex_mcp_server::tools::enrichment: Contact enrichment completed contact_id="contact_123" duration_ms=150 success=true
```

### Common Fields

- `contact_id` - Contact identifier
- `duration_ms` - Operation duration in milliseconds
- `success` - Operation success status
- `error` - Error message
- `cache_type` - Cache identifier
- `method` - HTTP method or function name

## Monitoring Best Practices

### Production Monitoring

1. **Track HTTP Error Rate**
   - Target: < 0.1%
   - Alert if > 1%

2. **Monitor Cache Hit Rate**
   - Target: > 90%
   - Investigate if < 80%

3. **Watch Search Performance**
   - Target: < 200ms for cached searches
   - Alert if > 500ms

### Logging Best Practices

1. **Use Appropriate Log Levels**
   ```rust
   // Don't log sensitive data
   tracing::debug!(contact_id = %id, "Fetching contact"); // ✓
   tracing::debug!(contact = ?contact, "Fetching contact"); // ✗ (may contain PII)
   ```

2. **Include Context**
   ```rust
   // Good - includes relevant context
   tracing::error!(
       contact_id = %id,
       error = %e,
       "Failed to enrich contact"
   );

   // Bad - missing context
   tracing::error!("Failed to enrich contact");
   ```

3. **Use Structured Fields**
   ```rust
   // Good - structured
   tracing::info!(duration_ms = 150, "Search completed");

   // Bad - string interpolation
   tracing::info!("Search completed in 150ms");
   ```

## Integration with External Systems

### Future: Prometheus Integration

In a future phase, metrics can be exported to Prometheus:

```toml
[dependencies]
metrics-exporter-prometheus = "0.12"
```

```rust
use metrics_exporter_prometheus::PrometheusBuilder;

let builder = PrometheusBuilder::new();
builder.install().expect("failed to install Prometheus exporter");

// Metrics now available at http://localhost:9090/metrics
```

### Future: Distributed Tracing

For distributed tracing with OpenTelemetry:

```toml
[dependencies]
tracing-opentelemetry = "0.21"
opentelemetry-jaeger = "0.20"
```

## Example: Full Observability Setup

```rust
use dex_mcp_server::observability::{MetricsTracker, Timer};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("dex_mcp_server=info")
        .init();

    // Create metrics tracker
    let metrics = Arc::new(MetricsTracker::new());

    // Use in application
    let timer = Timer::new("search_operation");

    let result = perform_search().await;

    let duration_ms = timer.finish_with_status(result.is_ok());

    if let Ok(results) = result {
        metrics.track_search_query(duration_ms, results.len());
    }

    // Periodically print metrics summary
    tokio::spawn({
        let metrics = metrics.clone();
        async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                tracing::info!(metrics = %metrics.summary(), "Metrics summary");
            }
        }
    });
}
```

## Troubleshooting

### No Logs Appearing

Check `RUST_LOG` environment variable:

```bash
echo $RUST_LOG  # Should show filter configuration
```

If not set:

```bash
export RUST_LOG=dex_mcp_server=debug
```

### Metrics Not Updating

Ensure `MetricsTracker` is shared via `Arc`:

```rust
// Correct
let metrics = Arc::new(MetricsTracker::new());
let metrics_clone = metrics.clone();

// Incorrect - creates separate tracker
let metrics = MetricsTracker::new();
```

### High Memory Usage

Check cache sizes and cleanup:

```rust
cache.cleanup_expired(); // Remove expired entries
```

Monitor metrics:

```rust
let summary = metrics.summary();
// Check cache hit rate - low rate may indicate ineffective caching
```

## References

- [Phase 3 Improvements](PHASE3_IMPROVEMENTS.md)
- [tracing Documentation](https://docs.rs/tracing)
- [metrics Documentation](https://docs.rs/metrics)

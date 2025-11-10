# Rust Port Plan for Dex MCP Server

## Executive Summary

This document outlines the detailed plan for porting the TypeScript/Node.js Dex MCP Server to Rust. The existing server provides an MCP (Model Context Protocol) interface to the Dex Personal CRM, enabling AI assistants to intelligently discover contacts, access relationship history, and enrich contact information.

## Project Analysis

### Existing Architecture

The current TypeScript implementation consists of:

1. **Main Server** (`index.ts`): ~560 lines
   - MCP protocol server using `@modelcontextprotocol/sdk`
   - Stdio transport for communication
   - 10 registered tools (contact discovery, history, enrichment, search)
   - Tool handler with switch-case dispatch

2. **Core Components**:
   - **DexClient** (`dex-client.ts`): HTTP API client (~276 lines)
     - Axios-based HTTP client
     - CRUD operations for contacts, notes, reminders
     - Error handling via interceptors
     - 10-second timeout

   - **Contact Discovery** (`tools/discovery.ts`): ~108 lines
     - Smart contact search with fuzzy matching
     - 5-minute in-memory cache
     - Pagination handling
     - Cache invalidation

   - **Relationship History** (`tools/history.ts`): ~122 lines
     - Timeline aggregation (notes + reminders)
     - Filtering by date and type
     - Chronological sorting

   - **Contact Enrichment** (`tools/enrichment.ts`): ~127 lines
     - Smart data merging (arrays merged, not replaced)
     - Note creation
     - Reminder creation

   - **Fuzzy Matcher** (`matching/fuzzy-matcher.ts`): ~262 lines
     - Fuse.js-based fuzzy search
     - Exact matching on email, phone, social URLs
     - String normalization
     - Confidence scoring (0-100)

   - **Full-Text Search Index** (`search/full-text-index.ts`): ~340 lines
     - Document-based indexing
     - Fuse.js full-text search
     - Match context with snippets
     - HTML stripping
     - 30-minute configurable cache TTL

3. **Configuration** (`config.ts`): ~49 lines
   - Manual .env file parsing (avoids stdout pollution)
   - Environment variable handling
   - Validation

4. **Type Definitions** (`types.ts`): ~65 lines
   - TypeScript interfaces for data structures

### Key Features

1. **Contact Discovery**:
   - Fuzzy name matching (handles typos, nicknames, name variations)
   - Exact matching (email, phone, social media URLs)
   - Confidence scoring (0-100 scale)
   - Company-based confidence boosting
   - Top 5 results with ranking

2. **Full-Text Search**:
   - Cross-document search (contacts, notes, reminders)
   - Fuzzy matching with configurable thresholds
   - Match context extraction with snippets
   - Result aggregation by contact
   - Multi-match confidence boosting

3. **Caching Strategy**:
   - Contact cache: 5 minutes TTL
   - Search index cache: 30 minutes TTL (configurable)
   - Automatic cache invalidation on updates
   - In-memory only (no persistence)

4. **Security**:
   - No PII storage on disk
   - HTTPS-only API communication
   - No sensitive data logging
   - Input validation

5. **Error Handling**:
   - Comprehensive error messages
   - Graceful fallbacks (e.g., email search failure → full contact list)
   - API error interception and formatting

### Dependencies

- `@modelcontextprotocol/sdk`: MCP protocol implementation
- `axios`: HTTP client
- `fuse.js`: Fuzzy search library

### Testing Approach

- Unit tests with vitest
- Mock client for isolated testing
- Test fixtures
- Coverage targets: 80% lines/functions, 75% branches

## Technology Mapping: TypeScript → Rust

### Core Crates (Optimized for Lightweight Performance)

| TypeScript Library | Rust Equivalent | Purpose | Notes |
|-------------------|-----------------|---------|-------|
| `@modelcontextprotocol/sdk` | `modelcontextprotocol/rust-sdk` (official) | MCP protocol | Official Rust SDK available |
| `axios` | `ureq` | HTTP client | Sync, lightweight, minimal deps (~40 crates vs reqwest's ~200) |
| `fuse.js` | `nucleo` | Fuzzy string matching | 6-10x faster than alternatives, used in Helix editor |
| `vitest` | Built-in `cargo test` | Testing framework | Native Rust testing |
| N/A (dotenv) | `dotenvy` | Environment variables | Standard choice |
| N/A | `serde` + `sonic-rs` | Serialization | sonic-rs 2-3x faster than serde_json (SIMD) |
| N/A | `tokio` (minimal features) or `smol` | Async runtime | Minimal tokio for MCP SDK, or smol if possible |
| N/A | `thiserror` + `anyhow` | Error handling | thiserror for libs, anyhow for apps |
| N/A | `tracing` | Logging | Structured logging, PII-safe |

### Pattern Mapping

| TypeScript Pattern | Rust Equivalent | Notes |
|-------------------|-----------------|-------|
| `class` with methods | `struct` with `impl` blocks | Similar structure |
| `async/await` | `async/await` | Native in both |
| Optional parameters | `Option<T>` | More explicit in Rust |
| Union types | `enum` | More powerful in Rust |
| Error throwing | `Result<T, E>` | Explicit error handling |
| `Map<K, V>` | `HashMap<K, V>` or `BTreeMap<K, V>` | Similar API |
| Array methods | Iterator chains | More functional |
| `setTimeout` caching | `tokio::time::Instant` | Similar approach |
| JSON parsing | `serde_json::Value` | Similar flexibility |

### Architecture Differences

1. **Memory Management**:
   - TypeScript: Garbage collected
   - Rust: Ownership system → Need careful lifetime management for caches

2. **Error Handling**:
   - TypeScript: Try-catch, exceptions
   - Rust: `Result<T, E>` → More explicit, compile-time checked

3. **Async Runtime**:
   - TypeScript: Built-in event loop
   - Rust: Requires runtime (tokio) → More explicit control

4. **Type System**:
   - TypeScript: Structural typing, gradual
   - Rust: Nominal typing, strict → More upfront design

## Rust Architecture Design

### Project Structure

```
src/
├── main.rs                 // Entry point, server setup
├── lib.rs                  // Library root (for testing)
├── server/
│   ├── mod.rs             // MCP server implementation
│   ├── transport.rs       // Stdio transport
│   └── handlers.rs        // Tool call handlers
├── client/
│   ├── mod.rs             // Dex API client
│   └── http.rs            // HTTP client wrapper
├── tools/
│   ├── mod.rs             // Tool trait definitions
│   ├── discovery.rs       // Contact discovery tools
│   ├── history.rs         // Relationship history tools
│   └── enrichment.rs      // Contact enrichment tools
├── matching/
│   ├── mod.rs             // Matching utilities
│   └── fuzzy_matcher.rs   // Fuzzy matching implementation
├── search/
│   ├── mod.rs             // Search module
│   └── full_text_index.rs // Full-text search index
├── models/
│   ├── mod.rs             // Data models
│   ├── contact.rs         // Contact structures
│   ├── note.rs            // Note structures
│   └── reminder.rs        // Reminder structures
├── cache/
│   ├── mod.rs             // Cache implementations
│   └── timed_cache.rs     // TTL-based cache
├── config.rs              // Configuration
└── error.rs               // Error types

tests/
├── integration/           // Integration tests
├── fixtures/              // Test data
└── mock_client.rs         // Mock HTTP client
```

### Key Design Decisions (Optimized for Lightweight Performance)

1. **MCP SDK** ✅:
   - **Decision**: Use official `modelcontextprotocol/rust-sdk`
   - Official Rust SDK is available and actively maintained
   - Provides stdio transport out of the box
   - Built on tokio (determines our async runtime choice)

2. **Async Runtime** ✅:
   - **Decision**: Use `tokio` with **minimal features** (required by MCP SDK)
   - Enable only: `macros`, `rt`, `io-std`, `sync`, `time`
   - Avoid `full` feature to reduce binary size and dependencies
   - Single-threaded runtime acceptable for local stdio use
   - **Alternative**: Evaluate if MCP SDK can work with `smol` (unlikely)

3. **HTTP Client** ✅:
   - **Decision**: Use `ureq` (synchronous, lightweight)
   - **Rationale**:
     - Only ~40 total dependencies vs reqwest's ~200
     - No need for async HTTP (API calls are infrequent)
     - Can use from tokio runtime via `tokio::task::spawn_blocking`
     - Simpler API, faster compile times
     - Pure Rust, no unsafe blocks
   - 10-second timeout configured
   - TLS via `rustls` (native-tls adds OpenSSL dependency)

4. **Fuzzy Matching** ✅:
   - **Decision**: Use `nucleo` library
   - **Rationale**:
     - 6-10x faster than `fuzzy-matcher` and `sublime_fuzzy`
     - Battle-tested in Helix editor (large user base)
     - Optimized Unicode segmentation (presegmented, only done once)
     - Excellent sorting performance (5% of match time)
     - Reduced memory usage vs alternatives
   - May need custom scoring adapter to match Fuse.js confidence scale
   - Fallback: Custom scoring function if needed

5. **Caching** ✅:
   - Custom `TimedCache<K, V>` struct with TTL
   - Use `std::time::Instant` for timestamps (no async needed)
   - `Arc<RwLock<HashMap>>` for thread-safe caching
   - No external cache library (keep it lightweight)

6. **Error Handling** ✅:
   - `thiserror` for library errors (precise types)
   - `anyhow` for application errors (convenience)
   - Custom error types: `DexApiError`, `ConfigError`, `MatchingError`

7. **JSON Serialization** ✅:
   - **Decision**: Use `sonic-rs` for performance, fallback to `serde_json`
   - **Rationale**:
     - sonic-rs is 2-3x faster than serde_json (SIMD-based)
     - Direct parsing to Rust structs (no intermediate tape)
     - Works on stable Rust
     - Supports x86_64 and aarch64 (our target platforms)
   - Use `serde_json` only if sonic-rs compatibility issues arise
   - Derive macros for models: `#[serde(rename_all = "snake_case")]`

8. **Logging** ✅:
   - `tracing` for structured logging
   - Minimal log output (errors only to stderr)
   - Careful to avoid PII logging
   - Matches TypeScript approach (suppressed stdout)

## Implementation Plan

### Phase 1: Foundation (Week 1)

**Goal**: Set up project structure and core dependencies

1. **Initialize Rust Project** ✅ (Already done)
   - `cargo init`
   - Update `Cargo.toml` with metadata

2. **Add Core Dependencies (Lightweight Stack)**
   ```toml
   [dependencies]
   # MCP Protocol (official SDK)
   mcp_sdk = { git = "https://github.com/modelcontextprotocol/rust-sdk" }
   # Or if published: mcp_sdk = "0.1"

   # Async runtime (minimal features for MCP SDK)
   tokio = { version = "1", features = ["macros", "rt", "io-std", "sync", "time"] }

   # HTTP client (lightweight, sync)
   ureq = { version = "2", features = ["json", "tls"] }
   # Use rustls for TLS (pure Rust, no OpenSSL dependency)
   rustls = "0.23"

   # JSON serialization (high performance SIMD)
   serde = { version = "1", features = ["derive"] }
   sonic-rs = "0.3"
   # Fallback: serde_json = "1"  # Only if sonic-rs has issues

   # Fuzzy matching (6-10x faster than alternatives)
   nucleo = "0.5"

   # Error handling
   anyhow = "1"
   thiserror = "1"

   # Environment variables
   dotenvy = "0.15"

   # Logging (minimal, stderr only)
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }

   [dev-dependencies]
   # HTTP mocking for tests
   mockito = "1"

   # Test utilities
   tokio-test = "0.4"

   [profile.release]
   # Optimize for size and performance
   opt-level = "z"         # Optimize for size
   lto = true              # Link-time optimization
   codegen-units = 1       # Better optimization (slower compile)
   strip = true            # Strip symbols
   panic = "abort"         # Smaller binary
   ```

   **Dependency Rationale**:
   - **Total deps**: ~60-80 (vs ~250 with reqwest + tokio full)
   - **Binary size**: Target <5MB (vs 8-12MB typical)
   - **Compile time**: ~2-3min first build (vs 4-6min with full stack)
   - **Runtime overhead**: Minimal (single-threaded tokio + sync HTTP)

3. **Research MCP SDK** ✅ (Completed)
   - ✅ Official Rust SDK confirmed: `modelcontextprotocol/rust-sdk`
   - ✅ Uses tokio async runtime
   - ✅ Provides stdio transport (our primary need)
   - Decision: Use official SDK

4. **Define Data Models** (`models/`)
   - Implement `Contact`, `Note`, `Reminder` structs
   - Add serde derives
   - Write unit tests for serialization

5. **Configuration Module** (`config.rs`)
   - Environment variable loading
   - Validation
   - Default values
   - Unit tests

**Deliverables**:
- ✅ Working Cargo project
- Core dependencies added
- Data models with tests
- Configuration module with tests

### Phase 2: HTTP Client (Week 2)

**Goal**: Implement lightweight Dex API client

1. **HTTP Client Wrapper** (`client/http.rs`)
   - `ureq::Agent` setup (synchronous)
   - Header injection (API key via middleware)
   - 10-second timeout configuration
   - TLS via rustls (no OpenSSL dependency)
   - Error handling and retry logic
   - Use `tokio::task::spawn_blocking` when calling from async context

2. **API Client Implementation** (`client/mod.rs`)
   - `DexClient` struct
   - Contact operations (CRUD)
   - Note operations (CRUD)
   - Reminder operations (CRUD)
   - Pagination handling
   - All methods use sync HTTP internally (wrapped for async if needed)

3. **Error Types** (`error.rs`)
   - `DexApiError` enum
   - HTTP error mapping
   - Error message formatting

4. **Unit Tests**
   - Mock HTTP server with `mockito`
   - Test all API methods
   - Test error handling
   - Test pagination

**Deliverables**:
- Complete `DexClient` implementation
- Comprehensive unit tests
- Error handling

### Phase 3: Caching & Matching (Week 3)

**Goal**: Implement caching and fuzzy matching

1. **Timed Cache** (`cache/timed_cache.rs`)
   - Generic `TimedCache<K, V>` struct
   - TTL support
   - Thread-safe with `Arc<RwLock<T>>`
   - Automatic expiration checking
   - Unit tests

2. **String Normalization** (`matching/fuzzy_matcher.rs`)
   - Email normalization (lowercase, trim)
   - Phone normalization (digits only, last 10)
   - URL normalization (extract username/path)
   - Unit tests

3. **Exact Matching** (`matching/fuzzy_matcher.rs`)
   - Email matching
   - Phone matching
   - Social URL matching
   - Confidence scoring (100 for exact)

4. **Fuzzy Name Matching**
   - Integrate `nucleo` library (6-10x faster than alternatives)
   - Implement scoring adapter to convert nucleo scores to Fuse.js-compatible confidence (0-100)
   - Company-based confidence boosting
   - Handle Unicode properly (nucleo handles this efficiently)

5. **ContactMatcher Implementation**
   - `ContactMatcher` struct
   - `find_matches()` method
   - Result ranking and limiting (top 5)
   - Comprehensive unit tests
   - Benchmark against TypeScript version to verify performance gains

**Deliverables**:
- Generic timed cache
- Complete fuzzy matching implementation
- Unit tests with 80%+ coverage

### Phase 4: Full-Text Search (Week 4)

**Goal**: Implement full-text search index

1. **Document Extraction** (`search/full_text_index.rs`)
   - `SearchableDocument` struct
   - Contact field extraction
   - Note extraction (HTML stripping)
   - Reminder extraction

2. **HTML Stripping**
   - Simple regex-based approach
   - Preserve text content
   - Handle common tags

3. **Search Index**
   - `FullTextSearchIndex` struct
   - Document indexing
   - Fuzzy search implementation
   - Match context extraction

4. **Snippet Generation**
   - Context extraction around matches
   - Ellipsis for truncation
   - Match highlighting (markdown bold)

5. **Result Aggregation**
   - Group by contact
   - Multi-match confidence boosting
   - Sorting and limiting

**Deliverables**:
- Full-text search implementation
- Snippet generation
- Unit tests

### Phase 5: Tool Implementations (Week 5)

**Goal**: Implement all MCP tools

1. **Contact Discovery Tools** (`tools/discovery.rs`)
   - `ContactDiscoveryTools` struct
   - `find_contact()` implementation
   - `get_contact_details()` implementation
   - Caching integration
   - Email search fallback

2. **Relationship History Tools** (`tools/history.rs`)
   - `RelationshipHistoryTools` struct
   - `get_contact_history()` - timeline aggregation
   - `get_contact_notes()` - filtered notes
   - `get_contact_reminders()` - filtered reminders
   - Date filtering
   - Sorting

3. **Contact Enrichment Tools** (`tools/enrichment.rs`)
   - `ContactEnrichmentTools` struct
   - `enrich_contact()` - smart merging
   - `add_contact_note()` - note creation
   - `create_contact_reminder()` - reminder creation
   - Array merging logic (tags, social profiles)

4. **Unit Tests**
   - Mock client usage
   - Test all tool methods
   - Test error handling
   - Test edge cases

**Deliverables**:
- All three tool modules
- Comprehensive unit tests
- Integration with client and matchers

### Phase 6: MCP Server (Week 6)

**Goal**: Implement MCP protocol server

1. **MCP Protocol** (`server/mod.rs`)
   - If SDK exists: Use SDK
   - If not: Implement JSON-RPC 2.0
   - Message parsing
   - Response formatting

2. **Stdio Transport** (`server/transport.rs`)
   - Read from stdin
   - Write to stdout
   - Line-by-line JSON parsing
   - Error handling

3. **Tool Registration** (`server/handlers.rs`)
   - Tool schema definitions
   - Match TypeScript schemas exactly
   - 10 tools: discovery (2), history (3), enrichment (3), search (2)

4. **Tool Call Handler**
   - Request routing
   - Argument parsing
   - Tool execution
   - Response formatting
   - Error handling

5. **Server Initialization** (`main.rs`)
   - Config loading
   - Client initialization
   - Tool initialization
   - Server start
   - Graceful shutdown

**Deliverables**:
- Complete MCP server
- All 10 tools working
- Proper error handling

### Phase 7: Integration Testing (Week 7)

**Goal**: End-to-end testing and validation

1. **Integration Tests** (`tests/integration/`)
   - Server initialization
   - Tool call scenarios
   - Cache behavior
   - Error scenarios

2. **Compatibility Testing**
   - Compare with TypeScript implementation
   - Verify matching behavior
   - Verify API compatibility
   - Verify response formats

3. **Performance Testing**
   - Benchmark fuzzy matching
   - Benchmark full-text search
   - Memory usage profiling
   - Compare with TypeScript version

4. **Manual Testing**
   - Test with Claude Desktop
   - Real API testing (with test account)
   - Edge case validation

**Deliverables**:
- Integration test suite
- Performance benchmarks
- Compatibility validation report

### Phase 8: Polish & Documentation (Week 8)

**Goal**: Production readiness

1. **Code Quality**
   - `cargo clippy` - zero warnings
   - `cargo fmt` - consistent formatting
   - Documentation comments (`///`)
   - Examples in docs

2. **Error Messages**
   - User-friendly error messages
   - Context preservation
   - Debugging information

3. **Configuration**
   - Environment variable documentation
   - Example `.env` file
   - Validation messages

4. **Documentation**
   - README.md (installation, usage, configuration)
   - API documentation (`cargo doc`)
   - Architecture documentation
   - Migration guide from TypeScript

5. **Security Review**
   - No PII logging
   - Secure credential handling
   - Dependency audit (`cargo audit`)

6. **Packaging**
   - Binary optimization
   - Release build configuration
   - Distribution planning

**Deliverables**:
- Production-ready codebase
- Complete documentation
- Security validation
- Optimized binary

## Testing Strategy

### Unit Tests

Each module should have unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_normalization() {
        let matcher = ContactMatcher::new();
        assert_eq!(
            matcher.normalize_email(" Test@Example.COM "),
            "test@example.com"
        );
    }
}
```

### Integration Tests

Full workflow tests in `tests/` directory:

```rust
#[tokio::test]
async fn test_contact_discovery_workflow() {
    let config = Config::from_env();
    let client = DexClient::new(config);
    let tools = ContactDiscoveryTools::new(client);

    let matches = tools.find_contact(FindContactParams {
        email: Some("test@example.com".to_string()),
        ..Default::default()
    }).await.unwrap();

    assert!(!matches.is_empty());
}
```

### Mock Testing

Use `mockito` for HTTP mocking:

```rust
#[tokio::test]
async fn test_api_error_handling() {
    let mut server = mockito::Server::new();
    let mock = server.mock("GET", "/contacts")
        .with_status(500)
        .create();

    let client = DexClient::with_base_url(server.url());
    let result = client.get_contacts(100, 0).await;

    assert!(result.is_err());
    mock.assert();
}
```

### Coverage Targets

- Lines: 80%
- Functions: 80%
- Branches: 75%

Use `cargo tarpaulin` or `cargo llvm-cov` for coverage.

## Open Questions & Considerations

### 1. MCP SDK Availability ✅ RESOLVED

**Question**: Is there a mature Rust MCP SDK?

**Resolution**: Yes! Official `modelcontextprotocol/rust-sdk` exists and is actively maintained.

**Details**:
- Built on tokio async runtime
- Provides stdio transport (perfect for local use)
- Follows MCP specification 2025-06-18
- Well-tested with comprehensive test suite

**Decision**: Use official SDK ✅

### 2. Fuzzy Matching Library ✅ RESOLVED

**Question**: Which fuzzy matching library best matches Fuse.js behavior?

**Resolution**: Use `nucleo` library

**Rationale**:
- **Performance**: 6-10x faster than `fuzzy-matcher` and `sublime_fuzzy`
- **Battle-tested**: Used in Helix editor with large user base
- **Memory efficient**: Optimized Unicode segmentation, reduced matrix width
- **Sorting**: Very fast sorting (only 5% of match time)

**Implementation Plan**:
1. Use nucleo for fuzzy matching
2. Implement scoring adapter (nucleo score → Fuse.js confidence 0-100)
3. Benchmark against TypeScript to verify compatibility
4. Adjust scoring if needed to match behavior

### 3. Performance Targets ✅ UPDATED

**Question**: What performance improvements should we target?

**Considerations**:
- Rust should be faster (compiled, no GC)
- Lightweight stack reduces overhead
- nucleo provides 6-10x matching improvement
- sonic-rs provides 2-3x JSON improvement

**Updated Targets** (with lightweight stack):
- **Fuzzy matching**: 6-10x faster (nucleo benchmark)
- **Full-text search**: 3-5x faster (nucleo + optimized iteration)
- **JSON parsing**: 2-3x faster (sonic-rs vs serde_json)
- **Memory**: 20-30% lower (no Node.js overhead, efficient caching)
- **Binary size**: <5MB release build (vs 8-12MB typical Rust + tokio full)
- **Startup time**: <100ms (vs ~300ms Node.js)
- **Dependencies**: ~60-80 crates (vs ~250 with reqwest + tokio full)

**Why these targets are achievable**:
- nucleo benchmarks show 6-10x improvement over fuzzy-matcher
- sonic-rs benchmarks show 2-3x over serde_json
- ureq adds ~40 deps vs reqwest's ~200
- Minimal tokio features vs full

### 4. Backwards Compatibility

**Question**: Must the Rust version be 100% compatible with TypeScript?

**Considerations**:
- API responses should match exactly
- Confidence scores should be similar (not necessarily identical)
- Tool schemas must match exactly

**Approach**:
- Match tool schemas exactly
- Allow minor scoring differences (document them)
- Validate with integration tests

### 5. Cross-Platform Support

**Question**: Which platforms to support?

**Considerations**:
- TypeScript version: Any platform (Node.js)
- Rust: Need to compile per platform

**Targets**:
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64/M1)
- Windows (x86_64)

### 6. Error Handling Strategy

**Question**: How detailed should error messages be?

**Considerations**:
- Debug mode: Very detailed
- Production: User-friendly
- No PII in errors

**Approach**:
- Use `anyhow` with context
- Separate debug info from user messages
- Test error messages

### 7. Configuration Flexibility

**Question**: Support config files or just environment variables?

**Current**: Environment variables only

**Options**:
- A) Keep environment variables only
- B) Add config file support (TOML, JSON)
- C) Support both

**Recommendation**: Start with A (match TypeScript), add B if needed.

### 8. Deployment

**Question**: How will users install and run the Rust version?

**Options**:
- Compiled binary distribution
- Cargo install from git
- Package managers (Homebrew, apt, etc.)

**Recommendation**:
1. Start with binary distribution
2. Add cargo install support
3. Package managers later

## Success Criteria

The Rust port is successful when:

1. ✅ **Functional Parity**
   - All 10 tools implemented
   - Same behavior as TypeScript version
   - Pass compatibility tests

2. ✅ **Performance**
   - Startup time ≤ TypeScript version
   - Response time ≤ TypeScript version
   - Memory usage ≤ 1.5x TypeScript version

3. ✅ **Code Quality**
   - 80%+ test coverage
   - Zero clippy warnings
   - Clean architecture

4. ✅ **Documentation**
   - Complete README
   - API documentation
   - Migration guide

5. ✅ **Production Ready**
   - Security validated
   - Error handling robust
   - Easy deployment

## Risk Assessment

### High Risk

1. **MCP Protocol Implementation**
   - Risk: No mature Rust SDK
   - Mitigation: Implement minimal protocol, focus on stdio transport
   - Impact: 2-3 days extra work

2. **Fuzzy Matching Compatibility**
   - Risk: Different scoring from Fuse.js
   - Mitigation: Custom implementation or score adjustment
   - Impact: 1-2 days extra work

### Medium Risk

1. **Performance Not Meeting Targets**
   - Risk: Rust version slower than expected
   - Mitigation: Profiling and optimization
   - Impact: 2-3 days extra work

2. **API Compatibility Issues**
   - Risk: Dex API quirks not documented
   - Mitigation: Extensive testing with real API
   - Impact: 1-2 days debugging

### Low Risk

1. **Cache Implementation**
   - Risk: TTL behavior differences
   - Mitigation: Well-tested cache module
   - Impact: < 1 day

2. **Async Runtime Issues**
   - Risk: Tokio complexity
   - Mitigation: Use standard patterns
   - Impact: < 1 day

## Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| 1. Foundation | Week 1 | Project structure, models, config |
| 2. HTTP Client | Week 2 | DexClient with tests |
| 3. Caching & Matching | Week 3 | Fuzzy matcher, cache |
| 4. Full-Text Search | Week 4 | Search index |
| 5. Tool Implementations | Week 5 | All tools |
| 6. MCP Server | Week 6 | Complete server |
| 7. Integration Testing | Week 7 | E2E tests, validation |
| 8. Polish & Documentation | Week 8 | Production ready |

**Total Estimated Time**: 8 weeks (assumes part-time work, ~20 hours/week)

**Fast Track**: Could be completed in 4 weeks with full-time effort.

## Lightweight Architecture Benefits

Our optimized crate selection provides significant advantages for a local MCP server:

### Performance Benefits
- **6-10x faster fuzzy matching** (nucleo vs fuzzy-matcher/Fuse.js)
- **2-3x faster JSON parsing** (sonic-rs vs serde_json)
- **<100ms startup time** (minimal runtime, no Node.js overhead)
- **Low memory footprint** (20-30% less than Node.js equivalent)

### Development Benefits
- **Faster compile times**: ~2-3min vs 4-6min (fewer dependencies)
- **Smaller binary**: Target <5MB (vs 8-12MB typical)
- **Simpler async model**: Minimal tokio, sync HTTP via ureq
- **Pure Rust**: No OpenSSL/C dependencies (rustls for TLS)

### Deployment Benefits
- **Single binary**: No runtime dependencies
- **Cross-platform**: Compile for Windows, macOS, Linux
- **Resource efficient**: Ideal for local client use
- **Fast cold starts**: Critical for CLI/MCP use case

### Dependency Comparison

| Aspect | Heavy Stack (reqwest + tokio full) | Lightweight Stack (ureq + minimal tokio) |
|--------|-------------------------------------|------------------------------------------|
| Total dependencies | ~250 crates | ~60-80 crates |
| Compile time (clean) | 4-6 minutes | 2-3 minutes |
| Binary size | 8-12 MB | <5 MB (target) |
| Memory baseline | ~15-20 MB | ~8-12 MB |
| Startup time | ~200-300ms | <100ms |
| TLS dependency | OpenSSL or rustls | rustls (pure Rust) |

## Next Steps

1. ✅ Initialize Rust project (DONE)
2. ✅ Create CLAUDE.md (DONE)
3. ✅ Create this plan (DONE)
4. ✅ Research MCP SDK for Rust (DONE - official SDK exists)
5. ✅ Research lightweight crate options (DONE - plan updated)
6. Begin Phase 1 implementation
7. Set up CI/CD pipeline
8. Create issue tracker for tasks

**Ready to Start**: Phase 1 - Foundation

## References

### Project Resources
- [Dex API Documentation](https://getdex.com/api)
- [MCP Specification](https://modelcontextprotocol.io/)
- [Original TypeScript Implementation](../DexMCPServer)

### Rust MCP
- [Official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [MCP Protocol SDK Docs](https://docs.rs/mcp-protocol-sdk)
- [Building MCP Servers in Rust](https://www.shuttle.dev/blog/2025/07/18/how-to-build-a-stdio-mcp-server-in-rust)

### Core Crate Documentation
- [Tokio Documentation](https://tokio.rs/)
- [ureq HTTP Client](https://docs.rs/ureq/)
- [nucleo Fuzzy Matcher](https://docs.rs/nucleo/)
- [sonic-rs JSON](https://docs.rs/sonic-rs/)
- [serde Documentation](https://serde.rs/)

### Performance Resources
- [Rust Async Runtimes Comparison](https://corrode.dev/blog/async/)
- [Rust HTTP Client Comparison](https://blog.logrocket.com/best-rust-http-client/)
- [JSON Parser Benchmarks](https://github.com/AnnikaCodes/rust-json-parsing-benchmarks)

### Best Practices
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)

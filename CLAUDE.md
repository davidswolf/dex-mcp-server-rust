# DexMCPServerRust

## Project Overview

This is a Rust port of an MCP (Model Context Protocol) server for Dex Personal CRM. The goal is to create a production-quality implementation that provides efficient and reliable integration with the Dex Personal CRM system.

## About MCP (Model Context Protocol)

MCP is a protocol that enables AI assistants to interact with external systems and tools. This server implementation provides Claude (and other AI assistants) with the ability to interact with Dex Personal CRM functionality.

## Rust Best Practices & Standards

### Code Quality

- **Use `clippy`**: Run `cargo clippy` regularly and address all warnings
  - Consider using `cargo clippy -- -D warnings` to treat warnings as errors in CI
- **Format code**: Always run `cargo fmt` before committing
- **Error handling**: Use `Result<T, E>` and avoid `unwrap()` in production code
  - Prefer `?` operator for error propagation
  - Use `.expect()` only when you can guarantee safety with a clear message
  - Consider using error handling crates like `anyhow` or `thiserror`

### Project Structure

```
src/
├── main.rs           # Application entry point
├── lib.rs            # Library root (if applicable)
├── server/           # MCP server implementation
├── handlers/         # Request handlers
├── models/           # Data models
├── client/           # Dex CRM client
├── error.rs          # Error types
└── config.rs         # Configuration management
```

### Testing

- Write unit tests in the same file as the code using `#[cfg(test)]`
- Write integration tests in `tests/` directory
- Aim for meaningful test coverage, especially for core business logic
- Run tests with `cargo test`
- Run tests with output: `cargo test -- --nocapture`

### Dependencies

- Keep dependencies minimal and well-vetted
- Use specific versions in `Cargo.toml` for reproducibility
- Regularly update dependencies: `cargo update`
- Check for outdated dependencies: `cargo outdated` (requires cargo-outdated)

### Common Rust Patterns

1. **Ownership & Borrowing**
   - Prefer borrowing (`&T`, `&mut T`) over ownership when possible
   - Use `Clone` sparingly; prefer references
   - Use `Arc<T>` for shared ownership in concurrent contexts
   - Use `Rc<T>` for shared ownership in single-threaded contexts

2. **Error Handling**
   - Define custom error types using `thiserror` crate
   - Use `anyhow` for application-level errors
   - Implement `std::error::Error` for custom error types

3. **Async/Await**
   - Use `tokio` or `async-std` for async runtime
   - Mark async functions with `async fn`
   - Use `.await` for futures
   - Be mindful of blocking operations in async context

4. **Type Safety**
   - Use newtype pattern for domain-specific types
   - Leverage the type system to prevent invalid states
   - Use enums for state machines and variants

### Documentation

- Document public APIs with `///` doc comments
- Include examples in doc comments where appropriate
- Generate docs: `cargo doc --open`
- Write a clear README.md with usage instructions

### Performance

- Profile before optimizing: `cargo build --release && cargo flamegraph`
- Use `cargo bench` for benchmarking
- Consider using `#[inline]` for small, frequently-called functions
- Use iterators instead of loops where appropriate for better performance

## Development Workflow

### Common Commands

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run the project
cargo run

# Run with release optimizations
cargo run --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run clippy for lints
cargo clippy

# Format code
cargo fmt

# Check code without building
cargo check

# Update dependencies
cargo update
```

### Pre-commit Checklist

- [ ] `cargo fmt` - Code is formatted
- [ ] `cargo clippy` - No clippy warnings
- [ ] `cargo test` - All tests pass
- [ ] `cargo build` - Project builds successfully
- [ ] Documentation updated if needed

## MCP Server Specific Notes

### Protocol Implementation

- Follow MCP specification for request/response handling
- Implement proper JSON-RPC message handling
- Handle method calls, notifications, and errors according to spec
- Support standard MCP capabilities (tools, resources, prompts)

### Server Capabilities

Document which MCP capabilities this server implements:
- **Tools**: Exposed tools/functions
- **Resources**: Available resources
- **Prompts**: Pre-defined prompts
- **Notifications**: Event notifications

### Connection Handling

- Support stdio transport (standard for MCP servers)
- Implement proper connection lifecycle management
- Handle graceful shutdown

## Security Considerations

- Validate all input from the MCP client
- Sanitize data before passing to Dex CRM
- Handle authentication/authorization if required
- Be mindful of rate limiting and resource exhaustion
- Never log sensitive information (tokens, passwords, PII)

## Configuration

- Use environment variables for configuration where appropriate
- Consider using `config` or `figment` crates for configuration management
- Document all configuration options
- Provide sensible defaults

## Logging

- Use `tracing` or `log` crate for structured logging
- Include appropriate log levels (error, warn, info, debug, trace)
- Avoid excessive logging in hot paths
- Consider log rotation for production deployments

## Useful Crates to Consider

- **serde**: Serialization/deserialization (essential for JSON)
- **tokio**: Async runtime
- **anyhow**: Error handling for applications
- **thiserror**: Error handling for libraries
- **tracing**: Structured logging
- **clap**: Command-line argument parsing
- **reqwest**: HTTP client (if calling Dex API)
- **sqlx**: Database access (if needed)

## Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [MCP Specification](https://modelcontextprotocol.io/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)

## Notes for AI Assistant

When working on this project:
- Follow Rust idioms and conventions
- Write safe, efficient code
- Add tests for new functionality
- Update documentation as needed
- Consider error cases and edge conditions
- Use type system to prevent bugs at compile time
- Profile performance-critical sections
- Keep dependencies minimal and justified

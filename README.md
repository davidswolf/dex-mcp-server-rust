# Dex MCP Server (Rust)

A production-quality Rust implementation of a Model Context Protocol (MCP) server for [Dex Personal CRM](https://getdex.com).

## Overview

This MCP server enables AI assistants (like Claude) to interact with Dex Personal CRM, providing capabilities for:

- **Contact Discovery**: Intelligent contact search with fuzzy matching and confidence scoring
- **Contact Enrichment**: Add notes, reminders, and update contact information
- **Relationship History**: Retrieve contact timeline and interaction history
- **Full-Text Search**: Fast search across contacts, notes, and reminders with caching

## Features

- 🦀 **Written in Rust**: Fast, safe, and reliable
- 🏗️ **Clean Architecture**: Repository pattern with clear separation of concerns
- 🔍 **Advanced Search**: Full-text search with BM25 ranking and fuzzy matching
- ⚡ **Performance Optimized**: Intelligent caching with configurable TTL
- 🧪 **Well-Tested**: Comprehensive unit and integration tests
- 📊 **Observability**: Built-in logging and metrics
- 🔒 **Secure**: Environment-based configuration for API keys

## Quick Start

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Dex API key (get one from [Dex Settings](https://app.getdex.com/settings))

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/DexMCPServerRust.git
cd DexMCPServerRust
```

2. Set up environment variables:
```bash
cp .env.example .env
# Edit .env and add your Dex API key
```

3. Build the project:
```bash
cargo build --release
```

4. Run the server:
```bash
cargo run --release
```

## Configuration

Configuration is managed through environment variables. Create a `.env` file based on `.env.example`:

```env
DEX_API_KEY=your_api_key_here
DEX_API_BASE_URL=https://api.getdex.com/api/rest
MAX_MATCH_RESULTS=10
MATCH_CONFIDENCE_THRESHOLD=70
CACHE_TTL_MINUTES=5
```

### Claude Desktop Integration

To use this MCP server with Claude Desktop, add it to your configuration:

**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "dex": {
      "command": "C:\\path\\to\\DexMCPServerRust\\target\\release\\dex-mcp-server.exe",
      "env": {
        "DEX_API_KEY": "your_api_key_here"
      }
    }
  }
}
```

See `claude_desktop_config_example.json` for a complete example.

## Available MCP Tools

### Contact Discovery

- **find_contact**: Search for contacts by name, email, phone, or company
- **get_contact_details**: Retrieve complete contact information

### Contact Enrichment

- **enrich_contact**: Add or update contact information
- **add_contact_note**: Create a note for a contact
- **create_contact_reminder**: Set a reminder for a contact

### Relationship History

- **get_contact_history**: Retrieve contact timeline with notes and reminders
- **get_contact_notes**: Get all notes for a contact
- **get_contact_reminders**: Get all reminders for a contact

### Search

- **search_full_text**: Fast full-text search across all data

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run library tests only
cargo test --lib

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run linter with warnings as errors
cargo clippy -- -D warnings
```

### Benchmarks

```bash
# Run benchmarks
cargo bench
```

## CI/CD

This project uses GitHub Actions for continuous integration and automated releases.

### Workflows

- **CI Pipeline**: Runs on every push and PR
  - Code formatting checks (`cargo fmt`)
  - Linting with Clippy
  - Tests on Linux, macOS, and Windows
  - Documentation checks
  - Security audits
  - Dependency checks

- **Release Pipeline**: Automated semantic versioning
  - Automatic version calculation from commit messages
  - Multi-platform binary builds (Linux, macOS, Windows, ARM)
  - GitHub releases with changelogs
  - Optional publishing to crates.io

### Commit Message Format

This project uses [Conventional Commits](https://www.conventionalcommits.org/) for automatic semantic versioning:

- `feat:` - New feature (minor version bump)
- `fix:` - Bug fix (patch version bump)
- `BREAKING CHANGE:` - Breaking change (major version bump)
- `docs:`, `chore:`, `refactor:`, `test:`, `ci:` - No version bump

Example:
```bash
git commit -m "feat: add contact export functionality"
```

For more details, see [.github/WORKFLOWS.md](.github/WORKFLOWS.md).

## Architecture

The project follows clean architecture principles:

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root
├── client/              # Dex API client (sync wrapper)
├── repositories/        # Data access layer
├── services/            # Business logic layer
├── tools/               # MCP tool implementations
├── models/              # Domain models
├── search/              # Full-text search engine
├── cache/               # Caching utilities
├── config.rs            # Configuration management
├── error.rs             # Error types
└── server/              # MCP server implementation
```

For more details, see [CLAUDE.md](CLAUDE.md).

## Performance

The server includes several performance optimizations:

- **Request Caching**: Configurable TTL-based caching for API responses
- **Parallel Fetching**: Concurrent API requests where possible
- **Efficient Indexing**: BM25-based search index with lazy initialization
- **Connection Pooling**: Reusable HTTP connections

See [PERFORMANCE_RESULTS.md](PERFORMANCE_RESULTS.md) for benchmarks.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Pre-commit Checklist

- [ ] `cargo fmt` - Code is formatted
- [ ] `cargo clippy` - No clippy warnings
- [ ] `cargo test` - All tests pass
- [ ] `cargo build` - Project builds successfully
- [ ] Documentation updated if needed

## Documentation

- [CLAUDE.md](CLAUDE.md) - Project guidelines and Rust best practices
- [architecture-review.md](architecture-review.md) - Architecture overview
- [LOGGING_GUIDE.md](LOGGING_GUIDE.md) - Logging and debugging guide
- [E2E_TESTS_SUMMARY.md](E2E_TESTS_SUMMARY.md) - End-to-end testing guide

## License

MIT License - see [LICENSE](LICENSE) for details

## Resources

- [MCP Specification](https://modelcontextprotocol.io/)
- [Dex API Documentation](https://docs.getdex.com/)
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## Acknowledgments

Built with the [MCP SDK for Rust](https://github.com/modelcontextprotocol/rust-sdk) and powered by [Dex Personal CRM](https://getdex.com).

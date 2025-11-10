# GitHub Actions Workflows

This document describes the CI/CD workflows configured for this project.

## Overview

This project uses GitHub Actions for continuous integration and automated releases with semantic versioning. The workflows are designed following Rust best practices and support multi-platform builds.

## Workflows

### 1. CI Workflow (`ci.yml`)

Runs on every push and pull request to ensure code quality.

**Triggers:**
- Push to `main`, `master`, or `develop` branches
- Pull requests to `main`, `master`, or `develop` branches

**Jobs:**

1. **Format Check** - Ensures code is formatted with `rustfmt`
2. **Clippy** - Runs Rust linter to catch common mistakes and anti-patterns
3. **Test** - Builds and tests on multiple platforms:
   - Ubuntu (Linux)
   - macOS
   - Windows
   - Rust stable and beta versions
4. **Documentation** - Checks that documentation builds without warnings
5. **Security Audit** - Runs `cargo audit` to check for security vulnerabilities
6. **Dependency Check** - Checks for outdated dependencies

**Caching:**
- Cargo registry, git index, and build artifacts are cached to speed up builds

### 2. Release Workflow (`release.yml`)

Automatically creates releases with semantic versioning based on conventional commits.

**Triggers:**
- Push to `main` or `master` branch (excluding markdown and license files)
- Manual workflow dispatch with optional version bump override

**Jobs:**

1. **Version Determination**
   - Analyzes commit messages using conventional commits format
   - Calculates the next semantic version
   - Generates changelog

2. **Build Release Binaries**
   - Builds optimized release binaries for multiple platforms:
     - Linux x86_64 (glibc)
     - Linux x86_64 (musl - static)
     - Linux ARM64
     - macOS x86_64 (Intel)
     - macOS ARM64 (Apple Silicon)
     - Windows x86_64
   - Strips binaries for smaller size
   - Creates compressed archives (tar.gz for Unix, zip for Windows)

3. **Create Release**
   - Updates version in `Cargo.toml`
   - Commits version bump
   - Creates GitHub release with changelog
   - Uploads all platform binaries
   - Creates and pushes git tag

4. **Publish to crates.io** (Optional)
   - Publishes package to crates.io if enabled
   - Requires `CARGO_REGISTRY_TOKEN` secret

## Semantic Versioning

This project uses [Semantic Versioning](https://semver.org/) with automatic version calculation based on [Conventional Commits](https://www.conventionalcommits.org/).

### Conventional Commit Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Commit Types and Version Bumps

| Commit Type | Version Bump | Example |
|-------------|--------------|---------|
| `feat:` | Minor (0.x.0) | `feat: add fuzzy search for contacts` |
| `fix:` | Patch (0.0.x) | `fix: correct date parsing in reminders` |
| `BREAKING CHANGE:` | Major (x.0.0) | `feat!: redesign API endpoint structure` |
| `docs:` | None | `docs: update README with examples` |
| `chore:` | None | `chore: update dependencies` |
| `refactor:` | None | `refactor: simplify contact search logic` |
| `perf:` | None | `perf: optimize database queries` |
| `test:` | None | `test: add integration tests` |
| `ci:` | None | `ci: update workflow configuration` |

### Examples

**Minor Version Bump (New Feature):**
```bash
git commit -m "feat: add contact export functionality"
```

**Patch Version Bump (Bug Fix):**
```bash
git commit -m "fix: handle null values in contact fields"
```

**Major Version Bump (Breaking Change):**
```bash
git commit -m "feat!: redesign contact search API

BREAKING CHANGE: The contact search endpoint now returns a different response format."
```

**Multiple Changes:**
```bash
git commit -m "feat: add reminder notifications

- Implement notification system
- Add configuration for notification preferences
- Update documentation"
```

## Setting Up Secrets

### Required Secrets

None required for basic CI/CD functionality.

### Optional Secrets

For publishing to crates.io:

1. **CARGO_REGISTRY_TOKEN**
   - Get token from https://crates.io/me
   - Add to repository secrets: Settings → Secrets and variables → Actions → New repository secret
   - Set repository variable `PUBLISH_TO_CRATES_IO=true` to enable publishing

## Manual Release

You can manually trigger a release:

1. Go to Actions tab in GitHub
2. Select "Release" workflow
3. Click "Run workflow"
4. Choose version bump type:
   - `auto` - Calculate based on commits (default)
   - `major` - Force major version bump
   - `minor` - Force minor version bump
   - `patch` - Force patch version bump

## Local Development

Before pushing, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Build documentation
cargo doc --no-deps --all-features

# Check for security issues
cargo install cargo-audit
cargo audit

# Check for outdated dependencies
cargo install cargo-outdated
cargo outdated
```

## Troubleshooting

### CI Failing on Format Check
Run `cargo fmt` locally and commit the changes.

### Clippy Warnings
Run `cargo clippy` locally to see warnings, then fix them.

### Tests Failing on Specific Platform
Check the test output in the Actions tab to see platform-specific issues.

### Release Not Created
- Ensure your commit messages follow conventional commits format
- Check that there are commits with `feat:` or `fix:` since the last release
- Review the "Determine Version" job output to see why no version was calculated

### Binary Build Failing
- Check the build logs for the specific platform
- Ensure all dependencies support the target platform
- Check for platform-specific compilation issues

## Best Practices

1. **Write meaningful commit messages** - Follow conventional commits format
2. **Keep commits focused** - One logical change per commit
3. **Test locally** - Run tests before pushing
4. **Review CI results** - Check workflow results and fix issues promptly
5. **Update CHANGELOG** - Keep track of notable changes
6. **Tag releases** - Use semantic versioning tags
7. **Document breaking changes** - Always document breaking changes in commit messages

## Resources

- [Semantic Versioning](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI/CD Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)

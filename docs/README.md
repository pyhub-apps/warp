# Warp Documentation

This directory contains additional documentation for the Warp Korean Legal Information CLI.

## Documentation Structure

- **[API Documentation](https://pyhub-apps.github.io/warp/)** - Auto-generated rustdoc documentation
- **Getting Started Guide** - Quick start guide for new users
- **API Reference** - Comprehensive API reference with examples
- **Configuration Guide** - Configuration options and best practices
- **Troubleshooting** - Common issues and solutions

## Building Documentation Locally

### Prerequisites

```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install additional tools (optional)
cargo install mdbook        # For additional guides
cargo install cargo-doc     # Enhanced documentation generation
```

### Generate API Documentation

```bash
# Generate documentation for the library
cargo doc --no-deps --open

# Generate documentation with private items
cargo doc --document-private-items --open

# Generate documentation with all features
cargo doc --all-features --open
```

### Build Documentation Website

```bash
# Build the complete documentation site
cargo doc --no-deps --document-private-items --all-features

# The generated documentation will be in target/doc/
# Open target/doc/warp/index.html in your browser
```

## Documentation Guidelines

### Rustdoc Comments

Use the following patterns for rustdoc comments:

```rust
/// Brief description of the item
///
/// Longer description with more details about the functionality,
/// use cases, and important considerations.
///
/// # Arguments
///
/// * `param1` - Description of parameter 1
/// * `param2` - Description of parameter 2
///
/// # Returns
///
/// Description of what the function returns
///
/// # Errors
///
/// * `ErrorType1` - When this error occurs
/// * `ErrorType2` - When this other error occurs
///
/// # Examples
///
/// ```
/// use warp::api::ApiClient;
///
/// let client = ApiClient::new();
/// let result = client.search("query").await?;
/// ```
///
/// # Panics
///
/// This function panics if...
///
/// # Safety
///
/// (Only for unsafe functions)
/// This function is safe if...
```

### Korean Text Handling

When documenting Korean legal concepts:

```rust
/// **National Law Information Center** (국가법령정보센터)
///
/// The NLIC is Korea's primary legal database containing:
/// - **Laws** (법률): National legislation
/// - **Presidential Decrees** (대통령령): Executive orders
/// - **Ministerial Ordinances** (부령): Ministry regulations
```

### Examples

Always include working examples:

```rust
/// # Examples
///
/// ```no_run
/// use warp::api::{ApiClientFactory, ApiType};
/// use warp::config::Config;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::load()?;
/// let client = ApiClientFactory::create(ApiType::Nlic, config.to_client_config())?;
/// # Ok(())
/// # }
/// ```
```

## Documentation Quality Checklist

- [ ] All public APIs documented
- [ ] Examples provided for complex functions
- [ ] Korean legal terms explained
- [ ] Error conditions documented
- [ ] Performance considerations noted
- [ ] Thread safety documented where applicable
- [ ] Examples compile and run
- [ ] Links to related functions provided

## Documentation Testing

### Doc Tests

Run documentation tests to ensure examples work:

```bash
# Run all documentation tests
cargo test --doc

# Run doc tests for a specific module
cargo test --doc api

# Run doc tests with all features
cargo test --doc --all-features
```

### Link Validation

Validate internal documentation links:

```bash
# Generate documentation and check for warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Use external tools for link checking
# (Install with: cargo install cargo-deadlinks)
cargo deadlinks
```

## Contributing to Documentation

### Adding New Documentation

1. Add rustdoc comments to new public APIs
2. Include practical examples
3. Document error conditions
4. Add performance notes where relevant
5. Test examples with `cargo test --doc`

### Updating Existing Documentation

1. Keep examples current with API changes
2. Update performance benchmarks
3. Add new use cases and examples
4. Improve clarity and completeness

### Documentation Review Process

1. Ensure all public APIs are documented
2. Verify examples compile and run
3. Check Korean text rendering
4. Validate internal links
5. Test accessibility on different devices

## Deployment

Documentation is automatically built and deployed via GitHub Actions:

- **Trigger**: Push to `main` branch or pull request
- **Build**: `cargo doc --no-deps --document-private-items --all-features`
- **Deploy**: GitHub Pages at https://pyhub-apps.github.io/warp/
- **Validation**: Link checking and doc test execution

## Style Guide

### Writing Style

- **Clear and Concise**: Use simple, direct language
- **Consistent Terminology**: Use the same terms throughout
- **User-Focused**: Write from the user's perspective
- **Action-Oriented**: Use active voice and imperative mood

### Code Examples

- **Complete**: Examples should compile without additional imports
- **Realistic**: Use realistic data and scenarios
- **Error Handling**: Show proper error handling patterns
- **Performance**: Include performance considerations when relevant

### Korean Content

- **Bilingual**: Provide both Korean and English terms
- **Context**: Explain Korean legal concepts for international users
- **Accuracy**: Ensure Korean translations are accurate
- **Formatting**: Use proper Korean typography and formatting

## Resources

- [Rust Documentation Guidelines](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
- [Korean Legal System Overview](https://www.law.go.kr/LSW/eng/engMain.do)
- [API Documentation Best Practices](https://swagger.io/resources/articles/best-practices-in-api-documentation/)
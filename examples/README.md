# Korean Legal API Examples

This directory contains comprehensive examples demonstrating how to use the Warp library to access Korean government legal databases.

## Overview

The examples show various aspects of using the Korean legal API clients:

- **Basic Searches**: Simple queries across different legal databases
- **Advanced Filtering**: Using search parameters and filters
- **Multi-API Integration**: Searching across multiple databases
- **Document Retrieval**: Getting detailed legal document information
- **Performance Optimization**: Caching and performance best practices

## Available Examples

### 1. API Usage Examples (`api_usage.rs`)

Comprehensive example covering:
- Basic NLIC (National Law Information Center) searches
- Advanced search with filters and sorting
- Multi-API searches across different databases
- Detailed document retrieval with revision history
- Performance optimization with caching

```bash
# Run the API usage examples
cargo run --example api_usage
```

### 2. Cache Usage Examples (`cache_usage.rs`)

Advanced caching functionality:
- Tiered caching strategies
- Cache key generation
- Cache invalidation patterns
- Performance metrics

```bash
# Run the cache examples
cargo run --example cache_usage
```

## Prerequisites

### 1. Configuration

Create a configuration file with your API keys:

```yaml
# ~/.config/warp/config.yaml
api:
  nlic:
    api_key: "your-nlic-api-key"
    base_url: "https://www.law.go.kr/DRF"
  elis:
    api_key: "your-elis-api-key"
    base_url: "https://www.elis.go.kr/api"

cache:
  enabled: true
  directory: "~/.cache/warp"
  ttl_hours: 24

network:
  timeout_seconds: 30
  max_retries: 3
  retry_delay_ms: 100
```

### 2. API Keys

Obtain API keys from the respective Korean government agencies:

- **NLIC**: National Law Information Center (법제처)
- **ELIS**: Local Regulations Information System
- **PREC**: Court Precedents Database
- **ADMRUL**: Administrative Rules
- **EXPC**: Legal Interpretation Cases

## Running Examples

### Basic Setup

```bash
# Clone the repository
git clone https://github.com/pyhub-apps/warp.git
cd warp

# Set up configuration
mkdir -p ~/.config/warp
cp examples/config.example.yaml ~/.config/warp/config.yaml
# Edit the config file with your API keys

# Run examples
cargo run --example api_usage
```

### Environment Variables

Alternatively, use environment variables:

```bash
export WARP_NLIC_API_KEY="your-nlic-key"
export WARP_ELIS_API_KEY="your-elis-key"
export WARP_CACHE_DIR="./cache"

cargo run --example api_usage
```

## Example Code Patterns

### Basic Search Pattern

```rust
use warp::api::{ApiClientFactory, ApiType};
use warp::api::types::UnifiedSearchRequest;
use warp::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let client = ApiClientFactory::create(ApiType::Nlic, config.to_client_config())?;

    let request = UnifiedSearchRequest {
        query: "민법".to_string(),
        page: Some(1),
        size: Some(10),
        ..Default::default()
    };

    let response = client.search(request).await?;
    println!("Found {} results", response.total_count.unwrap_or(0));

    Ok(())
}
```

### Multi-API Search Pattern

```rust
use warp::api::{ApiClientFactory, ApiType};

async fn search_all_apis(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let apis = vec![ApiType::Nlic, ApiType::Elis, ApiType::Prec];

    for api_type in apis {
        let client = ApiClientFactory::create(api_type, config.to_client_config())?;
        let request = UnifiedSearchRequest {
            query: query.to_string(),
            ..Default::default()
        };

        match client.search(request).await {
            Ok(response) => {
                println!("{:?}: {} results", api_type, response.total_count.unwrap_or(0));
            }
            Err(e) => eprintln!("Error searching {:?}: {}", api_type, e),
        }
    }

    Ok(())
}
```

### Caching Pattern

```rust
use warp::api::client::ClientConfig;
use warp::cache::CacheStore;
use std::sync::Arc;

async fn create_cached_client() -> Result<Box<dyn LegalApiClient>, Box<dyn std::error::Error>> {
    let cache = Arc::new(CacheStore::new("./cache")?);

    let config = ClientConfig {
        api_key: "your-key".to_string(),
        cache: Some(cache),
        bypass_cache: false,
        ..Default::default()
    };

    let client = ApiClientFactory::create(ApiType::Nlic, config)?;
    Ok(client)
}
```

## Performance Tips

### 1. Enable Caching

Always enable caching for production use:

```rust
let cache = Arc::new(CacheStore::new("./cache")?);
let config = ClientConfig {
    cache: Some(cache),
    bypass_cache: false,
    ..Default::default()
};
```

### 2. Configure Timeouts

Set appropriate timeouts based on your use case:

```rust
let config = ClientConfig {
    timeout: 60,        // 60 seconds for slow connections
    max_retries: 5,     // Retry up to 5 times
    retry_base_delay: 100, // Start with 100ms delay
    ..Default::default()
};
```

### 3. Batch Operations

When possible, batch multiple operations:

```rust
// Instead of multiple individual searches
let results = futures::future::join_all(
    queries.into_iter().map(|query| client.search(query))
).await;
```

## Error Handling

### Common Error Patterns

```rust
match client.search(request).await {
    Ok(response) => {
        // Handle successful response
        println!("Success: {} results", response.total_count.unwrap_or(0));
    }
    Err(WarpError::NetworkError(e)) => {
        eprintln!("Network error: {}", e);
        // Implement retry logic or fallback
    }
    Err(WarpError::ApiError(code, message)) => {
        eprintln!("API error {}: {}", code, message);
        // Handle specific API errors
    }
    Err(WarpError::RateLimitExceeded) => {
        eprintln!("Rate limit exceeded, waiting...");
        tokio::time::sleep(Duration::from_secs(60)).await;
        // Retry after delay
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
        return Err(e.into());
    }
}
```

## Testing

Run the examples with test mode to use mock data:

```bash
cargo run --example api_usage --features test-mode
```

## Contributing

When adding new examples:

1. Follow the existing code structure
2. Include comprehensive error handling
3. Add documentation comments
4. Test with different API configurations
5. Update this README with new examples

## Support

- **Documentation**: Run `cargo doc --open` for API documentation
- **Issues**: Report issues at [GitHub Issues](https://github.com/pyhub-apps/warp/issues)
- **Discussions**: Join discussions at [GitHub Discussions](https://github.com/pyhub-apps/warp/discussions)

## License

These examples are provided under the same license as the main Warp project.
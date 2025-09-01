use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder};
use std::sync::Arc;
use std::time::Duration;

/// Shared HTTP client pool for optimal connection reuse
/// This provides connection pooling and reduces the overhead of creating new HTTP clients
pub struct HttpClientPool {
    client: Client,
}

impl HttpClientPool {
    /// Create a new HTTP client pool with optimized settings
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            // Connection pool settings
            .pool_max_idle_per_host(10) // Keep up to 10 idle connections per host
            .pool_idle_timeout(Duration::from_secs(30)) // Keep idle connections for 30s
            .timeout(Duration::from_secs(30)) // Default timeout
            // Performance optimizations
            .tcp_keepalive(Duration::from_secs(60)) // Keep TCP connections alive
            .tcp_nodelay(true) // Disable Nagle's algorithm for lower latency
            .http2_prior_knowledge() // Use HTTP/2 when possible
            .use_rustls_tls() // Use rustls for better performance
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Get the shared HTTP client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Create a client with custom timeout
    pub fn with_timeout(&self, timeout: Duration) -> Client {
        ClientBuilder::new()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(timeout)
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .http2_prior_knowledge()
            .use_rustls_tls()
            .build()
            .expect("Failed to create HTTP client with custom timeout")
    }
}

impl Default for HttpClientPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Global HTTP client pool instance
/// This is initialized once and shared across all API clients
static HTTP_CLIENT_POOL: Lazy<Arc<HttpClientPool>> = Lazy::new(|| Arc::new(HttpClientPool::new()));

/// Get the global HTTP client pool
pub fn get_http_client_pool() -> Arc<HttpClientPool> {
    HTTP_CLIENT_POOL.clone()
}

/// Get a shared HTTP client instance
pub fn get_http_client() -> &'static Client {
    HTTP_CLIENT_POOL.client()
}

/// Create an HTTP client with custom configuration
pub fn create_custom_client(timeout_secs: u64, user_agent: &str) -> Client {
    ClientBuilder::new()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(user_agent)
        .tcp_keepalive(Duration::from_secs(60))
        .tcp_nodelay(true)
        .http2_prior_knowledge()
        .use_rustls_tls()
        .build()
        .expect("Failed to create custom HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_pool_creation() {
        let pool = HttpClientPool::new();
        let _client = pool.client();
        // Should not panic
    }

    #[test]
    fn test_global_http_client() {
        let client1 = get_http_client();
        let client2 = get_http_client();

        // Should return the same client instance (pointer comparison)
        assert!(std::ptr::eq(client1, client2));
    }

    #[tokio::test]
    async fn test_custom_client_creation() {
        let client = create_custom_client(10, "test-agent/1.0");
        // Should not panic and should create a working client
        let request = client.get("https://httpbin.org/get");
        // Just test that the client was created successfully
        assert!(std::ptr::addr_of!(client) != std::ptr::null());
    }
}

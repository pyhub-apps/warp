use crate::api::ApiType;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Parameters for PREC cache key generation
pub struct PrecKeyParams<'a> {
    pub endpoint: &'a str,
    pub query: Option<&'a str>,
    pub court: Option<&'a str>,
    pub case_type: Option<&'a str>,
    pub date_from: Option<&'a str>,
    pub date_to: Option<&'a str>,
    pub page: Option<u32>,
    pub size: Option<u32>,
}

/// Cache key generator for different API types
pub struct CacheKeyGenerator;

impl CacheKeyGenerator {
    /// Generate a cache key for API requests
    ///
    /// The key is generated using SHA256 hash of:
    /// - API type
    /// - Request URL or endpoint
    /// - Request parameters (sorted by key for consistency)
    /// - API version (if applicable)
    pub fn generate_key(
        api_type: ApiType,
        endpoint: &str,
        params: &HashMap<String, String>,
        version: Option<&str>,
    ) -> String {
        let mut hasher = Sha256::new();

        // Include API type
        hasher.update(api_type.as_str().as_bytes());
        hasher.update(b"|");

        // Include endpoint
        hasher.update(endpoint.as_bytes());
        hasher.update(b"|");

        // Include version if provided
        if let Some(v) = version {
            hasher.update(v.as_bytes());
        }
        hasher.update(b"|");

        // Include parameters sorted by key for consistency
        let mut sorted_params: Vec<(&String, &String)> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        for (key, value) in sorted_params {
            hasher.update(key.as_bytes());
            hasher.update(b"=");
            hasher.update(value.as_bytes());
            hasher.update(b"&");
        }

        let result = hasher.finalize();
        format!("{}:{:x}", api_type.as_str(), result)
    }

    /// Generate a simple key for basic string-based caching
    pub fn generate_simple_key(api_type: ApiType, query: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(api_type.as_str().as_bytes());
        hasher.update(b"|");
        hasher.update(query.as_bytes());

        let result = hasher.finalize();
        format!("{}:{:x}", api_type.as_str(), result)
    }

    /// Generate key for NLIC (National Law Information Center) API
    pub fn nlic_key(
        endpoint: &str,
        query: Option<&str>,
        law_type: Option<&str>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> String {
        let mut params = HashMap::new();

        if let Some(q) = query {
            params.insert("query".to_string(), q.to_string());
        }
        if let Some(lt) = law_type {
            params.insert("law_type".to_string(), lt.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }

        Self::generate_key(ApiType::Nlic, endpoint, &params, None)
    }

    /// Generate key for ELIS (Easy Law Information Service) API
    pub fn elis_key(
        endpoint: &str,
        query: Option<&str>,
        region: Option<&str>,
        category: Option<&str>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> String {
        let mut params = HashMap::new();

        if let Some(q) = query {
            params.insert("query".to_string(), q.to_string());
        }
        if let Some(r) = region {
            params.insert("region".to_string(), r.to_string());
        }
        if let Some(c) = category {
            params.insert("category".to_string(), c.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }

        Self::generate_key(ApiType::Elis, endpoint, &params, None)
    }

    /// Generate key for PREC (Precedent) API
    pub fn prec_key(params: PrecKeyParams) -> String {
        let mut param_map = HashMap::new();

        if let Some(q) = params.query {
            param_map.insert("query".to_string(), q.to_string());
        }
        if let Some(c) = params.court {
            param_map.insert("court".to_string(), c.to_string());
        }
        if let Some(ct) = params.case_type {
            param_map.insert("case_type".to_string(), ct.to_string());
        }
        if let Some(df) = params.date_from {
            param_map.insert("date_from".to_string(), df.to_string());
        }
        if let Some(dt) = params.date_to {
            param_map.insert("date_to".to_string(), dt.to_string());
        }
        if let Some(p) = params.page {
            param_map.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = params.size {
            param_map.insert("size".to_string(), s.to_string());
        }

        Self::generate_key(ApiType::Prec, params.endpoint, &param_map, None)
    }

    /// Generate key for ADMRUL (Administrative Rule) API
    pub fn admrul_key(
        endpoint: &str,
        query: Option<&str>,
        ministry: Option<&str>,
        rule_type: Option<&str>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> String {
        let mut params = HashMap::new();

        if let Some(q) = query {
            params.insert("query".to_string(), q.to_string());
        }
        if let Some(m) = ministry {
            params.insert("ministry".to_string(), m.to_string());
        }
        if let Some(rt) = rule_type {
            params.insert("rule_type".to_string(), rt.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }

        Self::generate_key(ApiType::Admrul, endpoint, &params, None)
    }

    /// Generate key for EXPC (Legal Interpretation) API
    pub fn expc_key(
        endpoint: &str,
        query: Option<&str>,
        interpretation_type: Option<&str>,
        requesting_agency: Option<&str>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> String {
        let mut params = HashMap::new();

        if let Some(q) = query {
            params.insert("query".to_string(), q.to_string());
        }
        if let Some(it) = interpretation_type {
            params.insert("interpretation_type".to_string(), it.to_string());
        }
        if let Some(ra) = requesting_agency {
            params.insert("requesting_agency".to_string(), ra.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }

        Self::generate_key(ApiType::Expc, endpoint, &params, None)
    }

    /// Generate key for unified search across multiple APIs
    pub fn unified_search_key(
        query: &str,
        apis: &[ApiType],
        page: Option<u32>,
        size: Option<u32>,
    ) -> String {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());

        // Include API types in sorted order
        let mut sorted_apis: Vec<_> = apis.iter().map(|api| api.as_str()).collect();
        sorted_apis.sort();
        let apis_str = sorted_apis.join(",");
        params.insert("apis".to_string(), apis_str);

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }

        Self::generate_key(ApiType::All, "unified_search", &params, None)
    }

    /// Validate cache key format
    pub fn is_valid_key(key: &str) -> bool {
        // Expected format: "api_type:hash"
        if let Some((api_part, hash_part)) = key.split_once(':') {
            // Check if API type is valid
            if api_part.parse::<ApiType>().is_err() {
                return false;
            }

            // Check if hash part looks like a hex string (64 chars for SHA256)
            hash_part.len() == 64 && hash_part.chars().all(|c| c.is_ascii_hexdigit())
        } else {
            false
        }
    }

    /// Extract API type from cache key
    pub fn extract_api_type(key: &str) -> Option<ApiType> {
        if let Some((api_part, _)) = key.split_once(':') {
            api_part.parse::<ApiType>().ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_consistency() {
        let mut params = HashMap::new();
        params.insert("query".to_string(), "test".to_string());
        params.insert("page".to_string(), "1".to_string());

        let key1 = CacheKeyGenerator::generate_key(ApiType::Nlic, "/api/search", &params, None);

        let key2 = CacheKeyGenerator::generate_key(ApiType::Nlic, "/api/search", &params, None);

        assert_eq!(key1, key2, "Keys should be consistent");
    }

    #[test]
    fn test_generate_key_different_params() {
        let mut params1 = HashMap::new();
        params1.insert("query".to_string(), "test1".to_string());

        let mut params2 = HashMap::new();
        params2.insert("query".to_string(), "test2".to_string());

        let key1 = CacheKeyGenerator::generate_key(ApiType::Nlic, "/api/search", &params1, None);

        let key2 = CacheKeyGenerator::generate_key(ApiType::Nlic, "/api/search", &params2, None);

        assert_ne!(key1, key2, "Keys should be different for different params");
    }

    #[test]
    fn test_parameter_order_independence() {
        let mut params1 = HashMap::new();
        params1.insert("a".to_string(), "1".to_string());
        params1.insert("b".to_string(), "2".to_string());

        let mut params2 = HashMap::new();
        params2.insert("b".to_string(), "2".to_string());
        params2.insert("a".to_string(), "1".to_string());

        let key1 = CacheKeyGenerator::generate_key(ApiType::Nlic, "/api/search", &params1, None);

        let key2 = CacheKeyGenerator::generate_key(ApiType::Nlic, "/api/search", &params2, None);

        assert_eq!(
            key1, key2,
            "Keys should be same regardless of parameter order"
        );
    }

    #[test]
    fn test_key_validation() {
        let valid_key = "nlic:a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890";
        let invalid_key1 = "invalid_format";
        let invalid_key2 = "nlic:short_hash";
        let invalid_key3 =
            "unknown_api:a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890";

        assert!(CacheKeyGenerator::is_valid_key(valid_key));
        assert!(!CacheKeyGenerator::is_valid_key(invalid_key1));
        assert!(!CacheKeyGenerator::is_valid_key(invalid_key2));
        assert!(!CacheKeyGenerator::is_valid_key(invalid_key3));
    }

    #[test]
    fn test_extract_api_type() {
        let key = "nlic:a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890";
        let api_type = CacheKeyGenerator::extract_api_type(key);
        assert_eq!(api_type, Some(ApiType::Nlic));

        let invalid_key = "invalid_format";
        let api_type = CacheKeyGenerator::extract_api_type(invalid_key);
        assert_eq!(api_type, None);
    }

    #[test]
    fn test_nlic_key_generation() {
        let key1 = CacheKeyGenerator::nlic_key(
            "/api/search",
            Some("테스트"),
            Some("law"),
            Some(1),
            Some(10),
        );

        let key2 = CacheKeyGenerator::nlic_key(
            "/api/search",
            Some("테스트"),
            Some("law"),
            Some(1),
            Some(10),
        );

        assert_eq!(key1, key2);
        assert!(key1.starts_with("nlic:"));
    }

    #[test]
    fn test_unified_search_key() {
        let apis = vec![ApiType::Nlic, ApiType::Elis, ApiType::Prec];
        let key1 = CacheKeyGenerator::unified_search_key("test", &apis, Some(1), Some(10));

        // Test with different order - should produce same key
        let apis2 = vec![ApiType::Elis, ApiType::Nlic, ApiType::Prec];
        let key2 = CacheKeyGenerator::unified_search_key("test", &apis2, Some(1), Some(10));

        assert_eq!(key1, key2);
        assert!(key1.starts_with("all:"));
    }
}

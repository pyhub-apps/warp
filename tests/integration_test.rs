use warp::api::{ApiType, types::*};
use warp::config::Config;
use warp::cli::OutputFormat;

#[test]
fn test_api_type_from_str() {
    assert_eq!(ApiType::from_str("nlic"), Some(ApiType::Nlic));
    assert_eq!(ApiType::from_str("law"), Some(ApiType::Nlic));
    assert_eq!(ApiType::from_str("elis"), Some(ApiType::Elis));
    assert_eq!(ApiType::from_str("ordinance"), Some(ApiType::Elis));
    assert_eq!(ApiType::from_str("invalid"), None);
}

#[test]
fn test_api_type_display_name() {
    assert_eq!(ApiType::Nlic.display_name(), "국가법령정보센터");
    assert_eq!(ApiType::Elis.display_name(), "자치법규정보시스템");
    assert_eq!(ApiType::Prec.display_name(), "판례");
}

#[test]
fn test_unified_search_request_default() {
    let request = UnifiedSearchRequest::default();
    assert_eq!(request.page_no, 1);
    assert_eq!(request.page_size, 50);
    assert!(request.query.is_empty());
}

#[test]
fn test_config_path() {
    let path = Config::config_path();
    assert!(path.is_ok());
    let path = path.unwrap();
    assert!(path.to_string_lossy().contains(".pyhub/warp"));
}

#[cfg(test)]
mod api_tests {
    use super::*;
    use mockito::{Server, Matcher};
    use warp::api::{client::{ClientConfig, LegalApiClient}, nlic::NlicClient};

    #[tokio::test]
    async fn test_nlic_client_no_api_key() {
        let config = ClientConfig::default();
        let client = NlicClient::new(config);
        
        let request = UnifiedSearchRequest {
            query: "test".to_string(),
            ..Default::default()
        };
        
        let result = client.search(request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            warp::error::WarpError::NoApiKey => (),
            _ => panic!("Expected NoApiKey error"),
        }
    }

    #[tokio::test]
    async fn test_nlic_search_mock() {
        let mut server = Server::new_async().await;
        let _m = server.mock("GET", "/DRF/lawSearch.do")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("OC".to_string(), "test_key".to_string()),
                Matcher::UrlEncoded("query".to_string(), "민법".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "totalCnt": 1,
                "page": 1,
                "display": 50,
                "law": [{
                    "법령ID": "001234",
                    "법령명한글": "민법",
                    "법령일련번호": "00001",
                    "법종구분명": "법률",
                    "소관부처명": "법무부",
                    "시행일자": "20210101",
                    "개정일자": "20201231"
                }]
            }"#)
            .create_async()
            .await;

        let config = ClientConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        };
        
        // Note: This test would require modifying the NlicClient to accept a base URL
        // For now, this is a template for future testing
    }
}

#[cfg(test)]
mod formatter_tests {
    use super::*;
    use warp::output::format_search_response;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_format_search_json() {
        let response = SearchResponse {
            total_count: 1,
            page_no: 1,
            page_size: 50,
            items: vec![SearchItem {
                id: "123".to_string(),
                title: "Test Law".to_string(),
                law_no: Some("001".to_string()),
                law_type: Some("법률".to_string()),
                department: Some("법무부".to_string()),
                enforcement_date: Some("20210101".to_string()),
                revision_date: None,
                summary: None,
                source: "NLIC".to_string(),
                metadata: HashMap::new(),
            }],
            source: "NLIC".to_string(),
            timestamp: Utc::now(),
        };

        let result = format_search_response(&response, OutputFormat::Json);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"total_count\": 1"));
        assert!(json.contains("Test Law"));
    }

    #[test]
    fn test_format_search_table() {
        let response = SearchResponse {
            total_count: 2,
            page_no: 1,
            page_size: 50,
            items: vec![
                SearchItem {
                    id: "123".to_string(),
                    title: "민법".to_string(),
                    law_no: Some("001".to_string()),
                    law_type: Some("법률".to_string()),
                    department: Some("법무부".to_string()),
                    enforcement_date: Some("20210101".to_string()),
                    revision_date: None,
                    summary: None,
                    source: "NLIC".to_string(),
                    metadata: HashMap::new(),
                },
                SearchItem {
                    id: "124".to_string(),
                    title: "형법".to_string(),
                    law_no: Some("002".to_string()),
                    law_type: Some("법률".to_string()),
                    department: Some("법무부".to_string()),
                    enforcement_date: Some("20210101".to_string()),
                    revision_date: None,
                    summary: None,
                    source: "NLIC".to_string(),
                    metadata: HashMap::new(),
                },
            ],
            source: "NLIC".to_string(),
            timestamp: Utc::now(),
        };

        let result = format_search_response(&response, OutputFormat::Table);
        assert!(result.is_ok());
        let table = result.unwrap();
        assert!(table.contains("민법"));
        assert!(table.contains("형법"));
        assert!(table.contains("Total: 2"));
    }
}
#[cfg(test)]
mod tests {
    use super::super::error::WarpError;

    #[test]
    fn test_no_api_key_error() {
        let error = WarpError::NoApiKey;
        
        // Check error message contains emoji and Korean text
        let msg = error.to_string();
        assert!(msg.contains("🔑"));
        assert!(msg.contains("API 키가 설정되지 않았습니다"));
        
        // Check hint is provided
        let hint = error.hint();
        assert!(hint.is_some());
        let hint_text = hint.unwrap();
        assert!(hint_text.contains("https://open.law.go.kr"));
        assert!(hint_text.contains("warp config set law.nlic.key"));
    }

    #[test]
    fn test_network_error_message() {
        // Test network error message format without creating actual reqwest::Error
        // The actual network error testing would be done in integration tests
        
        // We can test other network-related error types though
        let error = WarpError::Other("네트워크 연결 실패".to_string());
        let msg = error.to_string();
        assert!(msg.contains("⚠️"));
    }

    #[test]
    fn test_api_error_with_hint() {
        let error = WarpError::ApiError {
            code: "404".to_string(),
            message: "Not found".to_string(),
            hint: Some("자료를 찾을 수 없습니다".to_string()),
        };
        
        // Check error message format
        let msg = error.to_string();
        assert!(msg.contains("⚠️"));
        assert!(msg.contains("API 오류"));
        assert!(msg.contains("404"));
        assert!(msg.contains("Not found"));
        
        // Check custom hint is preserved
        let hint = error.hint();
        assert!(hint.is_some());
        assert_eq!(hint.unwrap(), "자료를 찾을 수 없습니다");
    }

    #[test]
    fn test_rate_limit_error() {
        let error = WarpError::RateLimit;
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("⏳"));
        assert!(msg.contains("요청 한도 초과"));
        
        // Check recovery hint
        let hint = error.hint();
        assert!(hint.is_some());
        let hint_text = hint.unwrap();
        assert!(hint_text.contains("잠시 후"));
        assert!(hint_text.contains("다시 시도"));
    }

    #[test]
    fn test_parse_error() {
        let error = WarpError::Parse("Invalid XML response".to_string());
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("🔍"));
        assert!(msg.contains("응답 파싱 오류"));
        assert!(msg.contains("Invalid XML response"));
        
        // Check hint for XML/JSON errors
        let hint = error.hint();
        assert!(hint.is_some());
        let hint_text = hint.unwrap();
        assert!(hint_text.contains("API 응답 형식"));
        assert!(hint_text.contains("--verbose"));
    }

    #[test]
    fn test_invalid_input_error() {
        let error = WarpError::InvalidInput("페이지 번호는 1 이상이어야 합니다".to_string());
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("❌"));
        assert!(msg.contains("잘못된 입력"));
        assert!(msg.contains("페이지 번호"));
        
        // Check hint
        let hint = error.hint();
        assert!(hint.is_some());
        let hint_text = hint.unwrap();
        assert!(hint_text.contains("입력한 값을 다시 확인"));
        assert!(hint_text.contains("warp --help"));
    }

    #[test]
    fn test_server_error() {
        let error = WarpError::ServerError("503 Service Unavailable".to_string());
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("🚨"));
        assert!(msg.contains("서버 오류"));
        assert!(msg.contains("503"));
        
        // Check recovery hint
        let hint = error.hint();
        assert!(hint.is_some());
        let hint_text = hint.unwrap();
        assert!(hint_text.contains("서버에 일시적인 문제"));
        assert!(hint_text.contains("law.go.kr"));
    }

    #[test]
    fn test_io_error_permission_denied() {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Permission denied"
        );
        let error = WarpError::Io(io_error);
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("💾"));
        assert!(msg.contains("파일 시스템 오류"));
        
        // Check permission-specific hint
        let hint = error.hint();
        assert!(hint.is_some());
        let hint_text = hint.unwrap();
        assert!(hint_text.contains("권한이 없습니다"));
        assert!(hint_text.contains("sudo"));
    }

    #[test]
    fn test_io_error_not_found() {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found"
        );
        let error = WarpError::Io(io_error);
        
        // Check hint for not found error
        let hint = error.hint();
        assert!(hint.is_some());
        let hint_text = hint.unwrap();
        assert!(hint_text.contains("찾을 수 없습니다"));
        assert!(hint_text.contains("경로를 다시 확인"));
    }

    #[test]
    fn test_config_error() {
        let error = WarpError::Config("Invalid configuration format".to_string());
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("⚙️"));
        assert!(msg.contains("설정 오류"));
        assert!(msg.contains("Invalid configuration format"));
    }

    #[test]
    fn test_authentication_failed() {
        let error = WarpError::AuthenticationFailed("Invalid API key".to_string());
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("🔐"));
        assert!(msg.contains("인증 실패"));
        assert!(msg.contains("Invalid API key"));
        
        // Check authentication-specific hint
        let hint = error.hint();
        assert!(hint.is_some());
        let hint_text = hint.unwrap();
        assert!(hint_text.contains("API 키가 올바른지"));
        assert!(hint_text.contains("warp config get"));
    }

    #[test]
    fn test_is_retryable() {
        // Retryable errors
        // Note: We can't easily create a reqwest::Error in unit tests,
        // but we can test the other retryable error types
        assert!(WarpError::Timeout(30).is_retryable());
        assert!(WarpError::ServerError("503".to_string()).is_retryable());
        assert!(WarpError::RateLimit.is_retryable());
        
        // Non-retryable errors
        assert!(!WarpError::NoApiKey.is_retryable());
        assert!(!WarpError::InvalidInput("bad input".to_string()).is_retryable());
        assert!(!WarpError::AuthenticationFailed("bad key".to_string()).is_retryable());
        assert!(!WarpError::Parse("bad format".to_string()).is_retryable());
    }

    #[test]
    fn test_not_found_error() {
        let error = WarpError::NotFound("법령 ID 12345".to_string());
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("🔎"));
        assert!(msg.contains("찾을 수 없음"));
        assert!(msg.contains("법령 ID 12345"));
        
        // Check hint with specific item
        let hint = error.hint();
        assert!(hint.is_some());
        let hint_text = hint.unwrap();
        assert!(hint_text.contains("법령 ID 12345"));
        assert!(hint_text.contains("항목을 찾을 수 없습니다"));
        assert!(hint_text.contains("다른 검색어"));
    }

    #[test]
    fn test_csv_error() {
        // Create a CSV error (this would normally come from csv crate)
        let csv_error = csv::Error::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid CSV format"
        ));
        let error = WarpError::Csv(csv_error);
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("📊"));
        assert!(msg.contains("CSV 처리 오류"));
    }

    #[test]
    fn test_serialization_error() {
        // Create a JSON error
        let json_str = "{invalid json}";
        let json_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let error = WarpError::Serialization(json_error);
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("📄"));
        assert!(msg.contains("데이터 변환 오류"));
    }

    #[test]
    fn test_other_error() {
        let error = WarpError::Other("예상치 못한 오류가 발생했습니다".to_string());
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("⚠️"));
        assert!(msg.contains("예상치 못한 오류"));
    }

    #[test]
    fn test_timeout_error_message() {
        // Test timeout error message and hint
        let error = WarpError::Timeout(30);
        
        // Check error message
        let msg = error.to_string();
        assert!(msg.contains("⏱️"));
        assert!(msg.contains("시간 초과"));
        assert!(msg.contains("30초"));
        
        // Note: Network timeout errors would be tested in integration tests
        // where we can create actual reqwest::Error instances
    }
}
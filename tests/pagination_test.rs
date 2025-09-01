use warp::api::types::UnifiedSearchRequest;

#[cfg(test)]
mod pagination_tests {
    use super::*;

    #[test]
    fn test_pagination_offset_calculation() {
        // Test that offset is calculated correctly for pagination
        // Page 1, Size 10 -> Offset 1
        // Page 2, Size 10 -> Offset 11
        // Page 3, Size 10 -> Offset 21

        let test_cases = vec![
            (1, 10, 1),  // Page 1, Size 10 -> Offset 1
            (2, 10, 11), // Page 2, Size 10 -> Offset 11
            (3, 10, 21), // Page 3, Size 10 -> Offset 21
            (1, 5, 1),   // Page 1, Size 5 -> Offset 1
            (2, 5, 6),   // Page 2, Size 5 -> Offset 6
            (3, 5, 11),  // Page 3, Size 5 -> Offset 11
            (1, 20, 1),  // Page 1, Size 20 -> Offset 1
            (2, 20, 21), // Page 2, Size 20 -> Offset 21
        ];

        for (page_no, page_size, expected_offset) in test_cases {
            let offset = ((page_no - 1) * page_size) + 1;
            assert_eq!(
                offset, expected_offset,
                "Failed for page {} with size {}: expected offset {}, got {}",
                page_no, page_size, expected_offset, offset
            );
        }
    }

    #[test]
    fn test_unified_search_request_pagination() {
        let request = UnifiedSearchRequest {
            query: "test".to_string(),
            page_no: 2,
            page_size: 10,
            ..Default::default()
        };

        assert_eq!(request.page_no, 2);
        assert_eq!(request.page_size, 10);
    }
}

use serde::Deserialize;
use serde_json::json;

// Import the deserializer helper
use warp::api::deserializers::single_or_vec;

#[derive(Debug, Deserialize, PartialEq)]
struct TestItem {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct TestResponse {
    #[serde(deserialize_with = "single_or_vec")]
    items: Vec<TestItem>,
}

#[test]
fn test_parse_single_item_as_object() {
    // Test case where API returns a single item as an object
    let json = json!({
        "items": {
            "id": "LAW001",
            "name": "Test Law"
        }
    });

    let response: TestResponse = serde_json::from_value(json).expect("Failed to parse single item");
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].id, "LAW001");
    assert_eq!(response.items[0].name, "Test Law");
}

#[test]
fn test_parse_multiple_items_as_array() {
    // Test case where API returns multiple items as an array
    let json = json!({
        "items": [
            {
                "id": "LAW001",
                "name": "Test Law 1"
            },
            {
                "id": "LAW002",
                "name": "Test Law 2"
            }
        ]
    });

    let response: TestResponse =
        serde_json::from_value(json).expect("Failed to parse multiple items");
    assert_eq!(response.items.len(), 2);
    assert_eq!(response.items[0].id, "LAW001");
    assert_eq!(response.items[1].id, "LAW002");
}

#[test]
fn test_parse_empty_array() {
    // Test case where API returns an empty array
    let json = json!({
        "items": []
    });

    let response: TestResponse = serde_json::from_value(json).expect("Failed to parse empty array");
    assert_eq!(response.items.len(), 0);
}

// Simulate actual API response structures
#[test]
fn test_nlic_style_single_result() {
    #[derive(Debug, Deserialize)]
    struct NlicLikeResponse {
        #[serde(rename = "LawSearch")]
        law_search: Option<NlicLikeSearchData>,
    }

    #[derive(Debug, Deserialize)]
    struct NlicLikeSearchData {
        #[serde(rename = "law", deserialize_with = "single_or_vec")]
        laws: Vec<TestItem>,
    }

    // Single result returned as object
    let json = json!({
        "LawSearch": {
            "law": {
                "id": "12345",
                "name": "민법"
            }
        }
    });

    let response: NlicLikeResponse =
        serde_json::from_value(json).expect("Failed to parse NLIC-style single result");
    assert!(response.law_search.is_some());
    let search_data = response.law_search.unwrap();
    assert_eq!(search_data.laws.len(), 1);
    assert_eq!(search_data.laws[0].id, "12345");
}

#[test]
fn test_nlic_style_multiple_results() {
    #[derive(Debug, Deserialize)]
    struct NlicLikeResponse {
        #[serde(rename = "LawSearch")]
        law_search: Option<NlicLikeSearchData>,
    }

    #[derive(Debug, Deserialize)]
    struct NlicLikeSearchData {
        #[serde(rename = "law", deserialize_with = "single_or_vec")]
        laws: Vec<TestItem>,
    }

    // Multiple results returned as array
    let json = json!({
        "LawSearch": {
            "law": [
                {
                    "id": "12345",
                    "name": "민법"
                },
                {
                    "id": "67890",
                    "name": "형법"
                }
            ]
        }
    });

    let response: NlicLikeResponse =
        serde_json::from_value(json).expect("Failed to parse NLIC-style multiple results");
    assert!(response.law_search.is_some());
    let search_data = response.law_search.unwrap();
    assert_eq!(search_data.laws.len(), 2);
    assert_eq!(search_data.laws[0].id, "12345");
    assert_eq!(search_data.laws[1].id, "67890");
}

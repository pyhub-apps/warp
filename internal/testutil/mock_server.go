package testutil

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
)

// MockServer represents a mock API server for testing
type MockServer struct {
	*httptest.Server
	Responses map[string]MockResponse
}

// MockResponse defines a mock response for a specific query
type MockResponse struct {
	StatusCode int
	Body       interface{}
	Error      bool
}

// NewMockServer creates a new mock API server
func NewMockServer() *MockServer {
	ms := &MockServer{
		Responses: make(map[string]MockResponse),
	}

	ms.Server = httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		query := r.URL.Query().Get("query")
		oc := r.URL.Query().Get("OC")

		// Check API key
		if oc == "" {
			w.WriteHeader(http.StatusUnauthorized)
			json.NewEncoder(w).Encode(map[string]string{
				"error": "API key required",
			})
			return
		}

		if oc == "INVALID_KEY" {
			w.WriteHeader(http.StatusForbidden)
			json.NewEncoder(w).Encode(map[string]string{
				"error": "Invalid API key",
			})
			return
		}

		// Return mock response based on query
		if response, exists := ms.Responses[query]; exists {
			w.WriteHeader(response.StatusCode)
			if response.Error {
				json.NewEncoder(w).Encode(map[string]string{
					"error": "Server error",
				})
			} else {
				json.NewEncoder(w).Encode(response.Body)
			}
			return
		}

		// Default response
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(DefaultMockResponse(query))
	}))

	// Set default responses
	ms.SetupDefaultResponses()

	return ms
}

// SetupDefaultResponses sets up common test responses
func (ms *MockServer) SetupDefaultResponses() {
	// Personal Information Protection Act response
	ms.Responses["개인정보 보호법"] = MockResponse{
		StatusCode: http.StatusOK,
		Body: map[string]interface{}{
			"totalCnt": 3,
			"page":     1,
			"law": []map[string]interface{}{
				{
					"법령ID":   "173995",
					"법령명한글":  "개인정보 보호법",
					"법령구분명":  "법률",
					"소관부처명":  "개인정보보호위원회",
					"시행일자":   "20240315",
					"법령상세링크": "https://www.law.go.kr/법령/개인정보보호법",
				},
				{
					"법령ID":   "173996",
					"법령명한글":  "개인정보 보호법 시행령",
					"법령구분명":  "대통령령",
					"소관부처명":  "개인정보보호위원회",
					"시행일자":   "20240315",
					"법령상세링크": "https://www.law.go.kr/법령/개인정보보호법시행령",
				},
				{
					"법령ID":   "173997",
					"법령명한글":  "개인정보 보호법 시행규칙",
					"법령구분명":  "부령",
					"소관부처명":  "개인정보보호위원회",
					"시행일자":   "20240315",
					"법령상세링크": "https://www.law.go.kr/법령/개인정보보호법시행규칙",
				},
			},
		},
	}

	// Empty result response
	ms.Responses["없는법령"] = MockResponse{
		StatusCode: http.StatusOK,
		Body: map[string]interface{}{
			"totalCnt": 0,
			"page":     1,
			"law":      []map[string]interface{}{},
		},
	}

	// Error response
	ms.Responses["error"] = MockResponse{
		StatusCode: http.StatusInternalServerError,
		Error:      true,
	}

	// Traffic Law response (for JSON format test)
	ms.Responses["도로교통법"] = MockResponse{
		StatusCode: http.StatusOK,
		Body: map[string]interface{}{
			"totalCnt": 1,
			"page":     1,
			"law": []map[string]interface{}{
				{
					"법령ID":   "174001",
					"법령명한글":  "도로교통법",
					"법령구분명":  "법률",
					"소관부처명":  "경찰청",
					"시행일자":   "20240401",
					"법령상세링크": "https://www.law.go.kr/법령/도로교통법",
				},
			},
		},
	}
}

// DefaultMockResponse returns a default response for unknown queries
func DefaultMockResponse(query string) map[string]interface{} {
	return map[string]interface{}{
		"totalCnt": 1,
		"page":     1,
		"law": []map[string]interface{}{
			{
				"법령ID":   "999999",
				"법령명한글":  query,
				"법령구분명":  "법률",
				"소관부처명":  "테스트부처",
				"시행일자":   "20240101",
				"법령상세링크": "https://www.law.go.kr/test",
			},
		},
	}
}

// AddResponse adds a custom response for a specific query
func (ms *MockServer) AddResponse(query string, response MockResponse) {
	ms.Responses[query] = response
}

// GetURL returns the mock server URL
func (ms *MockServer) GetURL() string {
	return ms.Server.URL
}

// GetSearchURL returns the search endpoint URL for the given API type
func (ms *MockServer) GetSearchURL(apiType string) string {
	u, _ := url.Parse(ms.Server.URL)

	// Set path based on API type
	switch apiType {
	case "nlic":
		u.Path = "/DRF/lawSearch.do"
	case "prec":
		u.Path = "/PREC/searchList.do"
	case "admr":
		u.Path = "/ADMR/searchList.do"
	case "expc":
		u.Path = "/EXPC/searchList.do"
	case "elis":
		u.Path = "/ELIS/searchList.do"
	default:
		u.Path = "/DRF/lawSearch.do"
	}

	return u.String()
}

// Close shuts down the mock server
func (ms *MockServer) Close() {
	ms.Server.Close()
}

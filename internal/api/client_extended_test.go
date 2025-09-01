package api

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestClient_GetDetail(t *testing.T) {
	tests := []struct {
		name       string
		lawID      string
		response   interface{}
		statusCode int
		wantErr    bool
	}{
		{
			name:  "Successful detail retrieval",
			lawID: "011357",
			response: &LawDetail{
				LawInfo: LawInfo{
					ID:         "011357",
					Name:       "개인정보 보호법",
					LawType:    "법률",
					Department: "개인정보보호위원회",
				},
			},
			statusCode: http.StatusOK,
			wantErr:    false,
		},
		{
			name:       "Empty law ID",
			lawID:      "",
			response:   nil,
			statusCode: http.StatusBadRequest,
			wantErr:    true,
		},
		{
			name:       "Server error",
			lawID:      "011357",
			response:   nil,
			statusCode: http.StatusInternalServerError,
			wantErr:    true,
		},
		{
			name:       "Not found",
			lawID:      "999999",
			response:   nil,
			statusCode: http.StatusNotFound,
			wantErr:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(tt.statusCode)
				if tt.response != nil {
					json.NewEncoder(w).Encode(tt.response)
				}
			}))
			defer server.Close()

			client := &Client{
				httpClient: &http.Client{Timeout: 5 * time.Second},
				endpoint:   server.URL,
				apiKey:     "test-key",
			}

			ctx := context.Background()
			result, err := client.GetDetail(ctx, tt.lawID)

			if (err != nil) != tt.wantErr {
				t.Errorf("GetDetail() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if !tt.wantErr && result != nil {
				if result.ID != tt.response.(*LawDetail).ID {
					t.Errorf("GetDetail() ID = %v, want %v", result.ID, tt.response.(*LawDetail).ID)
				}
			}
		})
	}
}

func TestClient_GetHistory(t *testing.T) {
	tests := []struct {
		name       string
		lawID      string
		response   interface{}
		statusCode int
		wantErr    bool
	}{
		{
			name:  "Successful history retrieval",
			lawID: "011357",
			response: &LawHistory{
				LawID:   "011357",
				LawName: "개인정보 보호법",
				Histories: []HistoryRecord{
					{
						Type:       "제정",
						PromulNo:   "법률 제10465호",
						Date:       "20110329",
						EffectDate: "20110930",
					},
				},
			},
			statusCode: http.StatusOK,
			wantErr:    false,
		},
		{
			name:       "Empty law ID",
			lawID:      "",
			response:   nil,
			statusCode: http.StatusBadRequest,
			wantErr:    true,
		},
		{
			name:       "Server error",
			lawID:      "011357",
			response:   nil,
			statusCode: http.StatusInternalServerError,
			wantErr:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(tt.statusCode)
				if tt.response != nil {
					json.NewEncoder(w).Encode(tt.response)
				}
			}))
			defer server.Close()

			client := &Client{
				httpClient: &http.Client{Timeout: 5 * time.Second},
				endpoint:   server.URL,
				apiKey:     "test-key",
			}

			ctx := context.Background()
			result, err := client.GetHistory(ctx, tt.lawID)

			if (err != nil) != tt.wantErr {
				t.Errorf("GetHistory() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if !tt.wantErr && result != nil {
				if result.LawID != tt.response.(*LawHistory).LawID {
					t.Errorf("GetHistory() LawID = %v, want %v", result.LawID, tt.response.(*LawHistory).LawID)
				}
			}
		})
	}
}

func TestCreateClient(t *testing.T) {
	tests := []struct {
		name       string
		apiType    APIType
		setupFunc  func()
		cleanupFunc func()
		wantErr    bool
	}{
		{
			name:    "Create NLIC client with API key",
			apiType: APITypeNLIC,
			setupFunc: func() {
				SetAPIKey(APITypeNLIC, "test-nlic-key")
			},
			cleanupFunc: func() {
				SetAPIKey(APITypeNLIC, "")
			},
			wantErr: false,
		},
		{
			name:    "Create PREC client with API key",
			apiType: APITypePrec,
			setupFunc: func() {
				SetAPIKey(APITypePrec, "test-prec-key")
			},
			cleanupFunc: func() {
				SetAPIKey(APITypePrec, "")
			},
			wantErr: false,
		},
		{
			name:    "Create ADMRUL client with API key",
			apiType: APITypeAdmrul,
			setupFunc: func() {
				SetAPIKey(APITypeAdmrul, "test-admrul-key")
			},
			cleanupFunc: func() {
				SetAPIKey(APITypeAdmrul, "")
			},
			wantErr: false,
		},
		{
			name:    "Create EXPC client with API key",
			apiType: APITypeExpc,
			setupFunc: func() {
				SetAPIKey(APITypeExpc, "test-expc-key")
			},
			cleanupFunc: func() {
				SetAPIKey(APITypeExpc, "")
			},
			wantErr: false,
		},
		{
			name:    "Create ELIS client with API key",
			apiType: APITypeELIS,
			setupFunc: func() {
				SetAPIKey(APITypeELIS, "test-elis-key")
			},
			cleanupFunc: func() {
				SetAPIKey(APITypeELIS, "")
			},
			wantErr: false,
		},
		{
			name:    "Create client without API key",
			apiType: APITypeNLIC,
			setupFunc: func() {
				SetAPIKey(APITypeNLIC, "")
			},
			cleanupFunc: func() {},
			wantErr: true,
		},
		{
			name:    "Invalid API type",
			apiType: APIType("invalid"),
			setupFunc: func() {},
			cleanupFunc: func() {},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.setupFunc != nil {
				tt.setupFunc()
			}
			if tt.cleanupFunc != nil {
				defer tt.cleanupFunc()
			}

			client, err := CreateClient(tt.apiType)

			if (err != nil) != tt.wantErr {
				t.Errorf("CreateClient() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if !tt.wantErr && client == nil {
				t.Errorf("CreateClient() returned nil client")
			}
		})
	}
}

func TestGetAPIKeyName(t *testing.T) {
	tests := []struct {
		apiType APIType
		want    string
	}{
		{APITypeNLIC, "law.nlic.key"},
		{APITypePrec, "law.prec.key"},
		{APITypeAdmrul, "law.admrul.key"},
		{APITypeExpc, "law.expc.key"},
		{APITypeELIS, "law.elis.key"},
		{APIType("invalid"), ""},
	}

	for _, tt := range tests {
		t.Run(string(tt.apiType), func(t *testing.T) {
			got := GetAPIKeyName(tt.apiType)
			if got != tt.want {
				t.Errorf("GetAPIKeyName() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestGetAPIEndpointName(t *testing.T) {
	tests := []struct {
		apiType APIType
		want    string
	}{
		{APITypeNLIC, "law.nlic.endpoint"},
		{APITypePrec, "law.prec.endpoint"},
		{APITypeAdmrul, "law.admrul.endpoint"},
		{APITypeExpc, "law.expc.endpoint"},
		{APITypeELIS, "law.elis.endpoint"},
		{APIType("invalid"), ""},
	}

	for _, tt := range tests {
		t.Run(string(tt.apiType), func(t *testing.T) {
			got := GetAPIEndpointName(tt.apiType)
			if got != tt.want {
				t.Errorf("GetAPIEndpointName() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestClient_Context(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Simulate slow response
		time.Sleep(100 * time.Millisecond)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(&SearchResponse{
			TotalCount: 1,
			Page:       1,
		})
	}))
	defer server.Close()

	client := &Client{
		httpClient: &http.Client{Timeout: 5 * time.Second},
		endpoint:   server.URL,
		apiKey:     "test-key",
		retryMax:   1,
	}

	t.Run("Context cancellation", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		
		// Cancel context immediately
		cancel()

		req := &UnifiedSearchRequest{
			Query:    "test",
			PageNo:   1,
			PageSize: 10,
		}

		_, err := client.Search(ctx, req)
		if err == nil {
			t.Error("Expected error due to context cancellation")
		}
	})

	t.Run("Context timeout", func(t *testing.T) {
		ctx, cancel := context.WithTimeout(context.Background(), 50*time.Millisecond)
		defer cancel()

		req := &UnifiedSearchRequest{
			Query:    "test",
			PageNo:   1,
			PageSize: 10,
		}

		_, err := client.Search(ctx, req)
		if err == nil {
			t.Error("Expected error due to context timeout")
		}
	})
}
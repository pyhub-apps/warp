package cmd

import (
	"bytes"
	"strings"
	"testing"

	"github.com/pyhub-apps/pyhub-warp-cli/internal/config"
	"github.com/pyhub-apps/pyhub-warp-cli/internal/testutil"
	"github.com/spf13/viper"
)

func TestPrecedentCommand(t *testing.T) {
	tests := []struct {
		name        string
		args        []string
		wantErr     bool
		wantContain string
		setup       func()
		cleanup     func()
	}{
		{
			name:        "No arguments shows help",
			args:        []string{"precedent"},
			wantErr:     false,
			wantContain: "판례 정보 검색 및 조회",
		},
		{
			name:        "Help flag",
			args:        []string{"precedent", "--help"},
			wantErr:     false,
			wantContain: "대법원 및 각급 법원의 판례",
		},
		{
			name:        "Short alias works",
			args:        []string{"prec", "--help"},
			wantErr:     false,
			wantContain: "판례 정보 검색 및 조회",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.setup != nil {
				tt.setup()
			}
			if tt.cleanup != nil {
				defer tt.cleanup()
			}

			cmd := createTestRootCommand()
			output := &bytes.Buffer{}
			cmd.SetOut(output)
			cmd.SetErr(output)
			cmd.SetArgs(tt.args)

			err := cmd.Execute()
			if (err != nil) != tt.wantErr {
				t.Errorf("Execute() error = %v, wantErr %v", err, tt.wantErr)
			}

			outputStr := output.String()
			if tt.wantContain != "" && !strings.Contains(outputStr, tt.wantContain) {
				t.Errorf("output does not contain expected string.\nwant: %s\ngot: %s", tt.wantContain, outputStr)
			}
		})
	}
}

func TestPrecedentSearchCommand(t *testing.T) {
	tests := []struct {
		name        string
		args        []string
		apiKey      string
		wantErr     bool
		wantContain string
		setup       func()
		cleanup     func()
	}{
		{
			name:        "No arguments",
			args:        []string{"precedent", "search"},
			wantErr:     true,
			wantContain: "requires at least 1 arg",
		},
		{
			name:        "Empty search query",
			args:        []string{"precedent", "search", ""},
			wantErr:     true,
			wantContain: "검색어가 비어있습니다",
		},
		{
			name:        "Search with valid query (no API key)",
			args:        []string{"precedent", "search", "계약 해지"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with flags",
			args:        []string{"precedent", "search", "손해배상", "--page", "2", "--size", "20"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with JSON format",
			args:        []string{"precedent", "search", "부당이득", "--format", "json"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search help",
			args:        []string{"precedent", "search", "--help"},
			wantErr:     false,
			wantContain: "키워드로 판례를 검색합니다",
		},
		{
			name:        "Search with alias",
			args:        []string{"prec", "s", "계약"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with API key",
			args:        []string{"precedent", "search", "계약"},
			apiKey:      "test-api-key",
			wantErr:     false,
			wantContain: "",
			setup: func() {
				// Set up mock server for testing with API key
				server := testutil.NewMockServer()
				server.SetupDefaultResponses()
				viper.Set("law.prec.key", "test-api-key")
				viper.Set("law.prec.endpoint", server.GetSearchURL("prec"))
			},
			cleanup: func() {
				viper.Set("law.prec.key", "")
				viper.Set("law.prec.endpoint", "")
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Initialize config
			config.Initialize()

			if tt.setup != nil {
				tt.setup()
			}
			if tt.cleanup != nil {
				defer tt.cleanup()
			}

			if tt.apiKey != "" {
				viper.Set("law.prec.key", tt.apiKey)
				defer viper.Set("law.prec.key", "")
			}

			cmd := createTestRootCommand()
			output := &bytes.Buffer{}
			cmd.SetOut(output)
			cmd.SetErr(output)
			cmd.SetArgs(tt.args)

			err := cmd.Execute()
			if (err != nil) != tt.wantErr {
				t.Errorf("Execute() error = %v, wantErr %v", err, tt.wantErr)
			}

			outputStr := output.String()
			if tt.wantContain != "" && !strings.Contains(outputStr, tt.wantContain) {
				t.Errorf("output does not contain expected string.\nwant: %s\ngot: %s", tt.wantContain, outputStr)
			}
		})
	}
}

func TestPrecedentDetailCommand(t *testing.T) {
	tests := []struct {
		name        string
		args        []string
		apiKey      string
		wantErr     bool
		wantContain string
		setup       func()
		cleanup     func()
	}{
		{
			name:        "No arguments",
			args:        []string{"precedent", "detail"},
			wantErr:     true,
			wantContain: "requires at least 1 arg",
		},
		{
			name:        "Empty precedent ID",
			args:        []string{"precedent", "detail", ""},
			wantErr:     true,
			wantContain: "판례 ID가 비어있습니다",
		},
		{
			name:        "Valid precedent ID (no API key)",
			args:        []string{"precedent", "detail", "12345"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Detail help",
			args:        []string{"precedent", "detail", "--help"},
			wantErr:     false,
			wantContain: "판례 상세 정보를 조회합니다",
		},
		{
			name:        "Detail with alias",
			args:        []string{"prec", "d", "12345"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Detail with JSON format",
			args:        []string{"precedent", "detail", "12345", "--format", "json"},
			wantErr:     false,
			wantContain: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Initialize config
			config.Initialize()

			if tt.setup != nil {
				tt.setup()
			}
			if tt.cleanup != nil {
				defer tt.cleanup()
			}

			if tt.apiKey != "" {
				viper.Set("law.prec.key", tt.apiKey)
				defer viper.Set("law.prec.key", "")
			}

			cmd := createTestRootCommand()
			output := &bytes.Buffer{}
			cmd.SetOut(output)
			cmd.SetErr(output)
			cmd.SetArgs(tt.args)

			err := cmd.Execute()
			if (err != nil) != tt.wantErr {
				t.Errorf("Execute() error = %v, wantErr %v", err, tt.wantErr)
			}

			outputStr := output.String()
			if tt.wantContain != "" && !strings.Contains(outputStr, tt.wantContain) {
				t.Errorf("output does not contain expected string.\nwant: %s\ngot: %s", tt.wantContain, outputStr)
			}
		})
	}
}
package cmd

import (
	"bytes"
	"strings"
	"testing"

	"github.com/pyhub-apps/pyhub-warp-cli/internal/config"
	"github.com/pyhub-apps/pyhub-warp-cli/internal/testutil"
	"github.com/spf13/viper"
)

func TestAdmruleCommand(t *testing.T) {
	tests := []struct {
		name        string
		args        []string
		wantErr     bool
		wantContain string
	}{
		{
			name:        "No arguments shows help",
			args:        []string{"admrule"},
			wantErr:     false,
			wantContain: "행정규칙 정보 검색 및 조회",
		},
		{
			name:        "Help flag",
			args:        []string{"admrule", "--help"},
			wantErr:     false,
			wantContain: "정부 부처의 행정규칙",
		},
		{
			name:        "Short alias works",
			args:        []string{"admr", "--help"},
			wantErr:     false,
			wantContain: "행정규칙 정보 검색 및 조회",
		},
		{
			name:        "Rule alias works",
			args:        []string{"rule", "--help"},
			wantErr:     false,
			wantContain: "행정규칙 정보 검색 및 조회",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
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

func TestAdmruleSearchCommand(t *testing.T) {
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
			args:        []string{"admrule", "search"},
			wantErr:     true,
			wantContain: "requires at least 1 arg",
		},
		{
			name:        "Empty search query",
			args:        []string{"admrule", "search", ""},
			wantErr:     true,
			wantContain: "검색어가 비어있습니다",
		},
		{
			name:        "Search with valid query (no API key)",
			args:        []string{"admrule", "search", "공공기관"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with flags",
			args:        []string{"admrule", "search", "개인정보", "--page", "2", "--size", "20"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with JSON format",
			args:        []string{"admrule", "search", "고시", "--format", "json"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search help",
			args:        []string{"admrule", "search", "--help"},
			wantErr:     false,
			wantContain: "키워드로 행정규칙을 검색합니다",
		},
		{
			name:        "Search with alias",
			args:        []string{"admr", "s", "훈령"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with rule alias",
			args:        []string{"rule", "search", "예규"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with API key",
			args:        []string{"admrule", "search", "행정처분"},
			apiKey:      "test-api-key",
			wantErr:     false,
			wantContain: "",
			setup: func() {
				// Set up mock server for testing with API key
				server := testutil.NewMockServer()
				server.SetupDefaultResponses()
				viper.Set("law.admr.key", "test-api-key")
				viper.Set("law.admr.endpoint", server.GetSearchURL("admr"))
			},
			cleanup: func() {
				viper.Set("law.admr.key", "")
				viper.Set("law.admr.endpoint", "")
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
				viper.Set("law.admr.key", tt.apiKey)
				defer viper.Set("law.admr.key", "")
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

func TestAdmruleDetailCommand(t *testing.T) {
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
			args:        []string{"admrule", "detail"},
			wantErr:     true,
			wantContain: "requires at least 1 arg",
		},
		{
			name:        "Empty admrule ID",
			args:        []string{"admrule", "detail", ""},
			wantErr:     true,
			wantContain: "행정규칙 ID가 비어있습니다",
		},
		{
			name:        "Valid admrule ID (no API key)",
			args:        []string{"admrule", "detail", "12345"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Detail help",
			args:        []string{"admrule", "detail", "--help"},
			wantErr:     false,
			wantContain: "행정규칙 상세 정보를 조회합니다",
		},
		{
			name:        "Detail with alias",
			args:        []string{"admr", "d", "12345"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Detail with rule alias",
			args:        []string{"rule", "detail", "12345"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Detail with JSON format",
			args:        []string{"admrule", "detail", "12345", "--format", "json"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Detail with markdown format",
			args:        []string{"admrule", "detail", "12345", "--format", "markdown"},
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
				viper.Set("law.admr.key", tt.apiKey)
				defer viper.Set("law.admr.key", "")
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

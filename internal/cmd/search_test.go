package cmd

import (
	"bytes"
	"strings"
	"testing"

	"github.com/pyhub-apps/pyhub-warp-cli/internal/config"
	"github.com/pyhub-apps/pyhub-warp-cli/internal/testutil"
	"github.com/spf13/viper"
)

func TestSearchCommand(t *testing.T) {
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
			name:        "No arguments shows help",
			args:        []string{"search"},
			wantErr:     false,
			wantContain: "법령 및 자치법규 통합 검색",
		},
		{
			name:        "Help flag",
			args:        []string{"search", "--help"},
			wantErr:     false,
			wantContain: "국가법령과 자치법규를 통합하여 검색합니다",
		},
		{
			name:        "Empty search query",
			args:        []string{"search", ""},
			wantErr:     true,
			wantContain: "검색어가 비어있습니다",
		},
		{
			name:        "Search with valid query (no API key)",
			args:        []string{"search", "개인정보"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with multiple words",
			args:        []string{"search", "개인정보", "보호법"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with flags",
			args:        []string{"search", "도로교통법", "--page", "2", "--size", "20"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with JSON format",
			args:        []string{"search", "민법", "--format", "json"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with markdown format",
			args:        []string{"search", "상법", "--format", "markdown"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with CSV format",
			args:        []string{"search", "형법", "--format", "csv"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with HTML format",
			args:        []string{"search", "헌법", "--format", "html"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with HTML-simple format",
			args:        []string{"search", "행정법", "--format", "html-simple"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with source all",
			args:        []string{"search", "조례", "--source", "all"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with source nlic",
			args:        []string{"search", "법률", "--source", "nlic"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with source elis",
			args:        []string{"search", "규칙", "--source", "elis"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with verbose flag",
			args:        []string{"search", "테스트", "-v"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with pagination",
			args:        []string{"search", "법령", "--page", "3", "--size", "100"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Search with API key",
			args:        []string{"search", "개인정보"},
			apiKey:      "test-api-key",
			wantErr:     false,
			wantContain: "",
			setup: func() {
				// Set up mock server for testing with API key
				server := testutil.NewMockServer()
				server.SetupDefaultResponses()
				viper.Set("law.nlic.key", "test-api-key")
				viper.Set("law.nlic.endpoint", server.GetSearchURL("nlic"))
			},
			cleanup: func() {
				viper.Set("law.nlic.key", "")
				viper.Set("law.nlic.endpoint", "")
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
				viper.Set("law.nlic.key", tt.apiKey)
				defer viper.Set("law.nlic.key", "")
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

func TestSearchCommandSources(t *testing.T) {
	tests := []struct {
		name        string
		args        []string
		wantErr     bool
		wantContain string
	}{
		{
			name:        "Invalid source",
			args:        []string{"search", "법령", "--source", "invalid"},
			wantErr:     true,
			wantContain: "유효하지 않은 소스",
		},
		{
			name:        "Source case insensitive - ALL",
			args:        []string{"search", "법령", "--source", "ALL"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Source case insensitive - NLic",
			args:        []string{"search", "법령", "--source", "NLic"},
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Source case insensitive - ElIs",
			args:        []string{"search", "법령", "--source", "ElIs"},
			wantErr:     false,
			wantContain: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Initialize config
			config.Initialize()

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

func TestSearchCommandFormats(t *testing.T) {
	tests := []struct {
		name        string
		format      string
		wantErr     bool
		wantContain string
	}{
		{
			name:        "Table format (default)",
			format:      "",
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Table format (explicit)",
			format:      "table",
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "JSON format",
			format:      "json",
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Markdown format",
			format:      "markdown",
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "CSV format",
			format:      "csv",
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "HTML format",
			format:      "html",
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "HTML-simple format",
			format:      "html-simple",
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Invalid format",
			format:      "invalid",
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Format case insensitive - JSON",
			format:      "JSON",
			wantErr:     false,
			wantContain: "",
		},
		{
			name:        "Format case insensitive - Table",
			format:      "TABLE",
			wantErr:     false,
			wantContain: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Initialize config
			config.Initialize()

			args := []string{"search", "테스트"}
			if tt.format != "" {
				args = append(args, "--format", tt.format)
			}

			cmd := createTestRootCommand()
			output := &bytes.Buffer{}
			cmd.SetOut(output)
			cmd.SetErr(output)
			cmd.SetArgs(args)

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

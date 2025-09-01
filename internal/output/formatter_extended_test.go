package output

import (
	"strings"
	"testing"

	"github.com/pyhub-apps/pyhub-warp-cli/internal/api"
)

func TestFormatDetailToString(t *testing.T) {
	detail := &api.LawDetail{
		LawInfo: api.LawInfo{
			ID:         "011357",
			Name:       "개인정보 보호법",
			LawType:    "법률",
			Department: "개인정보보호위원회",
			EffectDate: "20240315",
		},
		Articles: []api.Article{
			{
				Number:  "제1조",
				Title:   "목적",
				Content: "이 법은 개인정보의 처리 및 보호에 관한 사항을 정함으로써...",
			},
			{
				Number:  "제2조",
				Title:   "정의",
				Content: "이 법에서 사용하는 용어의 뜻은 다음과 같다.",
			},
		},
		Tables: []api.Table{
			{
				Number:  "별표1",
				Title:   "과태료 부과기준",
				Content: "과태료 부과기준은 다음과 같다.",
			},
		},
		SupplementaryProvisions: []api.SupplementaryProvision{
			{
				Number:  "부칙 제1조",
				Content: "이 법은 공포한 날부터 시행한다.",
			},
		},
	}

	tests := []struct {
		name        string
		format      string
		detail      *api.LawDetail
		wantContain string
		wantErr     bool
	}{
		{
			name:        "Table format",
			format:      "table",
			detail:      detail,
			wantContain: "개인정보 보호법",
			wantErr:     false,
		},
		{
			name:        "JSON format",
			format:      "json",
			detail:      detail,
			wantContain: "\"법령ID\": \"011357\"",
			wantErr:     false,
		},
		{
			name:        "Invalid format",
			format:      "invalid",
			detail:      detail,
			wantContain: "",
			wantErr:     true,
		},
		{
			name:        "Nil detail",
			format:      "table",
			detail:      nil,
			wantContain: "",
			wantErr:     true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			f := NewFormatter(tt.format)
			result, err := f.FormatDetailToString(tt.detail)

			if (err != nil) != tt.wantErr {
				t.Errorf("FormatDetailToString() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if tt.wantContain != "" && !strings.Contains(result, tt.wantContain) {
				t.Errorf("Result does not contain expected string.\nwant: %s\ngot: %s", tt.wantContain, result)
			}
		})
	}
}

func TestFormatDetailToStringWithOptions(t *testing.T) {
	detail := &api.LawDetail{
		LawInfo: api.LawInfo{
			ID:         "011357",
			Name:       "개인정보 보호법",
			LawType:    "법률",
			Department: "개인정보보호위원회",
			EffectDate: "20240315",
		},
		Articles: []api.Article{
			{
				Number:  "제1조",
				Title:   "목적",
				Content: "이 법은 개인정보의 처리 및 보호에 관한 사항을 정함으로써...",
			},
		},
		Tables: []api.Table{
			{
				Number:  "별표1",
				Title:   "과태료 부과기준",
				Content: "과태료 부과기준은 다음과 같다.",
			},
		},
		SupplementaryProvisions: []api.SupplementaryProvision{
			{
				Number:  "부칙 제1조",
				Content: "이 법은 공포한 날부터 시행한다.",
			},
		},
	}

	tests := []struct {
		name              string
		format            string
		showArticles      bool
		showTables        bool
		showSupplementary bool
		wantContain       string
		wantNotContain    string
	}{
		{
			name:              "Show articles only",
			format:            "table",
			showArticles:      true,
			showTables:        false,
			showSupplementary: false,
			wantContain:       "제1조",
			wantNotContain:    "별표1",
		},
		{
			name:              "Show tables only",
			format:            "table",
			showArticles:      false,
			showTables:        true,
			showSupplementary: false,
			wantContain:       "별표1",
			wantNotContain:    "제1조",
		},
		{
			name:              "Show supplementary only",
			format:            "table",
			showArticles:      false,
			showTables:        false,
			showSupplementary: true,
			wantContain:       "부칙",
			wantNotContain:    "제1조",
		},
		{
			name:              "Show all",
			format:            "table",
			showArticles:      true,
			showTables:        true,
			showSupplementary: true,
			wantContain:       "제1조",
			wantNotContain:    "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			f := NewFormatter(tt.format)
			result, err := f.FormatDetailToStringWithOptions(detail, tt.showArticles, tt.showTables, tt.showSupplementary)

			if err != nil {
				t.Errorf("FormatDetailToStringWithOptions() error = %v", err)
				return
			}

			if tt.wantContain != "" && !strings.Contains(result, tt.wantContain) {
				t.Errorf("Result does not contain expected string.\nwant: %s\ngot: %s", tt.wantContain, result)
			}

			if tt.wantNotContain != "" && strings.Contains(result, tt.wantNotContain) {
				t.Errorf("Result should not contain string.\nshould not have: %s\ngot: %s", tt.wantNotContain, result)
			}
		})
	}
}

func TestFormatHistoryToString(t *testing.T) {
	history := &api.LawHistory{
		LawID:   "011357",
		LawName: "개인정보 보호법",
		Histories: []api.HistoryRecord{
			{
				Type:       "제정",
				PromulNo:   "법률 제10465호",
				Date:       "20110329",
				EffectDate: "20110930",
				Reason:     "최초 제정",
			},
			{
				Type:       "일부개정",
				PromulNo:   "법률 제14839호",
				Date:       "20170726",
				EffectDate: "20171019",
				Reason:     "일부 조항 개정",
			},
		},
	}

	tests := []struct {
		name        string
		format      string
		history     *api.LawHistory
		wantContain string
		wantErr     bool
	}{
		{
			name:        "Table format",
			format:      "table",
			history:     history,
			wantContain: "개인정보 보호법",
			wantErr:     false,
		},
		{
			name:        "JSON format",
			format:      "json",
			history:     history,
			wantContain: "\"법령ID\": \"011357\"",
			wantErr:     false,
		},
		{
			name:        "Invalid format",
			format:      "invalid",
			history:     history,
			wantContain: "",
			wantErr:     true,
		},
		{
			name:        "Nil history",
			format:      "table",
			history:     nil,
			wantContain: "",
			wantErr:     true,
		},
		{
			name:        "Empty history records",
			format:      "table",
			history:     &api.LawHistory{LawID: "123", LawName: "Test Law", Histories: []api.HistoryRecord{}},
			wantContain: "이력이 없습니다",
			wantErr:     false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			f := NewFormatter(tt.format)
			result, err := f.FormatHistoryToString(tt.history)

			if (err != nil) != tt.wantErr {
				t.Errorf("FormatHistoryToString() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if tt.wantContain != "" && !strings.Contains(result, tt.wantContain) {
				t.Errorf("Result does not contain expected string.\nwant: %s\ngot: %s", tt.wantContain, result)
			}
		})
	}
}

func TestFormatMarkdownToString(t *testing.T) {
	resp := &api.SearchResponse{
		TotalCount: 2,
		Page:       1,
		Laws: []api.LawInfo{
			{
				ID:         "001234",
				Name:       "개인정보 보호법",
				LawType:    "법률",
				Department: "개인정보보호위원회",
				EffectDate: "20200805",
			},
		},
	}

	f := NewFormatter("markdown")
	result, err := f.FormatSearchResultToString(resp)

	if err != nil {
		t.Fatalf("FormatSearchResultToString() error = %v", err)
	}

	expectedContains := []string{
		"## 검색 결과",
		"총 **2**개",
		"| 번호 |",
		"개인정보 보호법",
	}

	for _, expected := range expectedContains {
		if !strings.Contains(result, expected) {
			t.Errorf("Markdown output does not contain expected string: %s", expected)
		}
	}
}

func TestFormatCSVToString(t *testing.T) {
	resp := &api.SearchResponse{
		TotalCount: 1,
		Page:       1,
		Laws: []api.LawInfo{
			{
				ID:         "001234",
				Name:       "개인정보 보호법",
				LawType:    "법률",
				Department: "개인정보보호위원회",
				EffectDate: "20200805",
			},
		},
	}

	f := NewFormatter("csv")
	result, err := f.FormatSearchResultToString(resp)

	if err != nil {
		t.Fatalf("FormatSearchResultToString() error = %v", err)
	}

	expectedContains := []string{
		"번호,법령ID,법령명,법령구분,소관부처,시행일자",
		"1,001234,개인정보 보호법,법률,개인정보보호위원회,2020-08-05",
	}

	for _, expected := range expectedContains {
		if !strings.Contains(result, expected) {
			t.Errorf("CSV output does not contain expected string: %s", expected)
		}
	}
}

func TestFormatHTMLToString(t *testing.T) {
	resp := &api.SearchResponse{
		TotalCount: 1,
		Page:       1,
		Laws: []api.LawInfo{
			{
				ID:         "001234",
				Name:       "개인정보 보호법",
				LawType:    "법률",
				Department: "개인정보보호위원회",
				EffectDate: "20200805",
			},
		},
	}

	f := NewFormatter("html")
	result, err := f.FormatSearchResultToString(resp)

	if err != nil {
		t.Fatalf("FormatSearchResultToString() error = %v", err)
	}

	expectedContains := []string{
		"<!DOCTYPE html>",
		"<html>",
		"<table",
		"개인정보 보호법",
		"</html>",
	}

	for _, expected := range expectedContains {
		if !strings.Contains(result, expected) {
			t.Errorf("HTML output does not contain expected string: %s", expected)
		}
	}
}

func TestFormatHTMLSimpleToString(t *testing.T) {
	resp := &api.SearchResponse{
		TotalCount: 1,
		Page:       1,
		Laws: []api.LawInfo{
			{
				ID:         "001234",
				Name:       "개인정보 보호법",
				LawType:    "법률",
				Department: "개인정보보호위원회",
				EffectDate: "20200805",
			},
		},
	}

	f := NewFormatter("html-simple")
	result, err := f.FormatSearchResultToString(resp)

	if err != nil {
		t.Fatalf("FormatSearchResultToString() error = %v", err)
	}

	expectedContains := []string{
		"<h2>검색 결과</h2>",
		"<table>",
		"개인정보 보호법",
		"</table>",
	}

	for _, expected := range expectedContains {
		if !strings.Contains(result, expected) {
			t.Errorf("HTML Simple output does not contain expected string: %s", expected)
		}
	}

	// Should not contain full HTML structure
	if strings.Contains(result, "<!DOCTYPE html>") {
		t.Errorf("HTML Simple should not contain DOCTYPE")
	}
}
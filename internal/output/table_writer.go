package output

import (
	"bytes"
	"encoding/csv"
	"fmt"
	"os"
	"strings"

	"github.com/fatih/color"
	"github.com/olekukonko/tablewriter"
	"golang.org/x/term"
)

// TableStyle defines the style for table output
type TableStyle struct {
	UseColor      bool
	Compact       bool
	BoxDrawing    bool
	TerminalWidth int
}

// GetDefaultTableStyle returns the default table style
func GetDefaultTableStyle() *TableStyle {
	// Check if output is to a terminal
	useColor := isTerminal()

	// Get terminal width
	width := getTerminalWidth()

	return &TableStyle{
		UseColor:      useColor,
		Compact:       false,
		BoxDrawing:    true,
		TerminalWidth: width,
	}
}

// isTerminal checks if output is to a terminal
func isTerminal() bool {
	fileInfo, err := os.Stdout.Stat()
	if err != nil {
		return false
	}
	return (fileInfo.Mode() & os.ModeCharDevice) != 0
}

// getTerminalWidth returns the terminal width or a default value
func getTerminalWidth() int {
	if width, _, err := term.GetSize(int(os.Stdout.Fd())); err == nil {
		return width
	}
	return 120 // default width
}

// RenderTable renders a table with the given headers and rows
func RenderTable(headers []string, rows [][]string, style *TableStyle) string {
	if style == nil {
		style = GetDefaultTableStyle()
	}

	var buf bytes.Buffer
	table := tablewriter.NewWriter(&buf)

	// Set headers
	if style.UseColor {
		coloredHeaders := make([]string, len(headers))
		for i, h := range headers {
			coloredHeaders[i] = color.New(color.FgCyan, color.Bold).Sprint(h)
		}
		table.SetHeader(coloredHeaders)
	} else {
		table.SetHeader(headers)
	}

	// Configure table style
	if style.BoxDrawing {
		table.SetBorder(true)
		table.SetCenterSeparator("│")
		table.SetColumnSeparator("│")
		table.SetRowSeparator("─")
	} else {
		table.SetBorder(false)
		table.SetColumnSeparator(" ")
		table.SetRowLine(false)
		table.SetHeaderLine(true)
	}

	// Set alignment
	table.SetAlignment(tablewriter.ALIGN_LEFT)
	table.SetHeaderAlignment(tablewriter.ALIGN_LEFT)

	// Auto wrap and merge for long content
	table.SetAutoWrapText(true)
	table.SetAutoFormatHeaders(true)
	table.SetReflowDuringAutoWrap(true)

	// Set column widths based on header content
	// This helps maintain consistent column alignment
	if len(headers) >= 6 {
		// For standard law search table: 번호, 법령ID, 법령명, 법령구분, 소관부처, 시행일자
		// Or unified search table: 번호, 법령명, 구분, 출처, 소관부처, 시행일자
		columnWidths := []int{4, 8, 25, 10, 20, 11}

		// Adjust widths based on terminal width if available
		totalWidth := 0
		for _, w := range columnWidths {
			totalWidth += w
		}
		totalWidth += len(columnWidths) * 3 // account for separators

		if style.TerminalWidth > 0 && totalWidth > style.TerminalWidth {
			// Scale down proportionally if needed
			scale := float64(style.TerminalWidth-20) / float64(totalWidth)
			for i := range columnWidths {
				columnWidths[i] = int(float64(columnWidths[i]) * scale)
				if columnWidths[i] < 4 {
					columnWidths[i] = 4 // minimum width
				}
			}
		}

		// Apply minimum column widths to each column
		for i, width := range columnWidths {
			if i < len(headers) {
				table.SetColMinWidth(i, width)
			}
		}
	}

	// Add rows
	for _, row := range rows {
		table.Append(row)
	}

	// Render
	table.Render()

	return buf.String()
}

// RenderMarkdownTable renders a markdown table
func RenderMarkdownTable(headers []string, rows [][]string) string {
	var buf bytes.Buffer

	// Write headers
	fmt.Fprintf(&buf, "| %s |\n", strings.Join(headers, " | "))

	// Write separator
	separators := make([]string, len(headers))
	for i := range separators {
		separators[i] = "---"
	}
	fmt.Fprintf(&buf, "| %s |\n", strings.Join(separators, " | "))

	// Write rows
	for _, row := range rows {
		// Escape pipe characters in content
		escapedRow := make([]string, len(row))
		for i, cell := range row {
			escapedRow[i] = strings.ReplaceAll(cell, "|", "\\|")
		}
		fmt.Fprintf(&buf, "| %s |\n", strings.Join(escapedRow, " | "))
	}

	return buf.String()
}

// RenderCSV renders data as CSV
func RenderCSV(headers []string, rows [][]string, withBOM bool) (string, error) {
	var buf bytes.Buffer

	// Add BOM for Excel compatibility with Korean characters
	if withBOM {
		buf.Write([]byte{0xEF, 0xBB, 0xBF})
	}

	writer := csv.NewWriter(&buf)

	// Write headers
	if err := writer.Write(headers); err != nil {
		return "", fmt.Errorf("CSV 헤더 작성 실패: %w", err)
	}

	// Write rows
	for _, row := range rows {
		if err := writer.Write(row); err != nil {
			return "", fmt.Errorf("CSV 데이터 작성 실패: %w", err)
		}
	}

	writer.Flush()

	if err := writer.Error(); err != nil {
		return "", fmt.Errorf("CSV 작성 실패: %w", err)
	}

	return buf.String(), nil
}

// RenderHTMLTable renders an HTML table
func RenderHTMLTable(headers []string, rows [][]string) string {
	var buf bytes.Buffer

	// Start table with basic styling
	fmt.Fprintln(&buf, `<table style="border-collapse: collapse; width: 100%;">`)

	// Headers
	fmt.Fprintln(&buf, "  <thead>")
	fmt.Fprintln(&buf, "    <tr>")
	for _, header := range headers {
		fmt.Fprintf(&buf, `      <th style="border: 1px solid #ddd; padding: 8px; background-color: #f2f2f2; text-align: left;">%s</th>%s`,
			escapeHTML(header), "\n")
	}
	fmt.Fprintln(&buf, "    </tr>")
	fmt.Fprintln(&buf, "  </thead>")

	// Body
	fmt.Fprintln(&buf, "  <tbody>")
	for _, row := range rows {
		fmt.Fprintln(&buf, "    <tr>")
		for _, cell := range row {
			fmt.Fprintf(&buf, `      <td style="border: 1px solid #ddd; padding: 8px;">%s</td>%s`,
				escapeHTML(cell), "\n")
		}
		fmt.Fprintln(&buf, "    </tr>")
	}
	fmt.Fprintln(&buf, "  </tbody>")

	fmt.Fprintln(&buf, "</table>")

	return buf.String()
}

// RenderHTMLSimpleTable renders a simple HTML table without CSS styles
func RenderHTMLSimpleTable(headers []string, rows [][]string) string {
	var buf bytes.Buffer

	// Start table
	fmt.Fprintln(&buf, `<table>`)

	// Headers
	fmt.Fprintln(&buf, "  <thead>")
	fmt.Fprintln(&buf, "    <tr>")
	for _, header := range headers {
		fmt.Fprintf(&buf, "      <th>%s</th>\n", escapeHTML(header))
	}
	fmt.Fprintln(&buf, "    </tr>")
	fmt.Fprintln(&buf, "  </thead>")

	// Body
	fmt.Fprintln(&buf, "  <tbody>")
	for _, row := range rows {
		fmt.Fprintln(&buf, "    <tr>")
		for _, cell := range row {
			fmt.Fprintf(&buf, "      <td>%s</td>\n", escapeHTML(cell))
		}
		fmt.Fprintln(&buf, "    </tr>")
	}
	fmt.Fprintln(&buf, "  </tbody>")

	fmt.Fprintln(&buf, "</table>")

	return buf.String()
}

// escapeHTML escapes HTML special characters
func escapeHTML(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, "\"", "&quot;")
	s = strings.ReplaceAll(s, "'", "&#39;")
	return s
}

// HighlightValue applies color to important values if color is enabled
func HighlightValue(value string, style *TableStyle) string {
	if style == nil || !style.UseColor {
		return value
	}

	// Highlight dates
	if len(value) == 10 && value[4] == '-' && value[7] == '-' {
		return color.YellowString(value)
	}

	// Highlight law types
	switch value {
	case "법률":
		return color.GreenString(value)
	case "대통령령":
		return color.CyanString(value)
	case "부령", "총리령":
		return color.MagentaString(value)
	default:
		return value
	}
}

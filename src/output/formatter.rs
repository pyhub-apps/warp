use colored::*;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use serde_json;

use crate::api::types::{LawDetail, LawHistory, SearchResponse};
use crate::cli::OutputFormat;
use crate::error::Result;

/// Manual div_ceil implementation for MSRV 1.70.0 compatibility
/// Can be replaced with u32::div_ceil when MSRV is increased to 1.73.0+
#[inline]
#[allow(clippy::manual_div_ceil)]
fn div_ceil(dividend: u32, divisor: u32) -> u32 {
    if divisor == 0 {
        panic!("Division by zero");
    }
    (dividend + divisor - 1) / divisor
}

pub struct Formatter {
    format: OutputFormat,
}

impl Formatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Format search response
    pub fn format_search(&self, response: &SearchResponse) -> Result<String> {
        match self.format {
            OutputFormat::Table => self.format_search_table(response),
            OutputFormat::Json => self.format_search_json(response),
            OutputFormat::Markdown => self.format_search_markdown(response),
            OutputFormat::Csv => self.format_search_csv(response),
            OutputFormat::Html | OutputFormat::HtmlSimple => self.format_search_html(response),
        }
    }

    /// Format law detail
    pub fn format_detail(&self, detail: &LawDetail) -> Result<String> {
        match self.format {
            OutputFormat::Table => self.format_detail_table(detail),
            OutputFormat::Json => self.format_detail_json(detail),
            OutputFormat::Markdown => self.format_detail_markdown(detail),
            OutputFormat::Csv => self.format_detail_csv(detail),
            OutputFormat::Html | OutputFormat::HtmlSimple => self.format_detail_html(detail),
        }
    }

    /// Format law history
    pub fn format_history(&self, history: &LawHistory) -> Result<String> {
        match self.format {
            OutputFormat::Table => self.format_history_table(history),
            OutputFormat::Json => self.format_history_json(history),
            OutputFormat::Markdown => self.format_history_markdown(history),
            OutputFormat::Csv => self.format_history_csv(history),
            OutputFormat::Html | OutputFormat::HtmlSimple => self.format_history_html(history),
        }
    }

    // Table formatting methods
    fn format_search_table(&self, response: &SearchResponse) -> Result<String> {
        let mut table = Table::new();

        // Set up headers with color
        table.set_header(vec![
            Cell::new("번호").fg(Color::Cyan),
            Cell::new("법령명").fg(Color::Cyan),
            Cell::new("법령번호").fg(Color::Cyan),
            Cell::new("종류").fg(Color::Cyan),
            Cell::new("소관부처").fg(Color::Cyan),
            Cell::new("시행일").fg(Color::Cyan),
        ]);

        // Add rows
        for (idx, item) in response.items.iter().enumerate() {
            let row_num =
                ((response.page_no - 1) * response.page_size + idx as u32 + 1).to_string();

            table.add_row(vec![
                Cell::new(&row_num),
                Cell::new(truncate_string(&item.title, 40)),
                Cell::new(item.law_no.as_deref().unwrap_or("-")),
                Cell::new(item.law_type.as_deref().unwrap_or("-")),
                Cell::new(truncate_string(
                    item.department.as_deref().unwrap_or("-"),
                    20,
                )),
                Cell::new(item.enforcement_date.as_deref().unwrap_or("-")),
            ]);
        }

        // Set table properties
        table.set_content_arrangement(ContentArrangement::Dynamic);

        let mut result = String::new();

        // Add summary
        result.push_str(&format!(
            "\n{} Total: {} | Page: {}/{} | Results: {}\n\n",
            "📊".cyan(),
            response.total_count.to_string().yellow(),
            response.page_no.to_string().yellow(),
            div_ceil(response.total_count, response.page_size)
                .to_string()
                .yellow(),
            response.items.len().to_string().yellow()
        ));

        result.push_str(&table.to_string());

        Ok(result)
    }

    fn format_detail_table(&self, detail: &LawDetail) -> Result<String> {
        let mut result = String::new();

        // Title section
        result.push_str(&format!("\n{} {}\n", "📜".cyan(), detail.law_name.bold()));
        result.push_str(&"=".repeat(80));
        result.push('\n');

        // Basic info
        if let Some(law_no) = &detail.law_no {
            result.push_str(&format!("법령번호: {}\n", law_no));
        }
        if let Some(law_type) = &detail.law_type {
            result.push_str(&format!("법령종류: {}\n", law_type));
        }
        if let Some(department) = &detail.department {
            result.push_str(&format!("소관부처: {}\n", department));
        }
        if let Some(enforcement_date) = &detail.enforcement_date {
            result.push_str(&format!("시행일자: {}\n", enforcement_date));
        }
        if let Some(revision_date) = &detail.revision_date {
            result.push_str(&format!("개정일자: {}\n", revision_date));
        }

        result.push_str(&"-".repeat(80));
        result.push('\n');

        // Articles
        if !detail.articles.is_empty() {
            result.push_str(&format!(
                "\n{} 조문 ({}개)\n",
                "📋".cyan(),
                detail.articles.len()
            ));
            result.push_str(&"-".repeat(80));
            result.push('\n');

            for article in &detail.articles {
                result.push_str(&format!("\n{} ", article.number.bold()));
                if let Some(title) = &article.title {
                    result.push_str(&format!("({})", title));
                }
                result.push('\n');
                result.push_str(&article.content);
                result.push('\n');
            }
        }

        Ok(result)
    }

    fn format_history_table(&self, history: &LawHistory) -> Result<String> {
        let mut table = Table::new();

        table.set_header(vec![
            Cell::new("순번").fg(Color::Cyan),
            Cell::new("개정일자").fg(Color::Cyan),
            Cell::new("시행일자").fg(Color::Cyan),
            Cell::new("개정구분").fg(Color::Cyan),
            Cell::new("개정이유").fg(Color::Cyan),
        ]);

        for entry in &history.entries {
            table.add_row(vec![
                Cell::new(entry.revision_no.to_string()),
                Cell::new(entry.revision_date.clone()),
                Cell::new(entry.enforcement_date.as_deref().unwrap_or("-")),
                Cell::new(entry.revision_type.clone()),
                Cell::new(truncate_string(entry.reason.as_deref().unwrap_or("-"), 40)),
            ]);
        }

        let mut result = String::new();
        result.push_str(&format!(
            "\n{} {} 개정 연혁\n\n",
            "📚".cyan(),
            history.law_name.bold()
        ));
        result.push_str(&table.to_string());

        Ok(result)
    }

    // JSON formatting methods
    fn format_search_json(&self, response: &SearchResponse) -> Result<String> {
        serde_json::to_string_pretty(response)
            .map_err(crate::error::WarpError::Serialization)
    }

    fn format_detail_json(&self, detail: &LawDetail) -> Result<String> {
        serde_json::to_string_pretty(detail).map_err(crate::error::WarpError::Serialization)
    }

    fn format_history_json(&self, history: &LawHistory) -> Result<String> {
        serde_json::to_string_pretty(history).map_err(crate::error::WarpError::Serialization)
    }

    // Markdown formatting methods
    fn format_search_markdown(&self, response: &SearchResponse) -> Result<String> {
        let mut result = String::new();

        result.push_str(&format!("# 검색 결과\n\n"));
        result.push_str(&format!("- **총 결과**: {}\n", response.total_count));
        result.push_str(&format!(
            "- **페이지**: {}/{}\n",
            response.page_no,
            div_ceil(response.total_count, response.page_size)
        ));
        result.push_str(&format!("- **출처**: {}\n\n", response.source));

        result.push_str("| 번호 | 법령명 | 법령번호 | 종류 | 소관부처 | 시행일 |\n");
        result.push_str("|------|--------|----------|------|----------|--------|\n");

        for (idx, item) in response.items.iter().enumerate() {
            let row_num = (response.page_no - 1) * response.page_size + idx as u32 + 1;
            result.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                row_num,
                escape_markdown(&item.title),
                item.law_no.as_deref().unwrap_or("-"),
                item.law_type.as_deref().unwrap_or("-"),
                item.department.as_deref().unwrap_or("-"),
                item.enforcement_date.as_deref().unwrap_or("-"),
            ));
        }

        Ok(result)
    }

    fn format_detail_markdown(&self, detail: &LawDetail) -> Result<String> {
        let mut result = String::new();

        result.push_str(&format!("# {}\n\n", detail.law_name));

        if let Some(law_no) = &detail.law_no {
            result.push_str(&format!("- **법령번호**: {}\n", law_no));
        }
        if let Some(law_type) = &detail.law_type {
            result.push_str(&format!("- **법령종류**: {}\n", law_type));
        }
        if let Some(department) = &detail.department {
            result.push_str(&format!("- **소관부처**: {}\n", department));
        }
        if let Some(enforcement_date) = &detail.enforcement_date {
            result.push_str(&format!("- **시행일자**: {}\n", enforcement_date));
        }

        result.push_str("\n---\n\n");

        if !detail.articles.is_empty() {
            result.push_str("## 조문\n\n");
            for article in &detail.articles {
                result.push_str(&format!("### {}", article.number));
                if let Some(title) = &article.title {
                    result.push_str(&format!(" ({})", title));
                }
                result.push_str("\n\n");
                result.push_str(&article.content);
                result.push_str("\n\n");
            }
        }

        Ok(result)
    }

    // CSV formatting
    fn format_search_csv(&self, response: &SearchResponse) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write headers
        wtr.write_record(&["번호", "법령명", "법령번호", "종류", "소관부처", "시행일"])?;

        // Write data
        for (idx, item) in response.items.iter().enumerate() {
            let row_num =
                ((response.page_no - 1) * response.page_size + idx as u32 + 1).to_string();
            wtr.write_record(&[
                &row_num,
                &item.title,
                item.law_no.as_deref().unwrap_or(""),
                item.law_type.as_deref().unwrap_or(""),
                item.department.as_deref().unwrap_or(""),
                item.enforcement_date.as_deref().unwrap_or(""),
            ])?;
        }

        let data = wtr
            .into_inner()
            .map_err(|e| crate::error::WarpError::Other(e.to_string()))?;

        // Add BOM for Excel compatibility
        let mut result = vec![0xEF, 0xBB, 0xBF];
        result.extend_from_slice(&data);

        String::from_utf8(result).map_err(|e| crate::error::WarpError::Other(e.to_string()))
    }

    // HTML formatting
    fn format_search_html(&self, response: &SearchResponse) -> Result<String> {
        let mut html = String::new();

        let is_simple = matches!(self.format, OutputFormat::HtmlSimple);

        if !is_simple {
            html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
            html.push_str("<meta charset=\"UTF-8\">\n");
            html.push_str("<title>법령 검색 결과</title>\n");
            html.push_str("<style>\n");
            html.push_str("body { font-family: 'Malgun Gothic', sans-serif; margin: 20px; }\n");
            html.push_str("table { border-collapse: collapse; width: 100%; }\n");
            html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
            html.push_str("th { background-color: #4CAF50; color: white; }\n");
            html.push_str("tr:nth-child(even) { background-color: #f2f2f2; }\n");
            html.push_str("</style>\n</head>\n<body>\n");
        }

        html.push_str(&format!("<h1>검색 결과</h1>\n"));
        html.push_str(&format!(
            "<p>총 {}건 | 페이지 {}/{}</p>\n",
            response.total_count,
            response.page_no,
            div_ceil(response.total_count, response.page_size)
        ));

        html.push_str("<table>\n<thead>\n<tr>\n");
        html.push_str("<th>번호</th><th>법령명</th><th>법령번호</th><th>종류</th><th>소관부처</th><th>시행일</th>\n");
        html.push_str("</tr>\n</thead>\n<tbody>\n");

        for (idx, item) in response.items.iter().enumerate() {
            let row_num = (response.page_no - 1) * response.page_size + idx as u32 + 1;
            html.push_str("<tr>\n");
            html.push_str(&format!("<td>{}</td>", row_num));
            html.push_str(&format!("<td>{}</td>", escape_html(&item.title)));
            html.push_str(&format!(
                "<td>{}</td>",
                item.law_no.as_deref().unwrap_or("-")
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                item.law_type.as_deref().unwrap_or("-")
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                item.department.as_deref().unwrap_or("-")
            ));
            html.push_str(&format!(
                "<td>{}</td>",
                item.enforcement_date.as_deref().unwrap_or("-")
            ));
            html.push_str("</tr>\n");
        }

        html.push_str("</tbody>\n</table>\n");

        if !is_simple {
            html.push_str("</body>\n</html>");
        }

        Ok(html)
    }

    fn format_detail_csv(&self, detail: &LawDetail) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write basic info as key-value pairs
        wtr.write_record(&["항목", "내용"])?;
        wtr.write_record(&["법령명", &detail.law_name])?;

        if let Some(law_no) = &detail.law_no {
            wtr.write_record(&["법령번호", law_no])?;
        }
        if let Some(law_type) = &detail.law_type {
            wtr.write_record(&["법령종류", law_type])?;
        }
        if let Some(department) = &detail.department {
            wtr.write_record(&["소관부처", department])?;
        }
        if let Some(enforcement_date) = &detail.enforcement_date {
            wtr.write_record(&["시행일자", enforcement_date])?;
        }
        if let Some(revision_date) = &detail.revision_date {
            wtr.write_record(&["개정일자", revision_date])?;
        }

        // Add articles if present
        if !detail.articles.is_empty() {
            wtr.write_record(&["", ""])?; // Empty row
            wtr.write_record(&["조문번호", "조문내용"])?;
            for article in &detail.articles {
                let title = article.title.as_deref().unwrap_or("");
                let header = if title.is_empty() {
                    article.number.clone()
                } else {
                    format!("{} ({})", article.number, title)
                };
                wtr.write_record(&[&header, &article.content])?;
            }
        }

        let data = wtr
            .into_inner()
            .map_err(|e| crate::error::WarpError::Other(e.to_string()))?;

        // Add BOM for Excel compatibility
        let mut result = vec![0xEF, 0xBB, 0xBF];
        result.extend_from_slice(&data);

        String::from_utf8(result).map_err(|e| crate::error::WarpError::Other(e.to_string()))
    }

    fn format_detail_html(&self, detail: &LawDetail) -> Result<String> {
        let mut html = String::new();
        let is_simple = matches!(self.format, OutputFormat::HtmlSimple);

        if !is_simple {
            html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
            html.push_str("<meta charset=\"UTF-8\">\n");
            html.push_str(&format!(
                "<title>{}</title>\n",
                escape_html(&detail.law_name)
            ));
            html.push_str("<style>\n");
            html.push_str("body { font-family: 'Malgun Gothic', sans-serif; margin: 20px; line-height: 1.6; }\n");
            html.push_str(".info { background: #f9f9f9; padding: 15px; border-radius: 5px; margin-bottom: 20px; }\n");
            html.push_str(
                ".article { margin: 20px 0; padding: 15px; border-left: 3px solid #4CAF50; }\n",
            );
            html.push_str(
                ".article-title { font-weight: bold; color: #333; margin-bottom: 10px; }\n",
            );
            html.push_str("</style>\n</head>\n<body>\n");
        }

        html.push_str(&format!("<h1>{}</h1>\n", escape_html(&detail.law_name)));

        html.push_str("<div class=\"info\">\n");
        if let Some(law_no) = &detail.law_no {
            html.push_str(&format!(
                "<p><strong>법령번호:</strong> {}</p>\n",
                escape_html(law_no)
            ));
        }
        if let Some(law_type) = &detail.law_type {
            html.push_str(&format!(
                "<p><strong>법령종류:</strong> {}</p>\n",
                escape_html(law_type)
            ));
        }
        if let Some(department) = &detail.department {
            html.push_str(&format!(
                "<p><strong>소관부처:</strong> {}</p>\n",
                escape_html(department)
            ));
        }
        if let Some(enforcement_date) = &detail.enforcement_date {
            html.push_str(&format!(
                "<p><strong>시행일자:</strong> {}</p>\n",
                escape_html(enforcement_date)
            ));
        }
        if let Some(revision_date) = &detail.revision_date {
            html.push_str(&format!(
                "<p><strong>개정일자:</strong> {}</p>\n",
                escape_html(revision_date)
            ));
        }
        html.push_str("</div>\n");

        if !detail.articles.is_empty() {
            html.push_str("<h2>조문</h2>\n");
            for article in &detail.articles {
                html.push_str("<div class=\"article\">\n");
                html.push_str(&format!(
                    "<div class=\"article-title\">{}",
                    escape_html(&article.number)
                ));
                if let Some(title) = &article.title {
                    html.push_str(&format!(" ({})", escape_html(title)));
                }
                html.push_str("</div>\n");
                html.push_str(&format!(
                    "<div>{}</div>\n",
                    escape_html(&article.content).replace("\n", "<br>")
                ));
                html.push_str("</div>\n");
            }
        }

        if !is_simple {
            html.push_str("</body>\n</html>");
        }

        Ok(html)
    }

    fn format_history_markdown(&self, history: &LawHistory) -> Result<String> {
        let mut result = String::new();

        result.push_str(&format!("# {} 개정 연혁\n\n", history.law_name));
        result.push_str(&format!("총 {}건의 개정 이력\n\n", history.total_count));

        result.push_str("| 순번 | 개정일자 | 시행일자 | 개정구분 | 개정이유 |\n");
        result.push_str("|------|----------|----------|----------|----------|\n");

        for entry in &history.entries {
            result.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                entry.revision_no,
                entry.revision_date,
                entry.enforcement_date.as_deref().unwrap_or("-"),
                entry.revision_type,
                escape_markdown(entry.reason.as_deref().unwrap_or("-")),
            ));
        }

        Ok(result)
    }

    fn format_history_csv(&self, history: &LawHistory) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write headers
        wtr.write_record(&["순번", "개정일자", "시행일자", "개정구분", "개정이유"])?;

        // Write data
        for entry in &history.entries {
            wtr.write_record(&[
                &entry.revision_no.to_string(),
                &entry.revision_date,
                entry.enforcement_date.as_deref().unwrap_or(""),
                &entry.revision_type,
                entry.reason.as_deref().unwrap_or(""),
            ])?;
        }

        let data = wtr
            .into_inner()
            .map_err(|e| crate::error::WarpError::Other(e.to_string()))?;

        // Add BOM for Excel compatibility
        let mut result = vec![0xEF, 0xBB, 0xBF];
        result.extend_from_slice(&data);

        String::from_utf8(result).map_err(|e| crate::error::WarpError::Other(e.to_string()))
    }

    fn format_history_html(&self, history: &LawHistory) -> Result<String> {
        let mut html = String::new();
        let is_simple = matches!(self.format, OutputFormat::HtmlSimple);

        if !is_simple {
            html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
            html.push_str("<meta charset=\"UTF-8\">\n");
            html.push_str(&format!(
                "<title>{} 개정 연혁</title>\n",
                escape_html(&history.law_name)
            ));
            html.push_str("<style>\n");
            html.push_str("body { font-family: 'Malgun Gothic', sans-serif; margin: 20px; }\n");
            html.push_str("table { border-collapse: collapse; width: 100%; }\n");
            html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
            html.push_str("th { background-color: #4CAF50; color: white; }\n");
            html.push_str("tr:nth-child(even) { background-color: #f2f2f2; }\n");
            html.push_str("</style>\n</head>\n<body>\n");
        }

        html.push_str(&format!(
            "<h1>{} 개정 연혁</h1>\n",
            escape_html(&history.law_name)
        ));
        html.push_str(&format!(
            "<p>총 {}건의 개정 이력</p>\n",
            history.total_count
        ));

        html.push_str("<table>\n<thead>\n<tr>\n");
        html.push_str(
            "<th>순번</th><th>개정일자</th><th>시행일자</th><th>개정구분</th><th>개정이유</th>\n",
        );
        html.push_str("</tr>\n</thead>\n<tbody>\n");

        for entry in &history.entries {
            html.push_str("<tr>\n");
            html.push_str(&format!("<td>{}</td>", entry.revision_no));
            html.push_str(&format!("<td>{}</td>", escape_html(&entry.revision_date)));
            html.push_str(&format!(
                "<td>{}</td>",
                escape_html(entry.enforcement_date.as_deref().unwrap_or("-"))
            ));
            html.push_str(&format!("<td>{}</td>", escape_html(&entry.revision_type)));
            html.push_str(&format!(
                "<td>{}</td>",
                escape_html(entry.reason.as_deref().unwrap_or("-"))
            ));
            html.push_str("</tr>\n");
        }

        html.push_str("</tbody>\n</table>\n");

        if !is_simple {
            html.push_str("</body>\n</html>");
        }

        Ok(html)
    }
}

// Helper functions
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

fn escape_markdown(s: &str) -> String {
    s.replace("|", "\\|")
        .replace("*", "\\*")
        .replace("_", "\\_")
}

fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

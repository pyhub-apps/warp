pub mod formatter;

pub use formatter::Formatter;

use crate::api::types::{SearchResponse, LawDetail, LawHistory};
use crate::cli::OutputFormat;
use crate::error::Result;

/// Format search response based on the specified format
pub fn format_search_response(response: &SearchResponse, format: OutputFormat) -> Result<String> {
    let formatter = Formatter::new(format);
    formatter.format_search(response)
}

/// Format law detail based on the specified format
pub fn format_law_detail(detail: &LawDetail, format: OutputFormat) -> Result<String> {
    let formatter = Formatter::new(format);
    formatter.format_detail(detail)
}

/// Format law history based on the specified format
pub fn format_law_history(history: &LawHistory, format: OutputFormat) -> Result<String> {
    let formatter = Formatter::new(format);
    formatter.format_history(history)
}
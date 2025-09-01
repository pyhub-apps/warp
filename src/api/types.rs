use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unified search request for all API types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchRequest {
    /// Search query
    pub query: String,
    /// Page number (1-based)
    pub page_no: u32,
    /// Results per page
    pub page_size: u32,
    /// Response type (JSON/XML)
    pub response_type: ResponseType,
    /// Region filter (for ELIS)
    pub region: Option<String>,
    /// Law type filter
    pub law_type: Option<String>,
    /// Department filter
    pub department: Option<String>,
    /// Date range start (YYYYMMDD)
    pub date_from: Option<String>,
    /// Date range end (YYYYMMDD)
    pub date_to: Option<String>,
    /// Sort order
    pub sort: Option<SortOrder>,
    /// API-specific extra parameters
    pub extras: HashMap<String, String>,
}

impl Default for UnifiedSearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            page_no: 1,
            page_size: 50,
            response_type: ResponseType::Json,
            region: None,
            law_type: None,
            department: None,
            date_from: None,
            date_to: None,
            sort: None,
            extras: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResponseType {
    Json,
    Xml,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortOrder {
    /// Relevance (default)
    Relevance,
    /// Date ascending
    DateAsc,
    /// Date descending
    DateDesc,
    /// Title ascending
    TitleAsc,
    /// Title descending
    TitleDesc,
}

/// Unified search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Total number of results
    pub total_count: u32,
    /// Current page number
    pub page_no: u32,
    /// Results per page
    pub page_size: u32,
    /// List of search results
    pub items: Vec<SearchItem>,
    /// Source API
    pub source: String,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// Individual search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    /// Unique identifier
    pub id: String,
    /// Law/document title
    pub title: String,
    /// Law number or document number
    pub law_no: Option<String>,
    /// Type of law/document
    pub law_type: Option<String>,
    /// Department or organization
    pub department: Option<String>,
    /// Enforcement date
    pub enforcement_date: Option<String>,
    /// Revision date
    pub revision_date: Option<String>,
    /// Summary or excerpt
    pub summary: Option<String>,
    /// Source API
    pub source: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Law detail information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawDetail {
    /// Law ID
    pub law_id: String,
    /// Law title
    pub law_name: String,
    /// Law number
    pub law_no: Option<String>,
    /// Law type
    pub law_type: Option<String>,
    /// Department
    pub department: Option<String>,
    /// Enforcement date
    pub enforcement_date: Option<String>,
    /// Revision date
    pub revision_date: Option<String>,
    /// Full content
    pub content: String,
    /// Articles
    pub articles: Vec<Article>,
    /// Attachments
    pub attachments: Vec<Attachment>,
    /// Related laws
    pub related_laws: Vec<RelatedLaw>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Law article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    /// Article number
    pub number: String,
    /// Article title
    pub title: Option<String>,
    /// Article content
    pub content: String,
    /// Paragraphs
    pub paragraphs: Vec<Paragraph>,
}

/// Article paragraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    /// Paragraph number
    pub number: String,
    /// Paragraph content
    pub content: String,
    /// Sub-items
    pub items: Vec<String>,
}

/// Attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment ID
    pub id: String,
    /// File name
    pub name: String,
    /// File type
    pub file_type: String,
    /// File size in bytes
    pub size: Option<u64>,
    /// Download URL
    pub url: Option<String>,
}

/// Related law
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedLaw {
    /// Law ID
    pub id: String,
    /// Law title
    pub title: String,
    /// Law number
    pub law_no: Option<String>,
    /// Relationship type
    pub relation_type: String,
}

/// Law history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawHistory {
    /// Law ID
    pub law_id: String,
    /// Law title
    pub law_name: String,
    /// Total count
    pub total_count: u32,
    /// History entries
    pub entries: Vec<HistoryEntry>,
}

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Revision number
    pub revision_no: u32,
    /// Revision date
    pub revision_date: String,
    /// Enforcement date
    pub enforcement_date: Option<String>,
    /// Revision type
    pub revision_type: String,
    /// Revision reason
    pub reason: Option<String>,
    /// Changed articles
    pub changed_articles: Vec<String>,
}
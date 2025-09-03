//! # Korean Legal API Module
//!
//! This module provides unified access to multiple Korean government legal databases
//! through a consistent interface. It supports searching, retrieving, and caching
//! legal documents from various sources.
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │   CLI Layer     │ -> │  Unified Client  │ -> │  Cache Layer    │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!                                 |
//!                        ┌────────┼────────┐
//!                        │        │        │
//!                   ┌────▼──┐ ┌──▼───┐ ┌──▼────┐
//!                   │ NLIC  │ │ ELIS │ │ PREC  │
//!                   └───────┘ └──────┘ └───────┘
//! ```
//!
//! ## Usage Examples
//!
//! ### Basic Law Search
//!
//! ```no_run
//! use warp::api::{ApiClientFactory, ApiType};
//! use warp::api::types::UnifiedSearchRequest;
//! use warp::config::Config;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::load()?;
//! let client = ApiClientFactory::create(ApiType::Nlic, config.to_client_config())?;
//!
//! let request = UnifiedSearchRequest {
//!     query: "민법".to_string(),
//!     page: Some(1),
//!     size: Some(20),
//!     ..Default::default()
//! };
//!
//! let response = client.search(request).await?;
//! for law in response.laws.unwrap_or_default() {
//!     println!("{}: {}", law.law_num, law.law_name);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Unified Multi-API Search
//!
//! ```no_run
//! use warp::api::{ApiClientFactory, ApiType};
//! use warp::api::types::UnifiedSearchRequest;
//! use warp::config::Config;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::load()?;
//! let client = ApiClientFactory::create(ApiType::All, config.to_client_config())?;
//!
//! let request = UnifiedSearchRequest {
//!     query: "환경보호".to_string(),
//!     ..Default::default()
//! };
//!
//! let response = client.search(request).await?;
//! println!("Total results across all APIs: {}", response.total_count.unwrap_or(0));
//! # Ok(())
//! # }
//! ```
//!
//! ### Detailed Document Retrieval
//!
//! ```no_run
//! use warp::api::{ApiClientFactory, ApiType};
//! use warp::config::Config;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::load()?;
//! let client = ApiClientFactory::create(ApiType::Nlic, config.to_client_config())?;
//!
//! let detail = client.get_detail("000001").await?;
//! println!("Law: {}", detail.law_name);
//! println!("Content: {}", detail.law_content.unwrap_or_default());
//! # Ok(())
//! # }
//! ```

pub mod admrul;
pub mod batcher;
pub mod client;
pub mod deserializers;
pub mod elis;
pub mod expc;
pub mod http_client;
pub mod nlic;
pub mod parallel;
pub mod pool;
pub mod prec;
pub mod streaming;
pub mod types;

use std::str::FromStr;

pub use client::{ApiClientFactory, LegalApiClient};

/// Enumeration of supported Korean legal API types
///
/// Each variant represents a different Korean government legal database
/// with specific characteristics and use cases.
///
/// # Examples
///
/// ```
/// use warp::api::ApiType;
/// use std::str::FromStr;
///
/// // Parse from string
/// let api_type = ApiType::from_str("nlic").unwrap();
/// assert_eq!(api_type, ApiType::Nlic);
///
/// // Display names in Korean
/// assert_eq!(api_type.display_name(), "국가법령정보센터");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiType {
    /// **National Law Information Center** (국가법령정보센터)
    ///
    /// Primary source for Korean national laws, statutes, and regulations.
    /// Contains the most comprehensive collection of Korean legal documents.
    ///
    /// **Data Types**: Laws, statutes, presidential decrees, ministerial ordinances
    /// **Update Frequency**: Real-time
    /// **Coverage**: National level legislation
    Nlic,

    /// **Local Regulations Information System** (자치법규정보시스템)
    ///
    /// Database for local government ordinances and regulations at city,
    /// province, and district levels across Korea.
    ///
    /// **Data Types**: Local ordinances, municipal regulations
    /// **Update Frequency**: Daily
    /// **Coverage**: Regional and local government legislation
    Elis,

    /// **Court Precedents Database** (판례)
    ///
    /// Collection of Korean court decisions and legal precedents from
    /// various court levels including Supreme Court decisions.
    ///
    /// **Data Types**: Court decisions, legal precedents, case summaries
    /// **Update Frequency**: Weekly
    /// **Coverage**: Judicial decisions and interpretations
    Prec,

    /// **Administrative Rules Database** (행정규칙)
    ///
    /// Repository for administrative rules, guidelines, and internal
    /// regulations issued by government ministries and agencies.
    ///
    /// **Data Types**: Administrative rules, ministry guidelines, agency directives
    /// **Update Frequency**: As published
    /// **Coverage**: Government administrative procedures
    Admrul,

    /// **Legal Interpretation Cases** (법령해석례)
    ///
    /// Official interpretations of laws and regulations provided by
    /// government agencies and legal committees.
    ///
    /// **Data Types**: Legal interpretations, regulatory clarifications
    /// **Update Frequency**: As issued
    /// **Coverage**: Official legal interpretations and guidance
    Expc,

    /// **Unified Multi-API Search**
    ///
    /// Special type that enables searching across all supported APIs
    /// simultaneously, providing comprehensive coverage of Korean legal information.
    ///
    /// **Benefits**: Complete coverage, parallel execution, unified results
    /// **Performance**: Optimized with concurrent requests and result merging
    All,
}

impl FromStr for ApiType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nlic" | "law" => Ok(Self::Nlic),
            "elis" | "ordinance" => Ok(Self::Elis),
            "prec" | "precedent" => Ok(Self::Prec),
            "admrul" | "administrative" => Ok(Self::Admrul),
            "expc" | "interpretation" => Ok(Self::Expc),
            "all" | "unified" => Ok(Self::All),
            _ => Err(format!("Unknown API type: {}", s)),
        }
    }
}

impl ApiType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nlic => "nlic",
            Self::Elis => "elis",
            Self::Prec => "prec",
            Self::Admrul => "admrul",
            Self::Expc => "expc",
            Self::All => "all",
        }
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Nlic => "국가법령정보센터",
            Self::Elis => "자치법규정보시스템",
            Self::Prec => "판례",
            Self::Admrul => "행정규칙",
            Self::Expc => "법령해석례",
            Self::All => "통합검색",
        }
    }
}

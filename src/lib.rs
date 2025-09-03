//! # Warp - Korean Legal Information CLI
//!
//! A powerful command-line interface for searching Korean legal documents.
//! Provides unified access to multiple Korean government legal databases
//! including laws, ordinances, precedents, and administrative rules.
//!
//! ## Supported APIs
//!
//! - **NLIC** (국가법령정보센터): National Law Information Center
//! - **ELIS** (자치법규정보시스템): Local Regulations Information System
//! - **PREC** (판례): Court Precedents Database
//! - **ADMRUL** (행정규칙): Administrative Rules Database
//! - **EXPC** (법령해석례): Legal Interpretation Cases
//!
//! ## Quick Start
//!
//! ```no_run
//! use warp::api::{ApiClientFactory, ApiType};
//! use warp::api::types::UnifiedSearchRequest;
//! use warp::config::Config;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = Config::load()?;
//!
//!     // Create API client
//!     let client = ApiClientFactory::create(
//!         ApiType::Nlic,
//!         config.to_client_config()
//!     )?;
//!
//!     // Search for laws
//!     let request = UnifiedSearchRequest {
//!         query: "민법".to_string(),
//!         ..Default::default()
//!     };
//!
//!     let response = client.search(request).await?;
//!     println!("Found {} results", response.total_count.unwrap_or(0));
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Multi-API Support**: Search across multiple Korean legal databases
//! - **Intelligent Caching**: Tiered caching system for improved performance
//! - **Internationalization**: Support for Korean and English interfaces
//! - **Progress Tracking**: Real-time progress indicators for long operations
//! - **Metrics Collection**: Built-in performance and usage metrics
//! - **Flexible Output**: JSON, YAML, CSV, and table formats

pub mod api;
pub mod cache;
pub mod cli;
pub mod config;
pub mod error;
pub mod metrics;
pub mod output;
pub mod progress;

// Initialize i18n system
rust_i18n::i18n!("locales", fallback = "en");

#[cfg(test)]
mod error_test;

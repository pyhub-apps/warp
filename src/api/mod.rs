pub mod admrul;
pub mod client;
pub mod deserializers;
pub mod elis;
pub mod expc;
pub mod http_client;
pub mod nlic;
pub mod parallel;
pub mod prec;
pub mod streaming;
pub mod types;

use std::str::FromStr;

pub use client::{ApiClientFactory, LegalApiClient};

/// API types supported by the CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiType {
    /// National Law Information Center (국가법령정보센터)
    Nlic,
    /// Local Regulations Information System (자치법규정보시스템)
    Elis,
    /// Precedent API (판례)
    Prec,
    /// Administrative Rule API (행정규칙)
    Admrul,
    /// Legal Interpretation API (법령해석례)
    Expc,
    /// Unified search across multiple APIs
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

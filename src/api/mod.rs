pub mod client;
pub mod nlic;
pub mod elis;
pub mod prec;
pub mod admrul;
pub mod expc;
pub mod types;
pub mod deserializers;

pub use client::{LegalApiClient, ApiClientFactory};

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

impl ApiType {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "nlic" | "law" => Some(Self::Nlic),
            "elis" | "ordinance" => Some(Self::Elis),
            "prec" | "precedent" => Some(Self::Prec),
            "admrul" | "administrative" => Some(Self::Admrul),
            "expc" | "interpretation" => Some(Self::Expc),
            "all" | "unified" => Some(Self::All),
            _ => None,
        }
    }

    #[allow(dead_code)]
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
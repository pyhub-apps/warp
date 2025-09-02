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

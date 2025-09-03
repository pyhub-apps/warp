//! # Korean Legal API Usage Examples
//!
//! This example demonstrates how to use the various Korean legal API clients
//! provided by the warp library to search for legal documents, retrieve
//! detailed information, and work with different data sources.

use warp::api::{ApiClientFactory, ApiType};
use warp::api::types::{UnifiedSearchRequest, SortOrder};
use warp::api::client::ClientConfig;
use warp::config::Config;
use warp::error::{Result, WarpError};

/// Example: Basic law search using NLIC API
///
/// Demonstrates how to perform a basic search for Korean laws
/// using the National Law Information Center API.
async fn basic_nlic_search() -> Result<()> {
    println!("=== Basic NLIC Search Example ===");

    let config = Config::load()?;

    let api_key = match config.get_nlic_api_key() {
        Some(key) => key,
        None => {
            println!("⚠ No NLIC API key configured. Please set up your configuration first.");
            return Ok(());
        }
    };

    let client_config = ClientConfig {
        api_key,
        ..Default::default()
    };

    let client = ApiClientFactory::create(ApiType::Nlic, client_config)?;

    let request = UnifiedSearchRequest {
        query: "민법".to_string(),
        page_no: 1,
        page_size: 5,
        ..Default::default()
    };

    let response = client.search(request).await?;

    println!("Found {} total results", response.total_count);

    if !response.items.is_empty() {
        for (i, item) in response.items.iter().enumerate() {
            println!("{}. {} ({})", i + 1, item.title, item.law_no.as_deref().unwrap_or("N/A"));
            if let Some(dept) = &item.department {
                println!("   Department: {}", dept);
            }
        }
    } else {
        println!("No results found");
    }

    Ok(())
}

/// Example: Advanced search with filters
async fn advanced_search_with_filters() -> Result<()> {
    println!("\n=== Advanced Search with Filters ===");

    let config = Config::load()?;

    let api_key = match config.get_nlic_api_key() {
        Some(key) => key,
        None => {
            println!("⚠ No NLIC API key configured. Skipping advanced search example.");
            return Ok(());
        }
    };

    let client_config = ClientConfig {
        api_key,
        ..Default::default()
    };

    let client = ApiClientFactory::create(ApiType::Nlic, client_config)?;

    let request = UnifiedSearchRequest {
        query: "환경보호".to_string(),
        page_no: 1,
        page_size: 10,
        sort: Some(SortOrder::DateDesc),
        law_type: Some("법률".to_string()),
        ..Default::default()
    };

    let response = client.search(request).await?;

    println!("Environmental protection laws found: {}", response.total_count);

    for item in response.items.iter().take(3) {
        println!("- {}: {}", item.law_no.as_deref().unwrap_or("N/A"), item.title);
        if let Some(law_type) = &item.law_type {
            println!("  Type: {}", law_type);
        }
    }

    Ok(())
}

/// Example: Multi-API search comparison
async fn multi_api_search() -> Result<()> {
    println!("\n=== Multi-API Search Comparison ===");

    let config = Config::load()?;
    let query = "교통안전";

    let apis = vec![
        (ApiType::Nlic, "National Laws", config.get_nlic_api_key()),
        (ApiType::Elis, "Local Regulations", config.get_elis_api_key()),
        (ApiType::Prec, "Court Precedents", config.get_prec_api_key()),
    ];

    for (api_type, description, api_key) in apis {
        println!("\n--- {} ({:?}) ---", description, api_type);

        let api_key = match api_key {
            Some(key) => key,
            None => {
                println!("⚠ No API key configured for {:?}", api_type);
                continue;
            }
        };

        let client_config = ClientConfig {
            api_key,
            ..Default::default()
        };

        match ApiClientFactory::create(api_type, client_config) {
            Ok(client) => {
                let request = UnifiedSearchRequest {
                    query: query.to_string(),
                    page_no: 1,
                    page_size: 3,
                    ..Default::default()
                };

                match client.search(request).await {
                    Ok(response) => {
                        println!("Results: {}", response.total_count);

                        for item in response.items.iter().take(2) {
                            println!("  • {}", item.title);
                        }
                    }
                    Err(e) => println!("Search failed: {}", e),
                }
            }
            Err(e) => println!("Client creation failed: {}", e),
        }
    }

    Ok(())
}

/// Example: Detailed document retrieval
async fn detailed_document_retrieval() -> Result<()> {
    println!("\n=== Detailed Document Retrieval ===");

    let config = Config::load()?;

    let api_key = match config.get_nlic_api_key() {
        Some(key) => key,
        None => {
            println!("⚠ No NLIC API key configured. Skipping detailed retrieval example.");
            return Ok(());
        }
    };

    let client_config = ClientConfig {
        api_key,
        ..Default::default()
    };

    let client = ApiClientFactory::create(ApiType::Nlic, client_config)?;

    let search_request = UnifiedSearchRequest {
        query: "민법".to_string(),
        page_no: 1,
        page_size: 1,
        ..Default::default()
    };

    let search_response = client.search(search_request).await?;

    if !search_response.items.is_empty() {
        let first_item = &search_response.items[0];
        println!("Selected document: {}", first_item.title);

        // Get detailed information
        match client.get_detail(&first_item.id).await {
            Ok(detail) => {
                println!("Document ID: {}", detail.law_id);
                println!("Document Name: {}", detail.law_name);
                let preview = if detail.content.len() > 200 {
                    &detail.content[..200]
                } else {
                    &detail.content
                };
                println!("Content Preview: {}...", preview);
            }
            Err(e) => println!("Failed to get details: {}", e),
        }

        // Get revision history
        match client.get_history(&first_item.id).await {
            Ok(history) => {
                println!("Revision History:");
                for entry in history.entries.iter().take(3) {
                    println!("  - {}: {}",
                        entry.revision_date,
                        entry.reason.as_deref().unwrap_or("No reason provided")
                    );
                }
            }
            Err(e) => println!("Failed to get history: {}", e),
        }
    } else {
        println!("No documents found for detailed retrieval");
    }

    Ok(())
}

/// Example: Performance optimization with caching
async fn performance_optimization_example() -> Result<()> {
    println!("\n=== Performance Optimization with Caching ===");

    let config = Config::load()?;

    let api_key = match config.get_nlic_api_key() {
        Some(key) => key,
        None => {
            println!("⚠ No NLIC API key configured. Skipping performance example.");
            return Ok(());
        }
    };

    let client_config = ClientConfig {
        api_key,
        ..Default::default()
    };

    let client = ApiClientFactory::create(ApiType::Nlic, client_config)?;

    let request = UnifiedSearchRequest {
        query: "계약".to_string(),
        page_no: 1,
        page_size: 5,
        ..Default::default()
    };

    println!("Making first request...");
    let start_time = std::time::Instant::now();
    let _response1 = client.search(request.clone()).await?;
    let first_duration = start_time.elapsed();

    println!("First request took: {:?}", first_duration);

    println!("Making second identical request...");
    let start_time = std::time::Instant::now();
    let _response2 = client.search(request).await?;
    let second_duration = start_time.elapsed();

    println!("Second request took: {:?}", second_duration);

    if second_duration < first_duration {
        println!("✓ Performance improvement detected (likely from caching)!");
    } else {
        println!("ℹ No significant performance difference observed");
    }

    Ok(())
}

/// Main function to run all examples
#[tokio::main]
async fn main() -> Result<()> {
    println!("Korean Legal API Usage Examples");
    println!("===============================");

    // Initialize logging
    env_logger::init();

    // Run examples and handle errors gracefully
    let examples: Vec<(&str, fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>)> = vec![
        ("Basic Search", || Box::pin(basic_nlic_search())),
        ("Advanced Search", || Box::pin(advanced_search_with_filters())),
        ("Multi-API Search", || Box::pin(multi_api_search())),
        ("Detailed Retrieval", || Box::pin(detailed_document_retrieval())),
        ("Performance Optimization", || Box::pin(performance_optimization_example())),
    ];

    for (name, example_fn) in examples {
        match example_fn().await {
            Ok(_) => println!("✓ {} example completed successfully", name),
            Err(WarpError::NoApiKey) => println!("⚠ {} example skipped (no API key)", name),
            Err(e) => eprintln!("✗ {} example failed: {}", name, e),
        }
    }

    println!("\n=== Examples completed ===");
    println!("For more information, see the API documentation:");
    println!("cargo doc --open");

    Ok(())
}
mod api;
mod cli;
mod config;
mod error;
mod output;

use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::Cli::run().await
}

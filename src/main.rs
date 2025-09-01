use warp::cli;
use warp::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::Cli::run().await
}

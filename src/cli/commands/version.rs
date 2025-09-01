/// Execute version command
pub fn execute() {
    println!("warp {}", env!("CARGO_PKG_VERSION"));
    println!("Korean Legal Information CLI");
    println!();
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    println!("License: {}", env!("CARGO_PKG_LICENSE"));
}

use rust_i18n::t;

/// Execute version command
pub fn execute() {
    println!("warp {}", env!("CARGO_PKG_VERSION"));
    println!("{}", t!("about"));
    println!();
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    println!("License: {}", env!("CARGO_PKG_LICENSE"));
}

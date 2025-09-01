use crate::cli::args::{ConfigArgs, ConfigCommand};
use crate::config::Config;
use crate::error::Result;

/// Execute config command
pub async fn execute(args: ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommand::Set { key, value } => {
            let mut config = Config::load()?;
            config.set(&key, &value)?;
            println!("✅ Configuration updated: {} = {}", key, mask_value(&value));
            Ok(())
        }
        ConfigCommand::Get { key } => {
            let config = Config::load()?;
            match config.get(&key) {
                Some(value) => {
                    println!("{}: {}", key, mask_value(&value));
                }
                None => {
                    println!("Configuration key '{}' not found", key);
                }
            }
            Ok(())
        }
        ConfigCommand::Path => {
            let path = Config::config_file_path()?;
            println!("Configuration file: {}", path.display());
            Ok(())
        }
        ConfigCommand::Init => {
            Config::initialize()?;
            println!("✅ Configuration initialized");
            println!();
            println!("To set your API key, run:");
            println!("  warp config set law.key YOUR_API_KEY");
            println!();
            println!("Get your API key from: https://open.law.go.kr");
            Ok(())
        }
    }
}

/// Mask sensitive values for display
fn mask_value(value: &str) -> String {
    if value.len() > 10 {
        format!("{}...({} characters)", &value[..10], value.len())
    } else {
        value.to_string()
    }
}

use anyhow::{Context, Result};
use directories::BaseDirs;
use std::path::PathBuf;

pub struct AppConfigs {
    pub model: String,
}

impl AppConfigs {
    pub fn new() -> Result<Self> {
        let config_file = Self::get_config_path()?;

        // Create default config if doesn't exist
        if !config_file.exists() {
            Self::create_default_config()?;
        }

        let config_content = std::fs::read_to_string(&config_file)
            .with_context(|| format!("Failed to read config at {:?}", config_file))?;

        let config: toml::Value =
            toml::from_str(&config_content).context("Failed to parse config file")?;

        Ok(Self {
            model: config["GEMINI"]["model"]
                .as_str()
                .context("Missing or invalid model in config")?
                .to_string(),
        })
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let base_dirs = BaseDirs::new().context("Could not determine config directory")?;
        let config_dir = base_dirs.config_dir().join("voido");
        Ok(config_dir.join("config.toml"))
    }

    pub fn create_default_config() -> Result<()> {
        let config_file = Self::get_config_path()?;
        let config_dir = config_file
            .parent()
            .context("Invalid config directory path")?;

        // Create directory if needed
        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir).with_context(|| {
                format!("Failed to create config directory at {:?}", config_dir)
            })?;
        }

        // Write default config
        let default_config = r#"
[GEMINI]
model = "gemini-pro"  # Changed to Gemini since that's what you're using
"#;

        std::fs::write(&config_file, default_config.trim())
            .with_context(|| format!("Failed to write config to {:?}", config_file))?;

        Ok(())
    }
}

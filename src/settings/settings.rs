use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use super::colors::Colors;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub api_key: String,
    pub main_color: Colors,
    pub secondary_color: Colors,
    pub accent_color: Colors,
    pub columns: Vec<String>,
}

impl AppConfig {
    pub fn create_default_config() -> AppConfig {
        let project_dirs = ProjectDirs::from("", "", "voido").unwrap();
        let config_path = project_dirs.config_dir().join("config.toml");

        let config = AppConfig {
            api_key: String::new(),
            main_color: Colors::Light,
            secondary_color: Colors::Dark,
            accent_color: Colors::Blue,
            columns: vec![
                "ID".to_string(),
                "PRIORITY".to_string(),
                "TOPIC".to_string(),
                "TODO".to_string(),
                "SUBs".to_string(),
                "CREATED".to_string(),
                "DUE DATE".to_string(),
                "STATUS".to_string(),
                "OWNER".to_string(),
            ],
        };

        std::fs::write(config_path, toml::to_string(&config).unwrap()).unwrap();

        config
    }

    pub fn load_config() -> AppConfig {
        let project_dirs = ProjectDirs::from("", "", "voido").unwrap();
        let config_path = project_dirs.config_dir().join("config.toml");

        let config = std::fs::read_to_string(config_path).unwrap();
        toml::from_str(&config).unwrap()
    }
}

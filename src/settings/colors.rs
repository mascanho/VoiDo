use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Colors {
    // RGB VALUES
    Light,
    Dark,
    Purple,
    Blue,
    Green,
    Red,
    Orange,
    Yellow,
}

impl Colors {
    pub fn get_color(&self) -> String {
        match self {
            Colors::Light => "(0, 0, 0)".to_string(),
            Colors::Dark => "(255, 255, 255)".to_string(),
            Colors::Purple => "(128, 0, 128)".to_string(),
            Colors::Blue => "(0, 0, 255)".to_string(),
            Colors::Green => "(0, 255, 0)".to_string(),
            Colors::Red => "(255, 0, 0)".to_string(),
            Colors::Orange => "(255, 165, 0)".to_string(),
            Colors::Yellow => "(255, 255, 0)".to_string(),
        }
    }
}

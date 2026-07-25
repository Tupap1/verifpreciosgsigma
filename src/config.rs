use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub puerto: u16,
    pub sucursal: String,
    pub db_url: String,
    pub auto_update: bool,
    pub repo_owner: String,
    pub repo_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            puerto: 8080,
            sucursal: "01".to_string(),
            db_url: "mysql://root:7iu7Wi0@localhost:3306/pv".to_string(),
            auto_update: false,
            repo_owner: "Tupap1".to_string(),
            repo_name: "verifpreciosgsigma".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string("config.json") {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }
        Self::default()
    }
}

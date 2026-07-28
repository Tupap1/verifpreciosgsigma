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
            db_url: "mysql://root:password@localhost:3306/pv".to_string(),
            auto_update: true,
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
        
        // Si config.json no existe, crearlo automáticamente con una plantilla segura
        let default_cfg = Self::default();
        if let Ok(json_str) = serde_json::to_string_pretty(&default_cfg) {
            let _ = fs::write("config.json", json_str);
        }
        default_cfg
    }
}

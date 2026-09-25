use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: Option<String>,
    pub email: Option<String>,
    pub org: Option<String>,
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
}

fn default_endpoint() -> String {
    "https://cache.cuv.dev".to_string()
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token: None,
            email: None,
            org: None,
            endpoint: default_endpoint(),
        }
    }
}

pub fn get_credentials_path() -> PathBuf {
    if let Ok(custom) = std::env::var("CUV_CONFIG_DIR") {
        PathBuf::from(custom).join("credentials.toml")
    } else {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".cuv").join("credentials.toml")
    }
}

pub fn load_auth_config() -> Result<AuthConfig> {
    if let Ok(token) = std::env::var("CUV_API_TOKEN") {
        let org = std::env::var("CUV_ORG").ok();
        let email = std::env::var("CUV_EMAIL").ok();
        let endpoint = std::env::var("CUV_CLOUD_ENDPOINT").unwrap_or_else(|_| default_endpoint());
        return Ok(AuthConfig {
            token: Some(token),
            email,
            org,
            endpoint,
        });
    }

    let path = get_credentials_path();
    if path.is_file() {
        let content = std::fs::read_to_string(&path)?;
        let cfg: AuthConfig = toml::from_str(&content)?;
        return Ok(cfg);
    }

    Ok(AuthConfig::default())
}

pub fn save_auth_config(cfg: &AuthConfig) -> Result<()> {
    let path = get_credentials_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(cfg)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn clear_auth_config() -> Result<()> {
    let path = get_credentials_path();
    if path.is_file() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

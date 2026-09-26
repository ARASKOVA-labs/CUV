use crate::cloud::auth::{load_auth_config, AuthConfig};
use anyhow::Result;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CloudClient {
    pub config: AuthConfig,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct CloudStatus {
    pub connected: bool,
    pub authenticated: bool,
    pub organization: Option<String>,
    pub endpoint: String,
    pub latency: Duration,
}

impl CloudClient {
    pub fn new() -> Result<Self> {
        let config = load_auth_config()?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent("cuv-cloud-client/0.1.0")
            .build()?;
        Ok(Self { config, client })
    }

    pub fn is_authenticated(&self) -> bool {
        self.config.token.is_some()
    }

    pub async fn fetch_cached_object(&self, hash: &str) -> Result<Option<Vec<u8>>> {
        let token = match &self.config.token {
            Some(t) => t,
            None => return Ok(None),
        };

        let url = format!(
            "{}/v1/cache/{}",
            self.config.endpoint.trim_end_matches('/'),
            hash
        );
        let resp = self.client.get(&url).bearer_auth(token).send().await;

        match resp {
            Ok(res) if res.status().is_success() => {
                let bytes = res.bytes().await?;
                Ok(Some(bytes.to_vec()))
            }
            _ => Ok(None),
        }
    }

    pub async fn upload_cached_object(&self, hash: &str, bytes: &[u8]) -> Result<bool> {
        let token = match &self.config.token {
            Some(t) => t,
            None => return Ok(false),
        };

        let url = format!(
            "{}/v1/cache/{}",
            self.config.endpoint.trim_end_matches('/'),
            hash
        );
        let resp = self
            .client
            .put(&url)
            .bearer_auth(token)
            .body(bytes.to_vec())
            .send()
            .await;

        match resp {
            Ok(res) => Ok(res.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn check_status(&self) -> Result<CloudStatus> {
        let start = std::time::Instant::now();
        let url = format!("{}/health", self.config.endpoint.trim_end_matches('/'));

        let resp = self.client.get(&url).send().await;
        let latency = start.elapsed();

        let connected = resp.is_ok();
        let authenticated = self.is_authenticated();

        Ok(CloudStatus {
            connected,
            authenticated,
            organization: self.config.org.clone(),
            endpoint: self.config.endpoint.clone(),
            latency,
        })
    }
}

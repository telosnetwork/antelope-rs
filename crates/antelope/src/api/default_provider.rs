use crate::api::client::Provider;
use reqwest::Client;
use std::fmt::{Debug, Formatter};
use tracing::debug;

use super::client::ProviderError;

#[derive(Default, Clone)]
pub struct DefaultProvider {
    base_url: String,
    client: Client,
}

impl DefaultProvider {
    pub fn new(base_url: String, timeout: Option<u64>) -> Result<Self, String> {
        let mut client_builder = Client::builder();
        if let Some(timeout) = timeout {
            client_builder =
                client_builder.timeout(std::time::Duration::from_secs(timeout));
        }
        let client = client_builder
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        })
    }
}

impl Debug for DefaultProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultProvider<{}>", self.base_url)
    }
}

#[async_trait::async_trait]
impl Provider for DefaultProvider {
    async fn get(&self, path: String) -> Result<String, ProviderError> {
        debug!("GET {}", self.base_url.to_string() + &path);
        let res = self
            .client
            .get(self.base_url.to_string() + &path)
            .send()
            .await
            .map_err(|e| ProviderError::from(e.to_string()))?;

        let response = res.text()
            .await
            .map_err(|e| ProviderError::from(e.to_string()))?;

        debug!("Response: {}", response);
        Ok(response)
    }

    async fn post(&self, path: String, body: Option<String>) -> Result<String, ProviderError> {
        let mut builder = self.client.post(self.base_url.to_string() + &path);
        if let Some(body_str) = body {
            debug!("POST {} {}", self.base_url.to_string() + &path, body_str);
            builder = builder.body(body_str);
        }
        let res = builder.send()
            .await
            .map_err(|e| ProviderError::from(e.to_string()))?;

        let response = res.text()
            .await
            .map_err(|e| ProviderError::from(e.to_string()))?;

        debug!("Response: {}", response);
        Ok(response)
    }
}

use std::fmt::{Debug, Formatter};

use reqwest::Client;

use crate::api::client::Provider;

#[derive(Default, Clone)]
pub struct DefaultProvider {
    base_url: String,
    client: Client,
}

impl DefaultProvider {
    pub fn new(base_url: String) -> Result<Self, String> {
        let client = Client::builder().build();
        if client.is_err() {
            let err = client.err();
            let mut err_message = String::from("Error building http client");
            if err.is_some() {
                err_message = err.unwrap().to_string();
            }
            return Err(err_message);
        }

        let url = base_url.trim_end_matches('/');

        Ok(Self {
            base_url: String::from(url),
            client: client.unwrap(),
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
    async fn get(&self, path: String) -> Result<String, String> {
        let res = self
            .client
            .get(self.base_url.to_string() + &path)
            .send()
            .await;
        if res.is_err() {
            return Err(res.err().unwrap().to_string());
        }

        Ok(res.unwrap().text().await.unwrap())
    }

    async fn post(&self, path: String, body: Option<String>) -> Result<String, String> {
        let mut builder = self.client.post(self.base_url.to_string() + &path);
        if body.is_some() {
            builder = builder.body(body.unwrap());
        }
        let res = builder.send().await;
        if res.is_err() {
            return Err(res.err().unwrap().to_string());
        }

        Ok(res.unwrap().text().await.unwrap())
    }
}

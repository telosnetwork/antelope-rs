use std::fmt::{Debug, Formatter};

use reqwest::Client;

use crate::api::client::Provider;

#[derive(Default, Clone)]
pub struct DefaultProvider {
    debug: bool,
    base_url: String,
    client: Client,
}

impl DefaultProvider {
    pub fn new(base_url: String, timeout: Option<u64>) -> Result<Self, String> {
        let mut client_builder = Client::builder();
        if timeout.is_some() {
            client_builder =
                client_builder.timeout(std::time::Duration::from_secs(timeout.unwrap()));
        }
        let client = client_builder.build();
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
            debug: false,
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
        if self.debug {
            println!("GET {}", self.base_url.to_string() + &path);
        }

        let res = self
            .client
            .get(self.base_url.to_string() + &path)
            .send()
            .await;
        if res.is_err() {
            let res_err = res.err().unwrap().to_string();
            if self.debug {
                println!("Error: {}", res_err);
            }

            return Err(res_err);
        }

        let response = res.unwrap().text().await.unwrap();
        if self.debug {
            println!("Response: {}", response);
        }

        Ok(response)
    }

    async fn post(&self, path: String, body: Option<String>) -> Result<String, String> {
        let mut builder = self.client.post(self.base_url.to_string() + &path);
        if body.is_some() {
            let body_str = body.unwrap();
            if self.debug {
                println!("POST {} {}", self.base_url.to_string() + &path, body_str);
            }

            builder = builder.body(body_str);
        }
        let res = builder.send().await;
        if res.is_err() {
            let err_str = res.err().unwrap().to_string();
            if self.debug {
                println!("Error: {}", err_str);
            }

            return Err(err_str);
        }

        let response = res.unwrap().text().await.unwrap();
        if self.debug {
            println!("Response: {}", response);
        }

        Ok(response)
    }

    fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }
}

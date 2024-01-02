use std::fmt::{Display, Formatter};
use crate::api::default_provider::DefaultProvider;
use crate::api::v1::chain::ChainAPI;

pub enum HTTPMethod {
    GET, POST
}

impl Display for HTTPMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HTTPMethod::GET => { write!(f, "GET") }
            HTTPMethod::POST => { write!(f, "POST") }
        }
    }
}

// TODO: Make this return an APIResponse with status code, timing, etc..
pub trait Provider {
    fn post(&self, path: String, body: Option<String>) -> Result<String, String>;
    fn get(&self, path: String) -> Result<String, String>;
}

pub struct APIClient {
    pub v1_chain: ChainAPI
}

impl APIClient {
    pub fn default_provider(base_url: String) -> Result<Self, String> {
        let provider = Box::new(DefaultProvider::new(base_url).unwrap());
        APIClient::custom_provider(provider)
    }

    pub fn custom_provider(provider: Box<dyn Provider>) -> Result<Self, String> {
        Ok(APIClient {
            v1_chain: ChainAPI::new(provider)
        })
    }
}
use crate::api::v1::chain::ChainAPI;
use std::fmt::{Debug, Display, Formatter};

pub use crate::api::default_provider::DefaultProvider;

pub enum HTTPMethod {
    GET,
    POST,
}

impl Display for HTTPMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HTTPMethod::GET => {
                write!(f, "GET")
            }
            HTTPMethod::POST => {
                write!(f, "POST")
            }
        }
    }
}

// TODO: Make this return an APIResponse with status code, timing, etc..
pub trait Provider: Debug + Default + Sync + Send {
    fn post(&self, path: String, body: Option<String>) -> Result<String, String>;
    fn get(&self, path: String) -> Result<String, String>;
}

#[derive(Debug, Default, Clone)]
pub struct APIClient<P: Provider> {
    pub v1_chain: ChainAPI<P>,
}

impl<P: Provider> APIClient<P> {
    pub fn default_provider(base_url: String) -> Result<APIClient<DefaultProvider>, String> {
        let provider = DefaultProvider::new(base_url).unwrap();
        APIClient::custom_provider(provider)
    }

    pub fn custom_provider(provider: P) -> Result<Self, String> {
        Ok(APIClient {
            v1_chain: ChainAPI::new(provider),
        })
    }
}

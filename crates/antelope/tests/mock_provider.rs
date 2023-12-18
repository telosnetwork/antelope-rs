use std::fs;
use std::path::PathBuf;
use antelope::api::client::{HTTPMethod, Provider};
use antelope::chain::checksum::Checksum160;

pub struct MockProvider {
}

impl Provider for MockProvider {
    fn call(&self, method: HTTPMethod, path: String, body: Option<String>) -> Result<String, String> {
        let mut to_hash = method.to_string() + &path;
        if body.is_some() {
            to_hash += body.unwrap().as_str();
        }

        let filename = Checksum160::hash(to_hash.into_bytes()).to_string();
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("tests/mock_provider_data/");
        d.push(filename + ".json");
        Ok(fs::read_to_string(&d).unwrap())
    }
}
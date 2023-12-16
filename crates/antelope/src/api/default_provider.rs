use crate::api::client::{HTTPMethod, Provider};

pub struct DefaultProvider {
    base_url: String,
    client: reqwest::blocking::Client
}

impl DefaultProvider {
    pub fn new(base_url: String) -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder().build();
        if client.is_err() {
            let err = client.err();
            let mut err_message = String::from("Error building http client");
            if err.is_some() {
                err_message = err.unwrap().to_string();
            }
            return Err(err_message);
        }

        let url = base_url.trim_end_matches("/");

        Ok(Self {
            base_url: String::from(url),
            client: client.unwrap()
        })
    }

    fn get(&self, path: String) -> Result<String, String> {
        let res = self.client.get(self.base_url.to_string() + &path).send();
        if res.is_err() {
            return Err(res.err().unwrap().to_string());
        }

        Ok(res.unwrap().text().unwrap())
    }

    fn post(&self, path: String, body: Option<String>) -> Result<String, String> {
        let mut builder = self.client.post(self.base_url.to_string() + &path);
        if body.is_some() {
            builder = builder.body(body.unwrap());
        }
        let res = builder.send();
        if res.is_err() {
            return Err(res.err().unwrap().to_string());
        }

        Ok(res.unwrap().text().unwrap())
    }
}

impl Provider for DefaultProvider {
    fn call(&self, method: HTTPMethod, path: String, body: Option<String>) -> Result<String, String> {
        match method {
            HTTPMethod::GET => {
                self.get(path)
            }
            HTTPMethod::POST => {
                self.post(path, body)
            }
        }
    }
}
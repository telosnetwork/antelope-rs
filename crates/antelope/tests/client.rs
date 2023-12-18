use antelope::api::client::APIClient;
use crate::mock_provider::MockProvider;

#[test]
fn client() {
    let mock_provider = MockProvider{};
    let client = APIClient::custom_client(Box::new(mock_provider));
    //let client = APIClient::default_client(String::from("https://telos.caleos.io"));
    let info = client.unwrap().v1_chain.get_info();
}
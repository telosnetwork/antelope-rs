use antelope::api::client::APIClient;
use antelope::chain::block_id::BlockId;
use antelope::name;
use antelope::chain::name::Name;
use antelope::util::hex_to_bytes;
mod utils;
use utils::mock_provider::MockProvider;

#[test]
fn client() {
    let mock_provider = MockProvider{};
    let client = APIClient::custom_provider(Box::new(mock_provider));
    //let client = APIClient::default_client(String::from("https://telos.caleos.io"));
    let info = client.unwrap().v1_chain.get_info().unwrap();
    assert_eq!(info.head_block_producer, name!("bp.boid"));
    assert_eq!(info.last_irreversible_block_id.bytes, BlockId::from_bytes(&hex_to_bytes("12cf00e89773c8497415c368960b9c57ba6ee076283f71df14aeee2daefbb2a6")).unwrap().bytes);
    assert_eq!(info.last_irreversible_block_num, 315556072);
}
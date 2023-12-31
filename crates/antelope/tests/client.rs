use antelope::api::client::APIClient;
use antelope::api::v1::structs::ClientError;
use antelope::chain::asset::Asset;
use antelope::chain::block_id::BlockId;
use antelope::chain::name::Name;
use antelope::name;
use antelope::util::{bytes_to_hex, hex_to_bytes};
mod utils;
use crate::utils::mock_provider;
use utils::mock_provider::MockProvider;

#[test]
fn chain_get_info() {
    let mock_provider = MockProvider {};
    let client = APIClient::custom_provider(Box::new(mock_provider));
    //let client = APIClient::default_provider(String::from("https://telos.caleos.io"));
    let info = client.unwrap().v1_chain.get_info().unwrap();
    assert_eq!(info.head_block_producer, name!("bp.boid"));
    assert_eq!(
        info.last_irreversible_block_id.bytes,
        BlockId::from_bytes(&hex_to_bytes(
            "12cf00e89773c8497415c368960b9c57ba6ee076283f71df14aeee2daefbb2a6"
        ))
        .unwrap()
        .bytes
    );
    assert_eq!(info.last_irreversible_block_num, 315556072);
}

#[test]
fn chain_send_transaction() {
    let mock_provider = MockProvider {};
    let client = APIClient::custom_provider(Box::new(mock_provider)).unwrap();
    //let client = APIClient::default_provider(String::from("https://testnet.telos.caleos.io")).unwrap();
    let info = client.v1_chain.get_info().unwrap();
    let transaction =
        mock_provider::make_mock_transaction(&info, Asset::from_string("0.0420 TLOS"));
    let signed_transaction = mock_provider::sign_mock_transaction(&transaction, &info);
    let result = client.v1_chain.send_transaction(signed_transaction);
    assert!(result.is_ok(), "Transaction result should be ok");
    let send_trx_response = result.unwrap();

    // NOTE: Don't bother testing the transaction id from the mock transaction, it will not match because the
    // get_info that was mocked isn't the same get_info used for the mocked response value from send_transaction
    //assert_eq!(send_trx_response.transaction_id, bytes_to_hex(&transaction.id()));

    assert_eq!(
        send_trx_response.transaction_id,
        "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49"
    );
    assert_eq!(
        send_trx_response.processed.id,
        "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49"
    );
    assert_eq!(
        send_trx_response.processed.block_time,
        "2024-01-02T19:01:00.000"
    );
    assert_eq!(send_trx_response.processed.receipt.cpu_usage_us, 185);
    assert_eq!(send_trx_response.processed.elapsed, 185);

    // TODO: Create a failed send_transaction response in the mock_data, properly detect errors in v1_chain.send_transaction and test for the error struct values
    let invalid_transaction =
        mock_provider::make_mock_transaction(&info, Asset::from_string("0.0420 NUNYA"));
    let signed_invalid_transaction =
        mock_provider::sign_mock_transaction(&invalid_transaction, &info);
    let failed_result = client.v1_chain.send_transaction(signed_invalid_transaction);
    assert!(
        failed_result.is_err(),
        "Failed transaction result should be err"
    );
    let failure_response = failed_result.err().unwrap();
    match failure_response {
        ClientError::SERVER(err) => {
            assert_eq!(err.error.code, 3050003);
        }
        _ => {
            assert!(
                false,
                "Failure response should be of type ClientError::SERVER"
            )
        }
    }
}

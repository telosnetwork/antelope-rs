use antelope::api::v1::structs::ErrorResponse;
use antelope::{
    api::{
        client::APIClient,
        v1::structs::{ClientError, GetTableRowsParams},
    },
    chain::{asset::Asset, checksum::Checksum256, name::Name},
    name,
    serializer::{Decoder, Encoder, Packer},
    StructPacker,
};

mod utils;
use utils::mock_provider::MockProvider;

use crate::utils::mock_provider;

#[tokio::test]
async fn chain_get_info() {
    let mock_provider = MockProvider {};
    let client = APIClient::custom_provider(mock_provider).expect("Failed to create API client");

    let result = client.v1_chain.get_info().await;

    if let Err(e) = &result {
        println!("Deserialization error: {:?}", e);
    }
    assert!(result.is_ok());

    let result_unwrapped = result.unwrap();
    assert_eq!(result_unwrapped.server_version, "6c1717c9".to_string());
    assert_eq!(
        result_unwrapped.chain_id,
        Checksum256::from_hex("4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11")
            .unwrap()
    );

    let last_irreversible_block_id_bytes =
        hex::decode("12cf00e89773c8497415c368960b9c57ba6ee076283f71df14aeee2daefbb2a6")
            .expect("Invalid hex for last_irreversible_block_id");
    assert_eq!(
        result_unwrapped.last_irreversible_block_id.bytes, last_irreversible_block_id_bytes,
        "last_irreversible_block_id does not match"
    );

    let head_block_id_bytes =
        hex::decode("12cf02388e0ac11fedb6da8589890f55660b2c64efb758528bf3c0d4f54f5af7")
            .expect("Invalid hex for head_block_id");
    assert_eq!(
        result_unwrapped.head_block_id.bytes, head_block_id_bytes,
        "head_block_id does not match"
    );

    let expected_datetime =
        chrono::NaiveDateTime::parse_from_str("2023-12-16T16:17:47.500", "%Y-%m-%dT%H:%M:%S%.f")
            .expect("Failed to parse datetime");
    let expected_timestamp_micros = expected_datetime.timestamp_nanos_opt()
        .expect("Failed to get timestamp nanos") // Handle potential failure
        as u64
        / 1_000; // Convert nanoseconds to microseconds

    assert_eq!(
        result_unwrapped.head_block_time.elapsed, expected_timestamp_micros,
        "head_block_time does not match expected value"
    );

    assert_eq!(result_unwrapped.head_block_producer, name!("bp.boid"));
}

#[tokio::test]
async fn chain_send_transaction() {
    let mock_provider = MockProvider {};
    let client = APIClient::custom_provider(mock_provider).unwrap();
    //let client = APIClient::default_provider(String::from("https://testnet.telos.caleos.io")).unwrap();
    let info = client.v1_chain.get_info().await.unwrap();
    let transaction =
        mock_provider::make_mock_transaction(&info, Asset::from_string("0.0420 TLOS"));
    let signed_transaction = mock_provider::sign_mock_transaction(&transaction, &info);
    let result = client.v1_chain.send_transaction(signed_transaction).await;
    assert!(result.is_ok(), "Transaction result should be ok");
    let send_trx_response = result.unwrap();

    // NOTE: Don't bother testing the transaction id from the mock transaction, it
    // will not match because the get_info that was mocked isn't the same
    // get_info used for the mocked response value from send_transaction
    // assert_eq!(send_trx_response.transaction_id,
    // bytes_to_hex(&transaction.id()));

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

    // TODO: Create a failed send_transaction response in the mock_data, properly
    // detect errors in v1_chain.send_transaction and test for the error struct
    // values
    let invalid_transaction =
        mock_provider::make_mock_transaction(&info, Asset::from_string("0.0420 NUNYA"));
    let signed_invalid_transaction =
        mock_provider::sign_mock_transaction(&invalid_transaction, &info);
    let failed_result = client
        .v1_chain
        .send_transaction(signed_invalid_transaction)
        .await;
    assert!(
        failed_result.is_err(),
        "Failed transaction result should be err"
    );
    let failure_response = failed_result.err().unwrap();
    println!("{:?}", failure_response);
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

#[tokio::test]
async fn chan_get_account() {
    // Setup - replace `APIClient::custom_provider(provider)` with your actual client initialization logic
    let mock_provider = MockProvider {};
    let client = APIClient::custom_provider(mock_provider).expect("Failed to create API client");

    // Act - Attempt to retrieve the account information for "nathan"
    let parsed = client
        .v1_chain
        .get_account(String::from("foflexitytls"))
        .await;

    match parsed {
        Ok(account) => {
            assert_eq!(account.account_name, name!("foflexitytls"));

            assert_eq!(
                account.core_liquid_balance,
                Asset::from_string("128559.5000 TLOS")
            );
        }
        Err(e) => {
            // Log or handle errors here to understand parsing issues
            println!("Failed to parse JSON: {:?}", e);
            panic!("Parsing failed for the given JSON data.");
        }
    }
}

#[tokio::test]
pub async fn chain_get_abi() {
    let mock_provider = MockProvider {};
    let client = APIClient::custom_provider(mock_provider).expect("Failed to create API client");

    let result = client.v1_chain.get_abi("eosio.token".to_string()).await;

    if let Err(e) = &result {
        println!("Deserialization error: {:?}", e);
    }
    assert!(result.is_ok());

    let abi_object = result.unwrap();

    // Perform various assertions to verify the correctness of the ABI parsing
    assert_eq!(abi_object.abi.version, "eosio::abi/1.2");
    assert!(abi_object.abi.types.is_empty());

    // Check structs parsing
    assert_eq!(abi_object.abi.structs.len(), 8);
    assert_eq!(abi_object.abi.structs[0].name, "account");
    assert_eq!(abi_object.abi.structs[0].fields[0].name, "balance");
    assert_eq!(abi_object.abi.structs[0].fields[0].r#type, "asset");

    // Check actions parsing
    assert_eq!(abi_object.abi.actions.len(), 6);
    assert_eq!(abi_object.abi.actions[0].name, name!("close"));
    assert_eq!(abi_object.abi.actions[0].r#type, "close");

    // Check tables parsing
    assert_eq!(abi_object.abi.tables.len(), 2);
    assert_eq!(abi_object.abi.tables[0].name, name!("accounts"));
}

#[test]
fn test_error_response_parsing() {
    let error_json = r#"{
            "code": 500,
            "message": "Internal Service Error",
            "error": {
                "code": 3050003,
                "name": "eosio_assert_message_exception",
                "what": "eosio_assert_message assertion failure",
                "details": [
                    {
                        "message": "assertion failure with message: unable to find key",
                        "file": "cf_system.cpp",
                        "line_number": 14,
                        "method": "eosio_assert"
                    },
                    {
                        "message": "pending console output: ",
                        "file": "apply_context.cpp",
                        "line_number": 124,
                        "method": "exec_one"
                    }
                ]
            }
        }"#;

    let parsed_error: Result<ErrorResponse, _> = serde_json::from_str(error_json);
    let error_response = parsed_error.expect("Failed to parse JSON");

    assert_eq!(
        error_response.error.code, 3050003,
        "Error code did not match"
    );
    assert_eq!(
        error_response.error.name, "eosio_assert_message_exception",
        "Error name did not match"
    );
    assert_eq!(
        error_response.error.what, "eosio_assert_message assertion failure",
        "Error what did not match"
    );

    assert_eq!(
        error_response.error.details.len(),
        2,
        "Expected 2 details, found {}",
        error_response.error.details.len()
    );

    let detail1 = &error_response.error.details[0];
    assert_eq!(
        detail1.message, "assertion failure with message: unable to find key",
        "First detail message did not match"
    );
    assert_eq!(
        detail1.file, "cf_system.cpp",
        "First detail file did not match"
    );
    assert_eq!(
        detail1.line_number, 14,
        "First detail line number did not match"
    );
    assert_eq!(
        detail1.method, "eosio_assert",
        "First detail method did not match"
    );

    let detail2 = &error_response.error.details[1];
    assert_eq!(
        detail2.message, "pending console output: ",
        "Second detail message did not match"
    );
    assert_eq!(
        detail2.file, "apply_context.cpp",
        "Second detail file did not match"
    );
    assert_eq!(
        detail2.line_number, 124,
        "Second detail line number did not match"
    );
    assert_eq!(
        detail2.method, "exec_one",
        "Second detail method did not match"
    );
}

#[tokio::test]
pub async fn chain_get_table_rows() {
    #[derive(StructPacker, Default)]
    struct UserRow {
        balance: Asset,
    }

    let mock_provider = MockProvider {};
    let client = APIClient::custom_provider(mock_provider).unwrap();
    //let client = APIClient::default_provider(String::from("https://testnet.telos.caleos.io")).unwrap();

    let res1 = client
        .v1_chain
        .get_table_rows::<UserRow>(GetTableRowsParams {
            code: name!("eosio.token"),
            table: name!("accounts"),
            scope: Some(name!("corecorecore")),
            lower_bound: None,
            upper_bound: None,
            limit: None,
            reverse: None,
            index_position: None,
            show_payer: None,
        })
        .await
        .unwrap();

    assert_eq!(res1.rows.len(), 1, "Should get 1 row back");
    assert_eq!(
        res1.rows[0].balance.symbol().code().to_string(),
        "TLOS",
        "Should get TLOS symbol back"
    );

    #[derive(StructPacker, Default)]
    struct TelosEVMConfig {
        trx_index: u32,
        last_block: u32,
        gas_used_block: Checksum256,
        gas_price: Checksum256,
        revision: Option<u32>,
    }

    let res2 = client
        .v1_chain
        .get_table_rows::<TelosEVMConfig>(GetTableRowsParams {
            code: name!("eosio.evm"),
            table: name!("config"),
            scope: Some(name!("eosio.evm")),
            lower_bound: None,
            upper_bound: None,
            limit: Some(1),
            reverse: None,
            index_position: None,
            show_payer: None,
        })
        .await
        .unwrap();

    assert_eq!(res2.rows.len(), 1, "Should get 1 config row back");
    assert!(
        res2.rows[0].revision.is_none(),
        "Empty binary extension should be None"
    );

    // const res1 = await eos.v1.chain.get_table_rows({
    //     code: 'fuel.gm',
    //     table: 'users',
    //     type: User,
    //     limit: 1,
    // })
    // assert.equal(res1.rows[0].account instanceof Name, true)
    // assert.equal(res1.more, true)
    // assert.equal(String(res1.rows[0].account), 'aaaa')
    // const res2 = await eos.v1.chain.get_table_rows({
    //     code: 'fuel.gm',
    //     table: 'users',
    //     type: User,
    //     limit: 2,
    //     lower_bound: res1.next_key,
    // })
    // assert.equal(String(res2.rows[0].account), 'atomichub')
    // assert.equal(String(res2.next_key), 'boidservices')
    // assert.equal(Number(res2.rows[1].balance).toFixed(6), (0.02566).toFixed(6))
}

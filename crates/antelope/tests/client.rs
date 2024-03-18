use antelope::api::v1::structs::TESTGetInfoResponse;
use antelope::{
    api::{
        client::APIClient,
        v1::structs::{
            AccountRamDelta, ActionTrace, ClientError, EncodingError, GetTableRowsParams,
        },
    },
    chain::{asset::Asset, block_id::BlockId, checksum::Checksum256, name::Name},
    name,
    serializer::{Decoder, Encoder, Packer},
    util::hex_to_bytes,
    StructPacker,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

mod utils;
use utils::mock_provider::MockProvider;

use crate::utils::mock_provider;

#[tokio::test]
async fn chain_get_info() {
    let mock_provider = MockProvider {};
    let client = APIClient::custom_provider(mock_provider);
    //let client = APIClient::default_provider(String::from("https://telos.caleos.io"));
    let info = client.unwrap().v1_chain.get_info().await.unwrap();
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

pub fn parse_action_traces(action_traces_json: Value) -> Result<Vec<ActionTrace>, EncodingError> {
    serde_json::from_value(action_traces_json)
        .map_err(|e| EncodingError::new(format!("Failed to deserialize 'Vec<ActionTrace>': {}", e)))
}

//Tests to manually check each items parsing in get info

#[test]
fn deserialize_getinfo() {
    let json = r#"{
        "server_version": "6c1717c9",
        "chain_id": "4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11",
        "head_block_num": 315556408,
        "last_irreversible_block_num": 315556072,
        "last_irreversible_block_id": "12cf00e89773c8497415c368960b9c57ba6ee076283f71df14aeee2daefbb2a6",
        "head_block_id": "12cf02388e0ac11fedb6da8589890f55660b2c64efb758528bf3c0d4f54f5af7",
        "head_block_time": "2023-12-16T16:17:47.500",
        "head_block_producer": "bp.boid",
        "virtual_block_cpu_limit": 200000000,
        "virtual_block_net_limit": 1048576000,
        "block_cpu_limit": 200000,
        "block_net_limit": 1048576,
        "server_version_string": "v4.0.3-hotfix",
        "fork_db_head_block_num": 315556408,
        "fork_db_head_block_id": "12cf02388e0ac11fedb6da8589890f55660b2c64efb758528bf3c0d4f54f5af7",
        "server_full_version_string": "v4.0.3-hotfix-6c1717c94394a9713d12b1a5a1742598300f6042",
        "total_cpu_weight": "53576658287",
        "total_net_weight": "45215580902",
        "earliest_available_block_num": 1,
        "last_irreversible_block_time": "2023-12-16T16:14:59.500"
    }"#;

    let result: Result<TESTGetInfoResponse, _> = serde_json::from_str(json);
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TransferData {
    from: String,
    to: String,
    quantity: String,
    memo: String,
}

#[test]
fn test_parse_first_action_trace() {
    // Mock JSON for the first action trace
    let mock_action_trace_json = json!({
        "action_ordinal": 1,
        "creator_action_ordinal": 0,
        "closest_unnotified_ancestor_action_ordinal": 0,
        "receipt": {
            "receiver": "eosio.token",
            "act_digest": "cadbd7470130836a0ca0c9403155b219c4776738378f09eda4d6ff7e4eee4530",
            "global_sequence": 383003514,
            "recv_sequence": 1837548,
            "auth_sequence": [
                ["corecorecore", 13]
            ],
            "code_sequence": 7,
            "abi_sequence": 8
        },
        "receiver": "eosio.token",
        "act": {
            "account": "eosio.token",
            "name": "transfer",
            "authorization": [
                {
                    "actor": "corecorecore",
                    "permission": "active"
                }
            ],
            "data": {
                "from": "corecorecore",
                "to": "teamgreymass",
                "quantity": "0.0420 TLOS",
                "memo": "Testing antelope-rs"
            },
            "hex_data": "a02e45ea52a42e4580b1915e5d268dcaa40100000000000004544c4f530000001354657374696e6720616e74656c6f70652d7273"
        },
        "context_free": false,
        "elapsed": 74,
        "console": "",
        "trx_id": "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49",
        "block_num": 275003381,
        "block_time": "2024-01-02T19:01:00.000",
        "producer_block_id": null,
        "account_ram_deltas": [],
        "except": null,
        "error_code": null,
        "return_value_hex_data": ""
    });

    // Deserialize the JSON into an ActionTrace instance
    let action_trace: ActionTrace = serde_json::from_value(mock_action_trace_json)
        .expect("Failed to parse ActionTrace from JSON");

    // Assertions to verify each field of the ActionTrace struct
    assert_eq!(action_trace.action_ordinal, 1);
    assert_eq!(action_trace.creator_action_ordinal, 0);
    assert_eq!(action_trace.closest_unnotified_ancestor_action_ordinal, 0);

    // Verifying the 'receipt' field
    let receipt = action_trace.receipt;
    assert_eq!(receipt.receiver, name!("eosio.token"));
    assert_eq!(
        receipt.act_digest,
        "cadbd7470130836a0ca0c9403155b219c4776738378f09eda4d6ff7e4eee4530"
    );
    assert_eq!(receipt.global_sequence, 383003514);
    assert_eq!(receipt.recv_sequence, 1837548);
    assert_eq!(receipt.auth_sequence[0].account, name!("corecorecore"));
    assert_eq!(receipt.auth_sequence[0].sequence, 13);
    assert_eq!(receipt.code_sequence, 7);
    assert_eq!(receipt.abi_sequence, 8);

    // Verifying the 'act' field
    let act = action_trace.act;
    assert_eq!(act.account, name!("eosio.token"));
    assert_eq!(act.name, name!("transfer"));
    assert_eq!(act.authorization.len(), 1);
    assert_eq!(act.authorization[0].actor, name!("corecorecore"));
    assert_eq!(act.authorization[0].permission, name!("active"));

    //missing tests for act.data

    // Verifying other fields
    assert!(!action_trace.context_free);
    assert_eq!(action_trace.elapsed, 74);
    assert_eq!(action_trace.console, "");
    assert_eq!(
        action_trace.trx_id,
        "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49"
    );
    assert_eq!(action_trace.block_num, 275003381);
    assert_eq!(action_trace.block_time, "2024-01-02T19:01:00.000");
    assert!(action_trace.producer_block_id.is_none());
    assert!(action_trace.account_ram_deltas.is_empty());
    assert!(action_trace.except.is_none());
    assert!(action_trace.error_code.is_none());
    assert_eq!(action_trace.return_value_hex_data, "");
}

#[test]
fn test_account_ram_delta_deserialization() {
    // Scenario 1: account_ram_delta is present
    let mock_json_present = json!({
        "account": "testaccount",
        "delta": 42
    });

    let account_ram_delta_present: Result<Option<AccountRamDelta>, serde_json::error::Error> =
        serde_json::from_value(mock_json_present).map(Some);

    assert!(
        account_ram_delta_present.is_ok(),
        "Deserialization should succeed when data is present"
    );
    assert_eq!(
        account_ram_delta_present.unwrap(),
        Some(AccountRamDelta {
            account: Name::new_from_str("testaccount"),
            delta: 42,
        }),
        "Deserialized data does not match expected value"
    );

    // Scenario 2: account_ram_delta is absent (represented as null in JSON)
    let mock_json_absent = json!(null);
    let account_ram_delta_absent: Option<AccountRamDelta> =
        serde_json::from_value(mock_json_absent).ok();

    assert!(
        account_ram_delta_absent.is_none(),
        "Should be None when absent"
    );
}

#[derive(Serialize, Deserialize)]
struct ActionData {
    from: String,
    to: String,
    quantity: String,
    memo: String,
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

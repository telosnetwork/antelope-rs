use antelope::api::client::APIClient;
use antelope::api::v1::structs::EncodingError;
use antelope::api::v1::structs::{AccountRamDelta, ActionTrace, ClientError, GetTableRowsParams};
use antelope::chain::asset::Asset;
use antelope::chain::block_id::BlockId;
use antelope::chain::checksum::Checksum256;
use antelope::chain::name::Name;
use antelope::name;
use antelope::serializer::{Decoder, Encoder, Packer};
use antelope::util::hex_to_bytes;
use antelope::StructPacker;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
mod utils;
use crate::utils::mock_provider;
use utils::mock_provider::MockProvider;

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

#[test]
fn test_parse_action_traces() {
    // Mock JSON structure for action traces, reflecting the provided mock data.
    let mock_action_traces_json = json!([
        {
            "action_ordinal": 1,
            "creator_action_ordinal": 0,
            "closest_unnotified_ancestor_action_ordinal": 0,
            "receipt": {
                "receiver": "eosio.token",
                "act_digest": "cadbd7470130836a0ca0c9403155b219c4776738378f09eda4d6ff7e4eee4530",
                "global_sequence": 383003514,
                "recv_sequence": 1837548,
                "auth_sequence": [["corecorecore", 13]],
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
                }
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
        },
        {
            "action_ordinal": 2,
            "creator_action_ordinal": 1,
            "closest_unnotified_ancestor_action_ordinal": 1,
            "receipt": {
                "receiver": "corecorecore",
                "act_digest": "cadbd7470130836a0ca0c9403155b219c4776738378f09eda4d6ff7e4eee4530",
                "global_sequence": 383003515,
                "recv_sequence": 6,
                "auth_sequence": [["corecorecore", 14]],
                "code_sequence": 7,
                "abi_sequence": 8
            },
            "receiver": "corecorecore",
            "act": {
                "account": "eosio.token",
                "name": "transfer",
                "authorization": [{"actor": "corecorecore", "permission": "active"}],
                "data": {"from": "corecorecore", "to": "teamgreymass", "quantity": "0.0420 TLOS", "memo": "Testing antelope-rs"}
            },
            "context_free": false,
            "elapsed": 3,
            "console": "",
            "trx_id": "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49",
            "block_num": 275003381,
            "block_time": "2024-01-02T19:01:00.000",
            "producer_block_id": null,
            "account_ram_deltas": [],
            "except": null,
            "error_code": null,
            "return_value_hex_data": ""
        },
        {
            "action_ordinal": 3,
            "creator_action_ordinal": 1,
            "closest_unnotified_ancestor_action_ordinal": 1,
            "receipt": {
                "receiver": "teamgreymass",
                "act_digest": "cadbd7470130836a0ca0c9403155b219c4776738378f09eda4d6ff7e4eee4530",
                "global_sequence": 383003516,
                "recv_sequence": 23,
                "auth_sequence": [["corecorecore", 15]],
                "code_sequence": 7,
                "abi_sequence": 8
            },
            "receiver": "teamgreymass",
            "act": {
                "account": "eosio.token",
                "name": "transfer",
                "authorization": [{"actor": "corecorecore", "permission": "active"}],
                "data": {"from": "corecorecore", "to": "teamgreymass", "quantity": "0.0420 TLOS", "memo": "Testing antelope-rs"}
            },
            "context_free": false,
            "elapsed": 6,
            "console": "",
            "trx_id": "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49",
            "block_num": 275003381,
            "block_time": "2024-01-02T19:01:00.000",
            "producer_block_id": null,
            "account_ram_deltas": [],
            "except": null,
            "error_code": null,
            "return_value_hex_data": ""
        }
    ]);

    let action_traces_result: Result<Vec<ActionTrace>, EncodingError> =
        parse_action_traces(mock_action_traces_json);

    assert!(
        action_traces_result.is_ok(),
        "Parsing action traces should succeed"
    );

    let action_traces = action_traces_result.expect("Failed to parse Vec<ActionTrace> from result");

    // Assert the correct number of action traces parsed
    assert_eq!(
        action_traces.len(),
        3,
        "There should be exactly 3 action traces parsed."
    );

    // Now, assert details for each action trace based on the provided data
    for (index, action_trace) in action_traces.iter().enumerate() {
        // Generic assertions applicable to all traces
        assert_eq!(
            action_trace.context_free,
            false,
            "Context free flag should match for trace {}",
            index + 1
        );
        assert_eq!(
            action_trace.console,
            "",
            "Console output should match for trace {}",
            index + 1
        );
        assert_eq!(
            action_trace.trx_id,
            "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49",
            "Transaction ID should match for trace {}",
            index + 1
        );
        assert_eq!(
            action_trace.block_num,
            275003381,
            "Block number should match for trace {}",
            index + 1
        );
        assert_eq!(
            action_trace.block_time,
            "2024-01-02T19:01:00.000",
            "Block time should match for trace {}",
            index + 1
        );

        // Specific assertions for each trace based on unique values
        match index {
            0 => {
                // Assertions for the first action trace
                assert_eq!(
                    action_trace.action_ordinal, 1,
                    "Action ordinal should match for the first trace."
                );
                assert_eq!(
                    action_trace.creator_action_ordinal, 0,
                    "Creator action ordinal should match for the first trace."
                );
                assert_eq!(
                    action_trace.closest_unnotified_ancestor_action_ordinal, 0,
                    "Closest unnotified ancestor action ordinal should match for the first trace."
                );

                // Receipt assertions for the first trace
                let receipt = &action_trace.receipt;
                assert_eq!(
                    receipt.receiver,
                    Name::new("eosio.token"),
                    "Receipt receiver should match for the first trace."
                );
                assert_eq!(
                    receipt.act_digest,
                    "cadbd7470130836a0ca0c9403155b219c4776738378f09eda4d6ff7e4eee4530",
                    "Act digest should match for the first trace."
                );
                assert_eq!(
                    receipt.global_sequence, 383003514,
                    "Global sequence should match for the first trace."
                );
                assert_eq!(
                    receipt.recv_sequence, 1837548,
                    "Recv sequence should match for the first trace."
                );

                // Auth sequence within receipt for the first trace
                assert_eq!(
                    receipt.auth_sequence.len(),
                    1,
                    "Auth sequence should contain one entry for the first trace."
                );
                let first_auth_sequence = &receipt.auth_sequence[0];
                assert_eq!(
                    first_auth_sequence.account,
                    Name::new("corecorecore"),
                    "Auth sequence account should match for the first trace."
                );
                assert_eq!(
                    first_auth_sequence.sequence, 13,
                    "Auth sequence number should match for the first trace."
                );

                assert_eq!(
                    receipt.code_sequence, 7,
                    "Code sequence should match for the first trace."
                );
                assert_eq!(
                    receipt.abi_sequence, 8,
                    "ABI sequence should match for the first trace."
                );

                // Act assertions for the first trace
                let act = &action_trace.act;
                assert_eq!(
                    act.account,
                    Name::new("eosio.token"),
                    "Act account should match for the first trace."
                );
                assert_eq!(
                    act.name,
                    Name::new("transfer"),
                    "Act name should match for the first trace."
                );

                // Authorization within act for the first trace
                assert_eq!(
                    act.authorization.len(),
                    1,
                    "Authorization should contain one entry for the first trace."
                );
                let first_authorization = &act.authorization[0];
                assert_eq!(
                    first_authorization.actor,
                    Name::new("corecorecore"),
                    "Authorization actor should match for the first trace."
                );
                assert_eq!(
                    first_authorization.permission,
                    Name::new("active"),
                    "Authorization permission should match for the first trace."
                );

                // Additional fields for the first trace
                assert!(
                    !action_trace.context_free,
                    "Context free flag should match for the first trace."
                );
                assert_eq!(
                    action_trace.elapsed, 74,
                    "Elapsed time should match for the first trace."
                );
                assert_eq!(
                    action_trace.console, "",
                    "Console output should match for the first trace."
                );
                assert_eq!(
                    action_trace.trx_id,
                    "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49",
                    "Transaction ID should match for the first trace."
                );
                assert_eq!(
                    action_trace.block_num, 275003381,
                    "Block number should match for the first trace."
                );
                assert_eq!(
                    action_trace.block_time, "2024-01-02T19:01:00.000",
                    "Block time should match for the first trace."
                );
                assert!(
                    action_trace.producer_block_id.is_none(),
                    "Producer block ID should be None for the first trace."
                );
                assert!(
                    action_trace.account_ram_deltas.is_empty(),
                    "Account RAM deltas should be empty for the first trace."
                );
                assert!(
                    action_trace.except.is_none(),
                    "Except should be None for the first trace."
                );
                assert!(
                    action_trace.error_code.is_none(),
                    "Error code should be None for the first trace."
                );
                assert_eq!(
                    action_trace.return_value_hex_data, "",
                    "Return value hex data should match for the first trace."
                );
            }
            1 => {
                assert_eq!(
                    action_trace.action_ordinal, 2,
                    "Action ordinal should match for the second trace"
                );
                assert_eq!(
                    action_trace.elapsed, 3,
                    "Elapsed time should match for the second trace"
                );
                // Add more specific assertions for the second trace here...
            }
            2 => {
                assert_eq!(
                    action_trace.action_ordinal, 3,
                    "Action ordinal should match for the third trace."
                );
                assert_eq!(
                    action_trace.creator_action_ordinal, 1,
                    "Creator action ordinal should match for the third trace."
                );
                assert_eq!(
                    action_trace.closest_unnotified_ancestor_action_ordinal, 1,
                    "Closest unnotified ancestor action ordinal should match for the third trace."
                );

                // Receipt assertions for the third trace
                let receipt = &action_trace.receipt;
                assert_eq!(
                    receipt.receiver,
                    Name::new("teamgreymass"),
                    "Receipt receiver should match for the third trace."
                );
                assert_eq!(
                    receipt.act_digest,
                    "cadbd7470130836a0ca0c9403155b219c4776738378f09eda4d6ff7e4eee4530",
                    "Act digest should match for the third trace."
                );
                assert_eq!(
                    receipt.global_sequence, 383003516,
                    "Global sequence should match for the third trace."
                );
                assert_eq!(
                    receipt.recv_sequence, 23,
                    "Recv sequence should match for the third trace."
                );

                // Auth sequence within receipt for the third trace
                assert_eq!(
                    receipt.auth_sequence.len(),
                    1,
                    "Auth sequence should contain one entry for the third trace."
                );
                let first_auth_sequence = &receipt.auth_sequence[0];
                assert_eq!(
                    first_auth_sequence.account,
                    Name::new("corecorecore"),
                    "Auth sequence account should match for the third trace."
                );
                assert_eq!(
                    first_auth_sequence.sequence, 15,
                    "Auth sequence number should match for the third trace."
                );

                assert_eq!(
                    receipt.code_sequence, 7,
                    "Code sequence should match for the third trace."
                );
                assert_eq!(
                    receipt.abi_sequence, 8,
                    "ABI sequence should match for the third trace."
                );

                // Act assertions for the third trace
                let act = &action_trace.act;
                assert_eq!(
                    act.account,
                    Name::new("eosio.token"),
                    "Act account should match for the third trace."
                );
                assert_eq!(
                    act.name,
                    Name::new("transfer"),
                    "Act name should match for the third trace."
                );

                // Authorization within act for the third trace
                assert_eq!(
                    act.authorization.len(),
                    1,
                    "Authorization should contain one entry for the third trace."
                );
                let first_authorization = &act.authorization[0];
                assert_eq!(
                    first_authorization.actor,
                    Name::new("corecorecore"),
                    "Authorization actor should match for the third trace."
                );
                assert_eq!(
                    first_authorization.permission,
                    Name::new("active"),
                    "Authorization permission should match for the third trace."
                );

                // Data within act for the third trace
                // Assuming `data` parsing and comparison logic is already implemented elsewhere or not directly compared due to its binary format

                assert_eq!(
                    action_trace.elapsed, 6,
                    "Elapsed time should match for the third trace."
                );
                assert!(
                    !action_trace.context_free,
                    "Context free flag should match for the third trace."
                );
                assert_eq!(
                    action_trace.console, "",
                    "Console output should match for the third trace."
                );
                assert_eq!(
                    action_trace.trx_id,
                    "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49",
                    "Transaction ID should match for the third trace."
                );
                assert_eq!(
                    action_trace.block_num, 275003381,
                    "Block number should match for the third trace."
                );
                assert_eq!(
                    action_trace.block_time, "2024-01-02T19:01:00.000",
                    "Block time should match for the third trace."
                );
                assert!(
                    action_trace.producer_block_id.is_none(),
                    "Producer block ID should be None for the third trace."
                );
                assert!(
                    action_trace.account_ram_deltas.is_empty(),
                    "Account RAM deltas should be empty for the third trace."
                );
                assert!(
                    action_trace.except.is_none(),
                    "Except should be None for the third trace."
                );
                assert!(
                    action_trace.error_code.is_none(),
                    "Error code should be None for the third trace."
                );
                assert_eq!(
                    action_trace.return_value_hex_data, "",
                    "Return value hex data should match for the third trace."
                );
            }
            _ => unreachable!(),
        }
    }
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
            delta: 42
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

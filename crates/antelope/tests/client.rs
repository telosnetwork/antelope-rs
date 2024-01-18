use antelope::api::client::APIClient;
use antelope::api::v1::structs::{ClientError, GetTableRowsParams};
use antelope::chain::asset::Asset;
use antelope::chain::block_id::BlockId;
use antelope::chain::checksum::Checksum256;
use antelope::chain::name::Name;
use antelope::name;
use antelope::serializer::{Decoder, Encoder, Packer};
use antelope::util::hex_to_bytes;
use antelope::StructPacker;
use antelope::serializer::formatter::JSONObject;
use antelope::util::hex_to_bytes;
use antelope::api::v1::utils::parse_action_trace;
use serde::{Serialize, Deserialize};
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

#[test]
fn test_parse_action_trace() {
    // Setup a complete mock JSON object representing an action trace
    let mock_json = r#"
    {
        "action_ordinal": 1,
        "creator_action_ordinal": 0,
        "closest_unnotified_ancestor_action_ordinal": 0,
        "receipt": {
          "receiver": "eosio.token",
          "act_digest": "cadbd7470130836a0ca0c9403155b219c4776738378f09eda4d6ff7e4eee4530",
          "global_sequence": 383003514,
          "recv_sequence": 1837548,
          "auth_sequence": [
            [
              "corecorecore",
              13
            ]
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
      }
    "#;

    let json_value: serde_json::Value = serde_json::from_str(mock_json).unwrap();
    let json_object = JSONObject::new(json_value);

    let result = parse_action_trace(json_object);


    assert!(result.is_ok());

    let action_trace = result.unwrap();

    // Assert individual fields of action_trace
    assert_eq!(action_trace.action_ordinal, 1);
    assert_eq!(action_trace.creator_action_ordinal, 0);
    assert_eq!(action_trace.closest_unnotified_ancestor_action_ordinal, 0);

    // Asserting fields inside receipt
    assert_eq!(action_trace.receipt.receiver, name!("eosio.token"));
    assert_eq!(action_trace.receipt.act_digest, "cadbd7470130836a0ca0c9403155b219c4776738378f09eda4d6ff7e4eee4530");
    assert_eq!(action_trace.receipt.global_sequence, 383003514);
    assert_eq!(action_trace.receipt.recv_sequence, 1837548);
    // Add more asserts for auth_sequence, code_sequence, and abi_sequence

    // Asserting fields inside act
    assert_eq!(action_trace.act.account, name!("eosio.token"));
    assert_eq!(action_trace.act.name, name!("transfer"));

    assert_eq!(action_trace.elapsed, 74);
    assert_eq!(action_trace.context_free, false);
    assert_eq!(action_trace.console, "");
    assert_eq!(action_trace.trx_id, "57dcff5a6dd9eed1a9a4b4554ed6aa69b4caf5f73b6abdf466ee61829cfaed49");
    assert_eq!(action_trace.block_num, 275003381);
    assert_eq!(action_trace.block_time, "2024-01-02T19:01:00.000");

    // Since producer_block_id, account_ram_deltas, except, error_code, and return_value_hex_data are optional and null in the mock, they should be None or empty
    assert!(action_trace.producer_block_id.is_none());
    assert!(action_trace.account_ram_deltas.is_empty());
    assert!(action_trace.except.is_none());
    assert!(action_trace.error_code.is_none());
    assert_eq!(action_trace.return_value_hex_data, "");
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

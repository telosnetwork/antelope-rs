use serde_json::Value;
use crate::api::client::{Provider};
use crate::chain::block_id::BlockId;
use crate::api::v1::structs::{ClientError, GetInfoResponse, ProcessedTransaction, ProcessedTransactionReceipt, SendTransactionResponse, SendTransactionResponseError};
use crate::chain::checksum::Checksum256;
use crate::chain::time::TimePoint;
use crate::chain::name::Name;
use crate::chain::transaction::{CompressionType, PackedTransaction, SignedTransaction};
use crate::name;
use crate::serializer::formatter::{JSONObject};

pub struct ChainAPI {
    provider: Box<dyn Provider>
}

impl ChainAPI {

    pub fn new(provider: Box<dyn Provider>) -> Self {
        ChainAPI {
            provider
        }
    }

    pub fn get_info(&self) -> Result<GetInfoResponse, ClientError<()>> {
        let result = self.provider.get(String::from("/v1/chain/get_info"));
        let json = serde_json::from_str(result.unwrap().as_str());
        if json.is_err() {
            return Err(ClientError::encoding("Failed to parse JSON".into()));
        }
        let obj = JSONObject::new(json.unwrap());
        Ok(GetInfoResponse {
            server_version: obj.get_string("server_version")?,
            chain_id: Checksum256::from_hex(obj.get_string("chain_id")?.as_str())?,
            head_block_num: obj.get_u32("head_block_num")?,
            last_irreversible_block_num: obj.get_u32("last_irreversible_block_num")?,
            last_irreversible_block_id: BlockId { bytes: obj.get_hex_bytes("last_irreversible_block_id")? },
            head_block_id: BlockId { bytes: obj.get_hex_bytes("head_block_id")? },
            head_block_time: TimePoint::from_timestamp(obj.get_str("head_block_time")?)?,
            head_block_producer: name!(obj.get_str("head_block_producer")?),
            virtual_block_cpu_limit: obj.get_u64("virtual_block_cpu_limit")?,
            virtual_block_net_limit: obj.get_u64("virtual_block_net_limit")?,
            block_cpu_limit: obj.get_u64("block_cpu_limit")?,
            block_net_limit: obj.get_u64("block_net_limit")?,
            server_version_string: obj.get_string("server_version_string").ok(),
            fork_db_head_block_num: obj.get_u32("fork_db_head_block_num").ok(),
            fork_db_head_block_id: BlockId::from_bytes(&obj.get_hex_bytes("fork_db_head_block_id")?).ok()
        })
    }

    pub fn send_transaction(&self, trx: SignedTransaction) -> Result<SendTransactionResponse, ClientError<SendTransactionResponseError>> {
        let packed_result = PackedTransaction::from_signed(trx, CompressionType::ZLIB);
        if packed_result.is_err() {
            return Err(ClientError::encoding("Failed to pack transaction".into()));
        }
        let packed = packed_result.unwrap();
        let trx_json = packed.to_json();
        let result = self.provider.post(String::from("/v1/chain/send_transaction"), Some(trx_json));
        let json: Value = serde_json::from_str(result.unwrap().as_str()).unwrap();
        let response_obj = JSONObject::new(json);
        if response_obj.has("code") {
            let error_value = response_obj.get_value("error").unwrap();
            let error_json = error_value.to_string();
            let error_obj = JSONObject::new(error_value);
            return Err(ClientError::server(SendTransactionResponseError {
                code: error_obj.get_u32("code")?,
                name: error_obj.get_string("name")?,
                message: error_json,
                stack: vec![],
            }))
        }
        let processed_obj = JSONObject::new(response_obj.get_value("processed").unwrap());
        let receipt_obj = JSONObject::new(processed_obj.get_value("receipt").unwrap());

        Ok(SendTransactionResponse {
            transaction_id: response_obj.get_string("transaction_id")?,
            processed: ProcessedTransaction {
                id: processed_obj.get_string("id")?,
                block_num: processed_obj.get_u64("block_num")?,
                block_time: processed_obj.get_string("block_time")?,
                receipt: ProcessedTransactionReceipt {
                    status: receipt_obj.get_string("status")?,
                    cpu_usage_us: receipt_obj.get_u32("cpu_usage_us")?,
                    net_usage_words: receipt_obj.get_u32("net_usage_words")?,
                },
                elapsed: processed_obj.get_u64("elapsed")?,
                except: None,
                net_usage: processed_obj.get_u32("net_usage")?,
                scheduled: false,
                action_traces: "".to_string(),  // TODO: Properly encode this
                account_ram_delta: "".to_string(), // TODO: Properly encode this
            },
        })
    }
}

use serde_json::map::Values;
use serde_json::Value;
use crate::api::client::{Provider};
use crate::api::client::HTTPMethod::{GET, POST};
use crate::chain::block_id::BlockId;
use crate::api::v1::structs::GetInfoResponse;
use crate::chain::checksum::Checksum256;
use crate::chain::time::TimePoint;
use crate::chain::name::Name;
use crate::chain::transaction::SignedTransaction;
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

    pub fn get_info(&self) -> Result<GetInfoResponse, String> {
        let result = self.provider.call(GET, String::from("/v1/chain/get_info"), None);
        let json = serde_json::from_str(result.unwrap().as_str());
        if json.is_err() {
            return Err(String::from("Failed to parse JSON"));
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

    pub fn send_transaction(&self, trx: SignedTransaction) -> Result<GetInfoResponse, String> {
        let result = self.provider.call(POST, String::from("/v1/chain/send_transaction"), trx);
        let json = serde_json::from_str(result.unwrap().as_str());
        if json.is_err() {
            return Err(String::from("Failed to parse JSON"));
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
}

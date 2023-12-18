use serde_json::Value;
use crate::api::client::{APIClient, Provider};
use crate::api::client::HTTPMethod::GET;
use crate::chain::block_id::BlockId;
use crate::api::v1::structs::GetInfoResponse;

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
        let json: Value = serde_json::from_str(result.unwrap().as_str()).unwrap();
        Ok(GetInfoResponse {
            server_version: "".to_string(),
            chain_id: Default::default(),
            head_block_num: 0,
            last_irreversible_block_num: 0,
            last_irreversible_block_id: BlockId { bytes: vec![] },
            head_block_id: BlockId { bytes: vec![] },
            head_block_time: Default::default(),
            head_block_producer: Default::default(),
            virtual_block_cpu_limit: 0,
            virtual_block_net_limit: 0,
            block_cpu_limit: 0,
            block_net_limit: 0,
            server_version_string: None,
            fork_db_head_block_num: None,
            fork_db_head_block_id: None,
        })
    }
}

use crate::chain::{
    checksum::Checksum256,
    name::Name,
    time::{TimePoint, TimePointSec},
    block_id::BlockId,
    transaction::TransactionHeader,
    varint::VarUint32,
};

pub struct GetInfoResponse {
    server_version: String,
    chain_id: Checksum256,
    head_block_num: u32,
    last_irreversible_block_num: u32,
    last_irreversible_block_id: BlockId,
    head_block_id: BlockId,
    head_block_time: TimePoint,
    head_block_producer: Name,
    virtual_block_cpu_limit: u64,
    virtual_block_net_limit: u64,
    block_cpu_limit: u64,
    block_net_limit: u64,
    server_version_string: Option<String>,
    fork_db_head_block_num: Option<u32>,
    fork_db_head_block_id: Option<BlockId>
}

impl GetInfoResponse {
    pub fn get_transaction_header(&self, seconds_ahead: u32) -> TransactionHeader {
        let expiration = TimePointSec {
            // head_block_time.elapsed is microseconds, convert to seconds
            seconds: (self.head_block_time.elapsed / 1000 / 1000) as u32 + seconds_ahead
        };
        let id = self.last_irreversible_block_id.bytes.to_vec();
        let prefix_array = &id[8..12];
        let prefix = u32::from_ne_bytes(prefix_array.try_into().unwrap());
        TransactionHeader {
            max_net_usage_words: VarUint32::default(),
            max_cpu_usage_ms: 0,
            delay_sec: VarUint32::default(),
            expiration,
            ref_block_num: (self.last_irreversible_block_num & 0xffff) as u16,
            ref_block_prefix: prefix
        }
    }
}
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::chain::asset::Asset;
use crate::chain::{
    action::Action,
    block_id::{deserialize_block_id, deserialize_optional_block_id, BlockId},
    checksum::{deserialize_checksum256, Checksum160, Checksum256},
    name::{deserialize_name, deserialize_optional_name, Name},
    time::{deserialize_optional_timepoint, deserialize_timepoint, TimePoint, TimePointSec},
    transaction::TransactionHeader,
    varint::VarUint32,
};

#[derive(Debug)]
pub enum ClientError<T> {
    SIMPLE(SimpleError),
    SERVER(ServerError<T>),
    HTTP(HTTPError),
    ENCODING(EncodingError),
    NETWORK(String),
}

impl<T> ClientError<T> {
    pub fn simple(message: String) -> Self {
        ClientError::SIMPLE(SimpleError { message })
    }

    pub fn encoding(message: String) -> Self {
        ClientError::ENCODING(EncodingError { message })
    }

    pub fn server(error: T) -> Self {
        ClientError::SERVER(ServerError { error })
    }
}

impl<T> From<EncodingError> for ClientError<T> {
    fn from(value: EncodingError) -> Self {
        ClientError::ENCODING(value)
    }
}

impl<T> From<String> for ClientError<T> {
    fn from(value: String) -> Self {
        ClientError::simple(value)
    }
}

#[derive(Debug)]
pub struct SimpleError {
    pub message: String,
}

#[derive(Debug)]
pub struct ServerError<T> {
    pub error: T,
}

#[derive(Debug)]
pub struct HTTPError {
    pub code: u16,
    pub message: String,
}

#[derive(Debug)]
pub struct EncodingError {
    pub message: String,
}

impl EncodingError {
    pub fn new(message: String) -> Self {
        EncodingError { message }
    }
}

// pub trait ClientError {
//     fn get_message(&self) -> &str;
// }
//
// pub struct SimpleError {
//     pub message: str,
// }
//
// impl ClientError for SimpleError {
//     fn get_message(&self) -> String {
//         self.message.to_string()
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct GetInfoResponse {
    pub server_version: String,
    #[serde(deserialize_with = "deserialize_checksum256")]
    pub chain_id: Checksum256,
    pub head_block_num: u32,
    pub last_irreversible_block_num: u32,
    #[serde(deserialize_with = "deserialize_block_id")]
    pub last_irreversible_block_id: BlockId,
    #[serde(deserialize_with = "deserialize_block_id")]
    pub head_block_id: BlockId,
    #[serde(deserialize_with = "deserialize_timepoint")]
    pub head_block_time: TimePoint,
    #[serde(deserialize_with = "deserialize_name")]
    pub head_block_producer: Name,
    pub virtual_block_cpu_limit: u64,
    pub virtual_block_net_limit: u64,
    pub block_cpu_limit: u64,
    pub block_net_limit: u64,
    pub server_version_string: Option<String>,
    pub fork_db_head_block_num: Option<u32>,
    #[serde(deserialize_with = "deserialize_optional_block_id")]
    pub fork_db_head_block_id: Option<BlockId>,
    pub server_full_version_string: String,
    pub total_cpu_weight: String,
    pub total_net_weight: String,
    pub earliest_available_block_num: u32,
    pub last_irreversible_block_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TESTGetInfoResponse {
    pub server_version: String,
    #[serde(deserialize_with = "deserialize_checksum256")]
    pub chain_id: Checksum256,
    pub head_block_num: u32,
    pub last_irreversible_block_num: u32,
    #[serde(deserialize_with = "deserialize_block_id")]
    pub last_irreversible_block_id: BlockId,
    #[serde(deserialize_with = "deserialize_block_id")]
    pub head_block_id: BlockId,
    #[serde(deserialize_with = "deserialize_timepoint")]
    pub head_block_time: TimePoint,
    #[serde(deserialize_with = "deserialize_name")]
    pub head_block_producer: Name,
    pub virtual_block_cpu_limit: u64,
    pub virtual_block_net_limit: u64,
    pub block_cpu_limit: u64,
    pub block_net_limit: u64,
    pub server_version_string: Option<String>,
    pub fork_db_head_block_num: Option<u32>,
    #[serde(deserialize_with = "deserialize_optional_block_id")]
    pub fork_db_head_block_id: Option<BlockId>,
    pub server_full_version_string: String,
    pub total_cpu_weight: String,
    pub total_net_weight: String,
    pub earliest_available_block_num: u32,
    pub last_irreversible_block_time: String,
}

impl GetInfoResponse {
    pub fn get_transaction_header(&self, seconds_ahead: u32) -> TransactionHeader {
        let expiration = TimePointSec {
            // head_block_time.elapsed is microseconds, convert to seconds
            seconds: (self.head_block_time.elapsed / 1000 / 1000) as u32 + seconds_ahead,
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
            ref_block_prefix: prefix,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedTransactionReceipt {
    pub status: String,
    pub cpu_usage_us: u32,
    pub net_usage_words: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedTransaction {
    pub id: String,
    pub block_num: u64,
    pub block_time: String,
    pub receipt: ProcessedTransactionReceipt,
    pub elapsed: u64,
    pub except: Option<SendTransactionResponseError>,
    pub net_usage: u32,
    pub scheduled: bool,
    pub action_traces: Vec<ActionTrace>,
    pub account_ram_delta: Option<AccountRamDelta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponseExceptionStackContext {
    pub level: String,
    pub file: String,
    pub line: u32,
    pub method: String,
    pub hostname: String,
    pub thread_name: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponseExceptionStack {
    pub context: SendTransactionResponseExceptionStackContext,
    pub format: String,
    pub data: String, // TODO: create a type for this?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponseError {
    pub code: u32,
    pub name: String,
    pub message: String,
    pub stack: Vec<SendTransactionResponseExceptionStack>,
}

#[derive(Deserialize)]
pub struct ErrorResponse {
    pub error: SendTransactionResponseError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponse {
    pub transaction_id: String,
    pub processed: ProcessedTransaction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionTrace {
    pub action_ordinal: u32,
    pub creator_action_ordinal: u32,
    pub closest_unnotified_ancestor_action_ordinal: u32,
    pub receipt: ActionReceipt,
    #[serde(deserialize_with = "deserialize_name")]
    pub receiver: Name,
    pub act: Action,
    pub context_free: bool,
    pub elapsed: u64,
    pub console: String,
    pub trx_id: String,
    pub block_num: u64,
    pub block_time: String,
    pub producer_block_id: Option<String>,
    pub account_ram_deltas: Vec<AccountRamDelta>,
    pub except: Option<String>,
    pub error_code: Option<u32>,
    pub return_value_hex_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionReceipt {
    #[serde(deserialize_with = "deserialize_name")]
    pub receiver: Name,
    pub act_digest: String,
    pub global_sequence: u64,
    pub recv_sequence: u64,
    pub auth_sequence: Vec<AuthSequence>,
    pub code_sequence: u64,
    pub abi_sequence: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSequence {
    #[serde(deserialize_with = "deserialize_name")]
    pub account: Name,
    pub sequence: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountRamDelta {
    #[serde(deserialize_with = "deserialize_name")]
    pub account: Name,
    pub delta: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IndexPosition {
    PRIMARY,
    SECONDARY,
    TERTIARY,
    FOURTH,
    FIFTH,
    SIXTH,
    SEVENTH,
    EIGHTH,
    NINTH,
    TENTH,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TableIndexType {
    NAME(Name),
    UINT64(u64),
    UINT128(u128),
    FLOAT64(f64),
    CHECKSUM256(Checksum256),
    CHECKSUM160(Checksum160),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTableRowsParams {
    #[serde(deserialize_with = "deserialize_name")]
    pub code: Name,
    #[serde(deserialize_with = "deserialize_name")]
    pub table: Name,
    #[serde(deserialize_with = "deserialize_optional_name")]
    pub scope: Option<Name>,
    pub lower_bound: Option<TableIndexType>,
    pub upper_bound: Option<TableIndexType>,
    pub limit: Option<u32>,
    pub reverse: Option<bool>,
    pub index_position: Option<IndexPosition>,
    pub show_payer: Option<bool>,
}

impl GetTableRowsParams {
    pub fn to_json(&self) -> String {
        let mut req: HashMap<&str, Value> = HashMap::new();
        req.insert("json", Value::Bool(false));
        req.insert("code", Value::String(self.code.to_string()));
        req.insert("table", Value::String(self.table.to_string()));

        let scope = self.scope.unwrap_or(self.code);
        req.insert("scope", Value::String(scope.to_string()));

        json!(req).to_string()
    }
}

pub struct GetTableRowsResponse<T> {
    pub rows: Vec<T>,
    pub more: bool,
    pub ram_payers: Option<Vec<Name>>,
    pub next_key: Option<TableIndexType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountObject {
    account_name: Name,
    head_block_num: u32,
    #[serde(deserialize_with = "deserialize_timepoint")]
    head_block_time: TimePoint,
    privileged: bool,
    #[serde(deserialize_with = "deserialize_timepoint")]
    last_code_update: TimePoint,
    #[serde(deserialize_with = "deserialize_timepoint")]
    created: TimePoint,
    core_liquid_balance: Asset,
    ram_quota: i64,
    net_weight: i64,
    cpu_weight: i64,
    net_limit: AccountResourceLimit,
    cpu_limit: AccountResourceLimit,
    subjective_cpu_bill_limit: Option<AccountResourceLimit>,
    ram_usage: u64,
    permissions: Vec<AccountPermission>,
    total_resources: Option<AccountTotalResources>,
    self_delegated_bandwidth: Option<SelfDelegatedBandwidth>,
    refund_request: Option<AccountRefundRequest>,
    voter_info: VoterInfo,
    rex_info: AccountRexInfo,
    eosio_any_linked_actions: Option<Vec<AccountLinkedAction>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRexInfo {
    version: u32,
    owner: Name,
    vote_stakes: Asset,
    rex_balance: Asset,
    matured_rex: i64,
    rex_maturities: Vec<AccountRexInfoMaturities>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRexInfoMaturities {
    #[serde(deserialize_with = "deserialize_optional_timepoint")]
    key: Option<TimePoint>,
    value: Option<i64>,
    #[serde(deserialize_with = "deserialize_optional_timepoint")]
    first: Option<TimePoint>,
    second: Option<i64>,
}

//export class AccountResourceLimit extends Struct {
//     @Struct.field('int64') declare used: Int64
//     @Struct.field('int64') declare available: Int64
//     @Struct.field('int64') declare max: Int64
//     @Struct.field('time_point', {optional: true}) declare last_usage_update_time: TimePoint
//     @Struct.field('int64', {optional: true}) declare current_used: Int64
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResourceLimit {
    used: i64,
    available: i64,
    max: i64,
    #[serde(deserialize_with = "deserialize_optional_timepoint")]
    last_usage_update_time: Option<TimePoint>,
    current_used: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRefundRequest {
    #[serde(deserialize_with = "deserialize_name")]
    owner: Name,
    #[serde(deserialize_with = "deserialize_timepoint")]
    request_time: TimePoint,
    net_amount: Asset,
    cpu_amount: Asset,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceLimit {
    max: String,
    available: String,
    used: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountTotalResources {
    #[serde(deserialize_with = "deserialize_name")]
    owner: Name,
    net_weight: Asset,
    cpu_weight: Asset,
    ram_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelfDelegatedBandwidth {
    #[serde(deserialize_with = "deserialize_name")]
    from: Name,
    #[serde(deserialize_with = "deserialize_name")]
    to: Name,
    net_weight: Asset,
    cpu_weight: Asset,
}

//@Struct.type('account_permission')
// export class AccountPermission extends Struct {
//     @Struct.field('name') declare perm_name: Name
//     @Struct.field('name') declare parent: Name
//     @Struct.field(Authority) declare required_auth: Authority
//     @Struct.field(AccountLinkedAction, {optional: true, array: true})
//     declare linked_actions: AccountLinkedAction[]
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountPermission {
    #[serde(deserialize_with = "deserialize_name")]
    parent: Name,
    #[serde(deserialize_with = "deserialize_name")]
    perm_name: Name,
    //required_auth: Authority, TODO: Create Authority type
    linked_actions: AccountLinkedAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLinkedAction {
    #[serde(deserialize_with = "deserialize_name")]
    account: Name,
    #[serde(deserialize_with = "deserialize_optional_name")]
    action: Option<Name>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequiredAuth {
    threshold: u32,
    keys: Vec<Key>,
    accounts: Vec<Account>,
    waits: Vec<Wait>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wait {
    wait_sec: u32,
    weight: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    key: String,
    weight: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    weight: u16,
    permission: PermissionLevel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionLevel {
    #[serde(deserialize_with = "deserialize_name")]
    actor: Name,
    #[serde(deserialize_with = "deserialize_name")]
    permission: Name,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoterInfo {
    #[serde(deserialize_with = "deserialize_name")]
    owner: Name,
    #[serde(deserialize_with = "deserialize_name")]
    proxy: Name,
    producers: Option<Vec<Name>>,
    staked: Option<i64>,
    last_vote_weight: f64,
    proxied_vote_weight: f64,
    is_proxy: bool,
    flags1: Option<u32>,
    reserved2: u32,
    reserved3: String,
}

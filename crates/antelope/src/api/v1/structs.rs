use std::collections::HashMap;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use std::fmt;
use std::mem::discriminant;

use crate::chain::abi::ABI;
use crate::chain::public_key::PublicKey;
use crate::chain::signature::Signature;
use crate::chain::transaction::PackedTransaction;
use crate::chain::{
    action::{Action, PermissionLevel},
    asset::{deserialize_asset, deserialize_optional_asset, Asset},
    authority::Authority,
    block_id::{deserialize_block_id, deserialize_optional_block_id, BlockId},
    checksum::{deserialize_checksum256, Checksum160, Checksum256},
    name::{deserialize_name, deserialize_optional_name, deserialize_vec_name, Name},
    signature::deserialize_signature,
    time::{deserialize_optional_timepoint, deserialize_timepoint, TimePoint, TimePointSec},
    transaction::TransactionHeader,
    varint::VarUint32,
};
use tracing::info;

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
    #[serde(deserialize_with = "deserialize_number_or_string")]
    pub total_cpu_weight: String,
    #[serde(deserialize_with = "deserialize_number_or_string")]
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
pub struct ProcessedTransaction2 {
    pub id: String,
    pub block_num: u64,
    pub block_time: String,
    pub producer_block_id: Option<String>,
    pub receipt: Option<ProcessedTransactionReceipt>,
    pub elapsed: u64,
    pub net_usage: u32,
    pub scheduled: bool,
    pub action_traces: Vec<Value>,
    pub account_ram_delta: Option<Value>,
    pub except: Option<SendTransactionResponse2Error>,
    pub error_code: Option<String>,
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
pub struct SendTransactionResponse2ExceptionStack {
    pub context: SendTransactionResponseExceptionStackContext,
    pub format: String,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponseError {
    pub code: Option<u32>,
    pub name: String,
    pub what: String,
    pub stack: Option<Vec<SendTransactionResponseExceptionStack>>,
    pub details: Vec<SendTransactionResponseErrorDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponse2Error {
    pub code: Option<u32>,
    pub name: String,
    pub message: String,
    pub stack: Vec<SendTransactionResponse2ExceptionStack>,
}

impl SendTransactionResponseError {
    pub fn print_error(&self) {
        self.details.iter().for_each(|d| info!("{:?}", d));
    }

    pub fn get_stack(&self) -> String {
        self.stack
            .as_ref()
            .map(|s| {
                s.iter()
                    .map(|s| s.format.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponseErrorDetails {
    pub message: String,
    pub file: String,
    pub line_number: u32,
    pub method: String,
}

pub struct SendTransaction2Options {
    pub return_failure_trace: bool,
    pub retry_trx: bool,
    pub retry_trx_num_blocks: u32,
}

#[derive(Serialize)]
pub struct SendTransaction2Request {
    pub return_failure_trace: bool,
    pub retry_trx: bool,
    pub retry_trx_num_blocks: u32,
    pub transaction: Value,
}

impl SendTransaction2Request {
    pub fn build(
        trx: PackedTransaction,
        options: Option<SendTransaction2Options>,
    ) -> SendTransaction2Request {
        let opts = options.unwrap_or(SendTransaction2Options {
            return_failure_trace: true,
            retry_trx: false,
            retry_trx_num_blocks: 0,
        });

        SendTransaction2Request {
            return_failure_trace: opts.return_failure_trace,
            retry_trx: opts.retry_trx,
            retry_trx_num_blocks: opts.retry_trx_num_blocks,
            transaction: trx.to_json(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub error: SendTransactionResponseError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse2 {
    pub code: u16,
    pub message: String,
    pub error: SendTransactionResponse2Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponse {
    pub transaction_id: String,
    pub processed: ProcessedTransaction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransaction2Response {
    pub transaction_id: String,
    pub processed: ProcessedTransaction2,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionState {
    LocallyApplied,
    ForkedOut,
    InBlock,
    Irreversible,
    Failed,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionStatusResponse {
    pub state: TransactionState,
    pub block_number: Option<u32>,
    #[serde(deserialize_with = "deserialize_optional_block_id", default)]
    pub block_id: Option<BlockId>,
    #[serde(deserialize_with = "deserialize_optional_timepoint", default)]
    pub block_timestamp: Option<TimePoint>,
    #[serde(deserialize_with = "deserialize_optional_timepoint", default)]
    pub expiration: Option<TimePoint>,
    pub head_number: u32,
    #[serde(deserialize_with = "deserialize_block_id")]
    pub head_id: BlockId,
    #[serde(deserialize_with = "deserialize_timepoint")]
    pub head_timestamp: TimePoint,
    pub irreversible_number: u32,
    #[serde(deserialize_with = "deserialize_block_id")]
    pub irreversible_id: BlockId,
    #[serde(deserialize_with = "deserialize_timepoint")]
    pub irreversible_timestamp: TimePoint,
    #[serde(deserialize_with = "deserialize_block_id")]
    pub earliest_tracked_block_id: BlockId,
    pub earliest_tracked_block_number: u32,
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
    #[serde(deserialize_with = "deserialize_u64_from_string_or_u64")]
    pub elapsed: u64,
    pub console: String,
    pub trx_id: String,
    #[serde(deserialize_with = "deserialize_u64_from_string_or_u64")]
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
    #[serde(deserialize_with = "deserialize_u64_from_string_or_u64")]
    pub global_sequence: u64,
    #[serde(deserialize_with = "deserialize_u64_from_string_or_u64")]
    pub recv_sequence: u64,
    pub auth_sequence: Vec<AuthSequence>,
    #[serde(deserialize_with = "deserialize_u64_from_string_or_u64")]
    pub code_sequence: u64,
    #[serde(deserialize_with = "deserialize_u64_from_string_or_u64")]
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

impl IndexPosition {
    pub fn to_json(&self) -> Value {
        match self {
            IndexPosition::PRIMARY => Value::String("primary".to_string()),
            IndexPosition::SECONDARY => Value::String("secondary".to_string()),
            IndexPosition::TERTIARY => Value::String("tertiary".to_string()),
            IndexPosition::FOURTH => Value::String("fourth".to_string()),
            IndexPosition::FIFTH => Value::String("fifth".to_string()),
            IndexPosition::SIXTH => Value::String("sixth".to_string()),
            IndexPosition::SEVENTH => Value::String("seventh".to_string()),
            IndexPosition::EIGHTH => Value::String("eighth".to_string()),
            IndexPosition::NINTH => Value::String("ninth".to_string()),
            IndexPosition::TENTH => Value::String("tenth".to_string()),
        }
    }
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

impl TableIndexType {
    pub fn to_json(&self) -> Value {
        match self {
            TableIndexType::NAME(name) => json!(name.to_string()),
            TableIndexType::UINT64(value) => json!(value.to_string()),
            TableIndexType::UINT128(value) => json!(value.to_string()),
            TableIndexType::FLOAT64(value) => json!(value.to_string()),
            TableIndexType::CHECKSUM256(value) => json!(value.to_index()),
            TableIndexType::CHECKSUM160(value) => json!(value.as_string()),
        }
    }

    pub fn get_key_type(&self) -> Value {
        match self {
            TableIndexType::NAME(_) => Value::String("name".to_string()),
            TableIndexType::UINT64(_) => Value::String("i64".to_string()),
            TableIndexType::UINT128(_) => Value::String("i128".to_string()),
            TableIndexType::FLOAT64(_) => Value::String("float64".to_string()),
            TableIndexType::CHECKSUM256(_) => Value::String("sha256".to_string()),
            TableIndexType::CHECKSUM160(_) => Value::String("ripemd160".to_string()),
        }
    }
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
        req.insert("code", Value::String(self.code.to_string()));
        req.insert("table", Value::String(self.table.to_string()));

        let scope = self.scope.unwrap_or(self.code);
        req.insert("scope", Value::String(scope.to_string()));

        req.insert("json", Value::Bool(false));

        if let Some(limit) = &self.limit {
            req.insert("limit", Value::String(limit.to_string()));
        }

        if let Some(reverse) = &self.reverse {
            req.insert("reverse", Value::Bool(*reverse));
        }

        if self.lower_bound.is_some() || self.upper_bound.is_some() {
            if self.upper_bound.is_none() {
                let lower = self.lower_bound.as_ref().unwrap();
                req.insert("key_type", lower.get_key_type());
                req.insert("lower_bound", lower.to_json());
            } else if self.lower_bound.is_none() {
                let upper = self.upper_bound.as_ref().unwrap();
                req.insert("key_type", upper.get_key_type());
                req.insert("upper_bound", upper.to_json());
            } else {
                let lower = self.lower_bound.as_ref().unwrap();
                let upper = self.upper_bound.as_ref().unwrap();
                if discriminant(lower) != discriminant(upper) {
                    panic!("lower_bound and upper_bound must be of the same type");
                }
                req.insert("key_type", lower.get_key_type());
                req.insert("lower_bound", lower.to_json());
                req.insert("upper_bound", upper.to_json());
            }

            if let Some(index_position) = &self.index_position {
                req.insert("index_position", index_position.to_json());
            }
        }

        json!(req).to_string()
    }
}

#[derive(Debug)]
pub struct GetTableRowsResponse<T> {
    pub rows: Vec<T>,
    pub more: bool,
    pub ram_payers: Option<Vec<Name>>,
    pub next_key: Option<TableIndexType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountObject {
    #[serde(deserialize_with = "deserialize_name")]
    pub account_name: Name,
    pub head_block_num: u32,
    #[serde(deserialize_with = "deserialize_timepoint")]
    pub head_block_time: TimePoint,
    pub privileged: bool,
    #[serde(deserialize_with = "deserialize_timepoint")]
    pub last_code_update: TimePoint,
    #[serde(deserialize_with = "deserialize_timepoint")]
    pub created: TimePoint,
    #[serde(
        deserialize_with = "deserialize_optional_asset",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub core_liquid_balance: Option<Asset>,
    pub ram_quota: i64,
    pub net_weight: i64,
    pub cpu_weight: i64,
    pub net_limit: AccountResourceLimit,
    pub cpu_limit: AccountResourceLimit,
    pub ram_usage: u64,
    pub permissions: Vec<AccountPermission>,
    pub total_resources: Option<AccountTotalResources>,
    pub self_delegated_bandwidth: Option<SelfDelegatedBandwidth>,
    pub refund_request: Option<AccountRefundRequest>,
    pub voter_info: Option<AccountVoterInfo>,
    pub rex_info: Option<AccountRexInfo>,
    pub subjective_cpu_bill_limit: Option<AccountResourceLimit>,
    pub eosio_any_linked_actions: Option<Vec<AccountLinkedAction>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRexInfo {
    version: u32,
    #[serde(deserialize_with = "deserialize_name")]
    owner: Name,
    #[serde(deserialize_with = "deserialize_asset")]
    vote_stake: Asset,
    #[serde(deserialize_with = "deserialize_asset")]
    rex_balance: Asset,
    #[serde(deserialize_with = "deserialize_i64_from_string_or_i64")]
    matured_rex: i64,
    rex_maturities: Vec<AccountRexInfoMaturities>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRexInfoMaturities {
    #[serde(
        deserialize_with = "deserialize_optional_timepoint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    key: Option<TimePoint>,
    #[serde(
        deserialize_with = "deserialize_optional_i64_from_string",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    value: Option<i64>,
    #[serde(
        deserialize_with = "deserialize_optional_timepoint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    first: Option<TimePoint>,
    #[serde(
        deserialize_with = "deserialize_optional_i64_from_string",
        default,
        skip_serializing_if = "Option::is_none"
    )]
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
    #[serde(deserialize_with = "deserialize_i64_from_string_or_i64")]
    available: i64,
    #[serde(deserialize_with = "deserialize_i64_from_string_or_i64")]
    max: i64,
    #[serde(
        deserialize_with = "deserialize_optional_timepoint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    last_usage_update_time: Option<TimePoint>,
    #[serde(
        deserialize_with = "deserialize_optional_i64_from_string",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    current_used: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRefundRequest {
    #[serde(deserialize_with = "deserialize_name")]
    owner: Name,
    #[serde(deserialize_with = "deserialize_timepoint")]
    request_time: TimePoint,
    #[serde(deserialize_with = "deserialize_asset")]
    net_amount: Asset,
    #[serde(deserialize_with = "deserialize_asset")]
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
    #[serde(deserialize_with = "deserialize_asset")]
    net_weight: Asset,
    #[serde(deserialize_with = "deserialize_asset")]
    cpu_weight: Asset,
    ram_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelfDelegatedBandwidth {
    #[serde(deserialize_with = "deserialize_name")]
    from: Name,
    #[serde(deserialize_with = "deserialize_name")]
    to: Name,
    #[serde(deserialize_with = "deserialize_asset")]
    net_weight: Asset,
    #[serde(deserialize_with = "deserialize_asset")]
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
    required_auth: Authority,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub linked_actions: Option<Vec<AccountLinkedAction>>,
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
pub struct AccountVoterInfo {
    #[serde(deserialize_with = "deserialize_name")]
    owner: Name,
    #[serde(deserialize_with = "deserialize_name")]
    proxy: Name,
    #[serde(deserialize_with = "deserialize_vec_name")]
    producers: Vec<Name>,
    staked: Option<i64>,
    last_stake: Option<i64>,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    last_vote_weight: f64,
    #[serde(deserialize_with = "deserialize_f64_from_string")]
    proxied_vote_weight: f64,
    #[serde(deserialize_with = "deserialize_bool_from_number")]
    is_proxy: bool,
    flags1: Option<u32>,
    reserved2: u32,
    reserved3: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ABIResponse {
    pub account_name: String,
    pub abi: ABI,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GetBlockResponse {
    #[serde(rename = "timestamp")]
    #[serde(deserialize_with = "deserialize_timepoint")]
    pub time_point: TimePoint,
    #[serde(deserialize_with = "deserialize_name")]
    pub producer: Name,
    pub confirmed: u16,
    #[serde(deserialize_with = "deserialize_block_id")]
    pub previous: BlockId,
    #[serde(deserialize_with = "deserialize_checksum256")]
    pub transaction_mroot: Checksum256,
    #[serde(deserialize_with = "deserialize_checksum256")]
    pub action_mroot: Checksum256,
    pub schedule_version: u32,
    pub new_producers: Option<NewProducers>,
    pub header_extensions: Option<HeaderExtension>,
    // pub new_protocol_features: any,
    #[serde(deserialize_with = "deserialize_signature")]
    pub producer_signature: Signature,
    pub transactions: Vec<GetBlockResponseTransactionReceipt>,
    pub block_extensions: Option<Vec<BlockExtension>>,
    #[serde(deserialize_with = "deserialize_block_id")]
    pub id: BlockId,
    pub block_num: u32,
    pub ref_block_prefix: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewProducers {
    pub version: u32,
    pub producers: Vec<NewProducersEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewProducersEntry {
    pub producer_name: Name,
    pub block_signing_key: PublicKey,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HeaderExtension {
    pub r#type: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GetBlockResponseTransactionReceipt {
    pub trx: String, //TODO: Implement TxVarient
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrxVariant {
    pub id: Checksum256,
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BlockExtension {
    pub r#type: u16,
    pub data: Vec<u8>,
}

// impl TrxVariant {
//     pub fn from(data: serde_json::Value) -> Result<Self, Box<dyn std::error::Error>> {
//         let id;
//         let extra: HashMap<String, serde_json::Value>;
//
//         match data {
//             Value::String(s) => {
//                 id = Checksum256::from(&s)?;
//                 extra = HashMap::new();
//             },
//             Value::Object(obj) => {
//                 let id_str = obj.get("id")
//                     .ok_or("id field missing")?
//                     .as_str()
//                     .ok_or("id field is not a string")?;
//                 id = Checksum256::from(id_str)?;
//                 extra = obj;
//             },
//             _ => return Err("Unsupported data type".into()),
//         }
//
//         Ok(TrxVariant { id, extra })
//     }
//
//     pub fn transaction(&self) -> Option<Transaction> {
//         self.extra.get("packed_trx").and_then(|packed_trx| {
//             match self.extra.get("compression").and_then(|c| c.as_str()) {
//                 Some("zlib") => {
//                     // Decompress using zlib and decode
//                     let inflated = decompress_zlib(&packed_trx);
//                     Some(packer::pack(&inflated, Transaction))
//                 },
//                 Some("none") => {
//                     // Directly decode
//                     Some(packer::pack(packed_trx, Transaction))
//                 },
//                 _ => None,
//             }
//         })
//     }
//
//     pub fn signatures(&self) -> Option<Vec<Signature>> {
//         self.extra.get("signatures").and_then(|sigs| {
//             sigs.as_array().map(|arr| {
//                 arr.iter().filter_map(|sig| Signature::from(sig)).collect()
//             })
//         })
//     }
//
//     pub fn equals(&self, other: &TrxVariant) -> bool {
//         self.id == other.id
//     }
//
//     pub fn to_json(&self) -> Value {
//         json!(self.id)
//     }
// }

fn deserialize_number_or_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(num) => Ok(num.to_string()),
        Value::String(s) => Ok(s),
        _ => Err(serde::de::Error::custom("expected a number or a string")),
    }
}

fn deserialize_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringToF64Visitor;

    impl Visitor<'_> for StringToF64Visitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string that can be parsed into a f64")
        }

        fn visit_str<E>(self, value: &str) -> Result<f64, E>
        where
            E: de::Error,
        {
            value.parse::<f64>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_str(StringToF64Visitor)
}

fn deserialize_bool_from_number<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct NumberToBoolVisitor;

    impl Visitor<'_> for NumberToBoolVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number representing a boolean (0 or 1)")
        }

        fn visit_u64<E>(self, value: u64) -> Result<bool, E>
        where
            E: de::Error,
        {
            match value {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(E::custom("expected 0 or 1 for boolean")),
            }
        }
    }

    deserializer.deserialize_any(NumberToBoolVisitor)
}

fn deserialize_u64_from_string_or_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct U64OrStringVisitor;

    impl serde::de::Visitor<'_> for U64OrStringVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer or a string representation of an integer")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse::<u64>().map_err(E::custom)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            u64::try_from(value).map_err(|_| E::custom("u64 value too large for i64"))
        }
    }

    deserializer.deserialize_any(U64OrStringVisitor)
}

fn deserialize_i64_from_string_or_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    struct I64OrStringVisitor;

    impl serde::de::Visitor<'_> for I64OrStringVisitor {
        type Value = i64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer or a string representation of an integer")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse::<i64>().map_err(E::custom)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            i64::try_from(value).map_err(|_| E::custom("u64 value too large for i64"))
        }
    }

    deserializer.deserialize_any(I64OrStringVisitor)
}

fn deserialize_optional_i64_from_string<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrI64Visitor;

    impl Visitor<'_> for StringOrI64Visitor {
        type Value = Option<i64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str(
                "a string representing a 64-bit signed integer, an actual integer, or null",
            )
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            value.parse::<i64>().map(Some).map_err(de::Error::custom)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            if value <= i64::MAX as u64 {
                Ok(Some(value as i64))
            } else {
                Err(de::Error::custom("u64 value too large for i64"))
            }
        }

        fn visit_none<E>(self) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Option<i64>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrI64Visitor)
}

#[cfg(test)]
mod tests {
    use crate::api::v1::structs::AccountObject;

    #[test]
    fn deserialize_simple_account() {
        // This simple account response doesn't contain details about `total_resources`, `self_delegated_bandwidth`,
        // `refund_request`, `voter_info`, and `rex_info`.
        // Such fields are null.
        let simple_account_json = r#"
        {
            "account_name": "eosio",
            "head_block_num": 56,
            "head_block_time": "2024-08-29T15:27:24.500",
            "privileged": true,
            "last_code_update": "2024-08-29T14:06:02.000",
            "created": "2019-08-07T12:00:00.000",
            "core_liquid_balance": "99986000.0000 TLOS",
            "ram_quota": -1,
            "net_weight": -1,
            "cpu_weight": -1,
            "net_limit": {
                "used": -1,
                "available": -1,
                "max": -1,
                "last_usage_update_time": "2024-08-29T15:27:25.000",
                "current_used": -1
            },
            "cpu_limit": {
                "used": -1,
                "available": -1,
                "max": -1,
                "last_usage_update_time": "2024-08-29T15:27:25.000",
                "current_used": -1
            },
            "ram_usage": 3485037,
            "permissions": [
                {
                    "perm_name": "active",
                    "parent": "owner",
                    "required_auth": {
                        "threshold": 1,
                        "keys": [
                            {
                                "key": "EOS5uHeBsURAT6bBXNtvwKtWaiDSDJSdSmc96rHVws5M1qqVCkAm6",
                                "weight": 1
                            }
                        ],
                        "accounts": [],
                        "waits": []
                    },
                    "linked_actions": []
                },
                {
                    "perm_name": "owner",
                    "parent": "",
                    "required_auth": {
                        "threshold": 1,
                        "keys": [
                            {
                                "key": "EOS5uHeBsURAT6bBXNtvwKtWaiDSDJSdSmc96rHVws5M1qqVCkAm6",
                                "weight": 1
                            }
                        ],
                        "accounts": [],
                        "waits": []
                    },
                    "linked_actions": []
                }
            ],
            "total_resources": null,
            "self_delegated_bandwidth": null,
            "refund_request": null,
            "voter_info": null,
            "rex_info": null,
            "subjective_cpu_bill_limit": {
                "used": 0,
                "available": 0,
                "max": 0,
                "last_usage_update_time": "2000-01-01T00:00:00.000",
                "current_used": 0
            },
            "eosio_any_linked_actions": []
        }
        "#;

        let res = serde_json::from_str::<AccountObject>(simple_account_json).unwrap();
        println!("{:#?}", res);
    }

    #[test]
    fn deserialize_detailed_account() {
        // This detailed account response contains additional fields compared to the simple account (see test above),
        // in particular `total_resources`, `self_delegated_bandwidth`, `refund_request`, `voter_info`, and `rex_info`.
        let detailed_account_json = r#"
        {
            "account_name": "alice",
            "head_block_num": 56,
            "head_block_time": "2024-08-29T15:27:24.500",
            "privileged": false,
            "last_code_update": "1970-01-01T00:00:00.000",
            "created": "2024-08-29T14:06:02.000",
            "core_liquid_balance": "100.0000 TLOS",
            "ram_quota": 610645714,
            "net_weight": 10000000,
            "cpu_weight": 10000000,
            "net_limit": {
                "used": 0,
                "available": "95719449600",
                "max": "95719449600",
                "last_usage_update_time": "2024-08-29T14:06:02.000",
                "current_used": 0
            },
            "cpu_limit": {
                "used": 0,
                "available": "364783305600",
                "max": "364783305600",
                "last_usage_update_time": "2024-08-29T14:06:02.000",
                "current_used": 0
            },
            "ram_usage": 3566,
            "permissions": [
                {
                    "perm_name": "active",
                    "parent": "owner",
                    "required_auth": {
                        "threshold": 1,
                        "keys": [
                            {
                                "key": "EOS77jzbmLuakAHpm2Q5ew8EL7Y7gGkfSzqJCmCNDDXWEsBP3xnDc",
                                "weight": 1
                            }
                        ],
                        "accounts": [],
                        "waits": []
                    },
                    "linked_actions": []
                },
                {
                    "perm_name": "owner",
                    "parent": "",
                    "required_auth": {
                        "threshold": 1,
                        "keys": [
                            {
                                "key": "EOS77jzbmLuakAHpm2Q5ew8EL7Y7gGkfSzqJCmCNDDXWEsBP3xnDc",
                                "weight": 1
                            }
                        ],
                        "accounts": [],
                        "waits": []
                    },
                    "linked_actions": []
                }
            ],
            "total_resources": {
                "owner": "alice",
                "net_weight": "1000.0000 TLOS",
                "cpu_weight": "1000.0000 TLOS",
                "ram_bytes": 610644314
            },
            "self_delegated_bandwidth": {
                "from": "alice",
                "to": "alice",
                "net_weight": "1000.0000 TLOS",
                "cpu_weight": "1000.0000 TLOS"
            },
            "refund_request": null,
            "voter_info": {
                "owner": "alice",
                "proxy": "",
                "producers": [],
                "staked": 20000000,
                "last_stake": 0,
                "last_vote_weight": "0.00000000000000000",
                "proxied_vote_weight": "0.00000000000000000",
                "is_proxy": 0,
                "flags1": 0,
                "reserved2": 0,
                "reserved3": "0 "
            },
            "rex_info": null,
            "subjective_cpu_bill_limit": {
                "used": 0,
                "available": 0,
                "max": 0,
                "last_usage_update_time": "2000-01-01T00:00:00.000",
                "current_used": 0
            },
            "eosio_any_linked_actions": []
        }
        "#;

        let res = serde_json::from_str::<AccountObject>(detailed_account_json).unwrap();
        println!("{:#?}", res);
    }

    #[test]
    fn deserialize_account_without_core_liquid_balance() {
        // This simple account response doesn't contain details about `total_resources`, `self_delegated_bandwidth`,
        // `refund_request`, `voter_info`, and `rex_info`.
        // Such fields are null.
        let detailed_account_json = r#"
        {
            "account_name": "alice",
            "head_block_num": 56,
            "head_block_time": "2024-08-29T15:46:42.000",
            "privileged": false,
            "last_code_update": "1970-01-01T00:00:00.000",
            "created": "2024-08-29T14:06:02.000",
            "ram_quota": 610645714,
            "net_weight": 10000000,
            "cpu_weight": 10000000,
            "net_limit": {
                "used": 0,
                "available": "95719449600",
                "max": "95719449600",
                "last_usage_update_time": "2024-08-29T14:06:02.000",
                "current_used": 0
            },
            "cpu_limit": {
                "used": 0,
                "available": "364783305600",
                "max": "364783305600",
                "last_usage_update_time": "2024-08-29T14:06:02.000",
                "current_used": 0
            },
            "ram_usage": 3566,
            "permissions": [
                {
                    "perm_name": "active",
                    "parent": "owner",
                    "required_auth": {
                        "threshold": 1,
                        "keys": [
                            {
                                "key": "EOS77jzbmLuakAHpm2Q5ew8EL7Y7gGkfSzqJCmCNDDXWEsBP3xnDc",
                                "weight": 1
                            }
                        ],
                        "accounts": [],
                        "waits": []
                    },
                    "linked_actions": []
                },
                {
                    "perm_name": "owner",
                    "parent": "",
                    "required_auth": {
                        "threshold": 1,
                        "keys": [
                            {
                                "key": "EOS77jzbmLuakAHpm2Q5ew8EL7Y7gGkfSzqJCmCNDDXWEsBP3xnDc",
                                "weight": 1
                            }
                        ],
                        "accounts": [],
                        "waits": []
                    },
                    "linked_actions": []
                }
            ],
            "total_resources": {
                "owner": "alice",
                "net_weight": "1000.0000 TLOS",
                "cpu_weight": "1000.0000 TLOS",
                "ram_bytes": 610644314
            },
            "self_delegated_bandwidth": {
                "from": "alice",
                "to": "alice",
                "net_weight": "1000.0000 TLOS",
                "cpu_weight": "1000.0000 TLOS"
            },
            "refund_request": null,
            "voter_info": {
                "owner": "alice",
                "proxy": "",
                "producers": [],
                "staked": 20000000,
                "last_stake": 0,
                "last_vote_weight": "0.00000000000000000",
                "proxied_vote_weight": "0.00000000000000000",
                "is_proxy": 0,
                "flags1": 0,
                "reserved2": 0,
                "reserved3": "0 "
            },
            "rex_info": null,
            "subjective_cpu_bill_limit": {
                "used": 0,
                "available": 0,
                "max": 0,
                "last_usage_update_time": "2000-01-01T00:00:00.000",
                "current_used": 0
            },
            "eosio_any_linked_actions": []
        }
        "#;

        let res = serde_json::from_str::<AccountObject>(detailed_account_json).unwrap();
        println!("{:#?}", res);
    }
}

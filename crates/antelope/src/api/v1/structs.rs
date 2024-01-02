use crate::chain::{
    checksum::Checksum256,
    name::Name,
    time::{TimePoint, TimePointSec},
    block_id::BlockId,
    transaction::TransactionHeader,
    varint::VarUint32,
};

#[derive(Debug)]
pub enum ClientError<T> {
    SIMPLE(SimpleError),
    SERVER(T),
    HTTP(HTTPError),
    ENCODING(EncodingError)
}

impl<T> ClientError<T> {
    pub fn simple(message: String) -> Self {
        ClientError::SIMPLE(SimpleError { message })
    }

    pub fn encoding(message: String) -> Self {
        ClientError::ENCODING(EncodingError { message })
    }

    pub fn server(server_error: T) -> Self {
        ClientError::SERVER(server_error)
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
    pub message: String
}

#[derive(Debug)]
pub struct ServerError<T> {
    error: T
}

#[derive(Debug)]
pub struct HTTPError {
    pub code: u16,
    pub message: String
}

#[derive(Debug)]
pub struct EncodingError {
    pub message: String
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

pub struct GetInfoResponse {
    pub server_version: String,
    pub chain_id: Checksum256,
    pub head_block_num: u32,
    pub last_irreversible_block_num: u32,
    pub last_irreversible_block_id: BlockId,
    pub head_block_id: BlockId,
    pub head_block_time: TimePoint,
    pub head_block_producer: Name,
    pub virtual_block_cpu_limit: u64,
    pub virtual_block_net_limit: u64,
    pub block_cpu_limit: u64,
    pub block_net_limit: u64,
    pub server_version_string: Option<String>,
    pub fork_db_head_block_num: Option<u32>,
    pub fork_db_head_block_id: Option<BlockId>
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

pub struct ProcessedTransactionReceipt {
    pub status: String,
    pub cpu_usage_us: u32,
    pub net_usage_words: u32
}

pub struct ProcessedTransaction {
    pub id: String,
    pub block_num: u64,
    pub block_time: String,
    pub receipt: ProcessedTransactionReceipt,
    pub elapsed: u64,
    pub except: Option<SendTransactionResponseException>,
    pub net_usage: u32,
    pub scheduled: bool,
    pub action_traces: String, // TODO: create a type for this?
    pub account_ram_delta: String // TODO: create a type for this?

}

pub struct SendTransactionResponseExceptionStackContext {
    pub level: String,
    pub file: String,
    pub line: u32,
    pub method: String,
    pub hostname: String,
    pub thread_name: String,
    pub timestamp: String
}

pub struct SendTransactionResponseExceptionStack {
    pub context: SendTransactionResponseExceptionStackContext,
    pub format: String,
    pub data: String // TODO: create a type for this?
}

pub struct SendTransactionResponseException {
    pub code: u32,
    pub name: String,
    pub message: String,
    pub stack: Vec<SendTransactionResponseExceptionStack>
}

pub struct SendTransactionResponse {
    pub transaction_id: String,
    pub processed: ProcessedTransaction
}


#[derive(Debug)]
pub struct SendTransactionError {
    pub message: String
}
//
// impl From<dyn ClientError> for SendTransactionError {
//     fn from(value: ClientError) -> Self {
//         Self {
//             message: value.message
//         }
//     }
// }
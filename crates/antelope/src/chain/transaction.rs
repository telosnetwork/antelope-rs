use crate::chain::checksum::Checksum256;
use crate::chain::signature::Signature;
use crate::chain::{
    action::Action, time::TimePointSec, varint::VarUint32, Decoder, Encoder, Packer,
};
use crate::util::{bytes_to_hex, zlib_compress};
use antelope_client_macros::StructPacker;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Default, StructPacker, Serialize, Deserialize)]
pub struct TransactionExtension {
    pub ty: u16,
    pub data: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq, Default, StructPacker, Serialize, Deserialize)]
pub struct TransactionHeader {
    pub expiration: TimePointSec,
    pub ref_block_num: u16,
    pub ref_block_prefix: u32,
    pub max_net_usage_words: VarUint32,
    pub max_cpu_usage_ms: u8,
    pub delay_sec: VarUint32,
}

#[derive(Clone, Eq, PartialEq, Default, StructPacker, Serialize, Deserialize)]
pub struct Transaction {
    pub header: TransactionHeader,
    pub context_free_actions: Vec<Action>,
    pub actions: Vec<Action>,
    pub extension: Vec<TransactionExtension>,
}

impl Transaction {
    pub fn id(&self) -> Vec<u8> {
        Checksum256::hash(Encoder::pack(self)).data.to_vec()
    }

    pub fn signing_data(&self, chain_id: &[u8]) -> Vec<u8> {
        let mut bytes = chain_id.to_vec();
        let encoded = &mut Encoder::pack(self);
        bytes.append(encoded);
        bytes.append(&mut vec![0u8; 32]);
        bytes
    }

    pub fn signing_digest(&self, chain_id: &[u8]) -> Vec<u8> {
        Checksum256::hash(self.signing_data(chain_id)).data.to_vec()
    }
}

#[derive(Clone, Eq, PartialEq, Default, StructPacker, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signatures: Vec<Signature>,
    pub context_free_data: Vec<Vec<u8>>,
}

#[derive(PartialEq)]
pub enum CompressionType {
    ZLIB,
    NONE,
}

impl CompressionType {
    pub fn index(&self) -> usize {
        match self {
            CompressionType::NONE => 0,
            CompressionType::ZLIB => 1,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Default, StructPacker, Serialize, Deserialize)]
pub struct PackedTransaction {
    signatures: Vec<Signature>,
    compression: Option<u8>,
    packed_context_free_data: Vec<u8>,
    packed_transaction: Vec<u8>,
}

impl PackedTransaction {
    pub fn from_signed(
        signed: SignedTransaction,
        compression: CompressionType,
    ) -> Result<Self, String> {
        let mut packed_transaction = Encoder::pack(&signed.transaction);
        let mut packed_context_free_data = Encoder::pack(&signed.context_free_data);
        if compression == CompressionType::ZLIB {
            packed_transaction = zlib_compress(packed_transaction.as_slice())?;
            packed_context_free_data = zlib_compress(packed_context_free_data.as_slice())?;
        }

        Ok(Self {
            signatures: signed.signatures,
            compression: Some(compression.index() as u8),
            packed_transaction,
            packed_context_free_data,
        })
    }

    pub fn to_json(&self) -> String {
        let mut trx: HashMap<&str, Value> = HashMap::new();
        let signatures: Vec<String> = self.signatures.iter().map(|sig| sig.to_string()).collect();
        trx.insert("signatures", json!(signatures));
        if self.compression.is_some() {
            trx.insert(
                "compression",
                Value::Number(self.compression.unwrap().into()),
            );
        }
        trx.insert(
            "packed_context_free_data",
            Value::String(bytes_to_hex(&self.packed_context_free_data)),
        );
        trx.insert(
            "packed_trx",
            Value::String(bytes_to_hex(&self.packed_transaction)),
        );
        json!(trx).to_string()
    }
}

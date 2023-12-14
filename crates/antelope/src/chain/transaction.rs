use rust_chain::{ Packer, Encoder, Decoder };
use antelope_macros::StructPacker;
use crate::api::v1::structs::GetInfoResponse;
use crate::chain::signature::Signature;
use crate::chain::{Action, TimePointSec, VarUint32};
use crate::chain::checksum::Checksum256;

#[derive(Clone, Eq, PartialEq, Default, StructPacker)]
pub struct TransactionExtension {
    pub ty:     u16,
    pub data:   Vec<u8>,
}

#[derive(Clone, Eq, PartialEq, Default, StructPacker)]
pub struct TransactionHeader {
    pub expiration:             TimePointSec,
    pub ref_block_num:          u16,
    pub ref_block_prefix:       u32,
    pub max_net_usage_words:    VarUint32,
    pub max_cpu_usage_ms:       u8,
    pub delay_sec:              VarUint32,
}

#[derive(Clone, Eq, PartialEq, Default, StructPacker)]
pub struct Transaction {
    pub header:                 TransactionHeader,
    pub context_free_actions:   Vec<Action>,
    pub actions:                Vec<Action>,
    pub extension:              Vec<rust_chain::TransactionExtension>,
}

impl Transaction {

    pub fn signing_data(&self, chain_id: &Vec<u8>) -> Vec<u8> {
        let mut bytes = chain_id.to_vec();
        let encoded = &mut Encoder::pack(self);
        bytes.append(encoded);
        bytes.append(&mut vec![0u8; 32]);
        bytes
    }

    pub fn signing_digest(&self, chain_id: &Vec<u8>) -> Vec<u8> {
        return Checksum256::hash(self.signing_data(chain_id)).to_bytes();
    }
}

#[derive(Clone, Eq, PartialEq, Default, StructPacker)]
pub struct SignedTransaction {
    transaction:        Transaction,
    signatures:         Vec<Signature>,
    context_free_data:  Vec<Vec<u8>>
}

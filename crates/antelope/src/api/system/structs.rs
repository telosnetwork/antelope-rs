use antelope_client_macros::StructPacker;
use crate::chain::{Encoder, Decoder, Packer};
use crate::chain::asset::Asset;
use crate::chain::authority::Authority;
use crate::chain::binary_extension::BinaryExtension;
use crate::chain::name::Name;
use crate::chain::public_key::PublicKey;

pub struct CreateAccountParams {
    pub name: Name,
    pub creator: Name,
    pub owner: Authority,
    pub active: Authority,
    pub ram_bytes: u64,
    pub stake_net: u64,
    pub stake_cpu: u64,
    pub transfer: bool,
}

impl CreateAccountParams {
    pub fn testing(
        name: Name,
        creator: Name,
        public_key: PublicKey,
    ) -> Self {
        let owner = Authority::new_single_key(public_key);
        let active = owner.clone();
        CreateAccountParams {
            name,
            creator,
            owner,
            active,
            ram_bytes: 1_048_576,
            stake_net: 10_000,
            stake_cpu: 10_000,
            transfer: true,
        }
    }
}

#[derive(Debug, Clone, StructPacker)]
pub struct TransferAction {
    pub from: Name,
    pub to: Name,
    pub quantity: Asset,
    pub memo: String,
}

#[derive(Debug, Clone, StructPacker)]
pub struct NewAccountAction {
    pub creator: Name,
    pub name: Name,
    pub owner: Authority,
    pub active: Authority,
}

#[derive(Debug, Clone, StructPacker)]
pub struct BuyRamBytesAction {
    pub payer: Name,
    pub receiver: Name,
    pub bytes: u64,
}

#[derive(Debug, Clone, StructPacker)]
pub struct DelegateBandwidthAction {
    pub from: Name,
    pub receiver: Name,
    pub stake_net_quantity: u64,
    pub stake_cpu_quantity: u64,
    pub transfer: bool,
}

#[derive(Debug, Clone, StructPacker)]
pub struct SetCodeAction {
    pub account: Name,
    pub vmtype: u8,
    pub vmversion: u8,
    pub code: Vec<u8>,
    pub memo: BinaryExtension<String>
}

#[derive(Debug, Clone, StructPacker)]
pub struct SetAbiAction {
    pub account: Name,
    pub abi: Vec<u8>,
    pub memo: BinaryExtension<String>
}
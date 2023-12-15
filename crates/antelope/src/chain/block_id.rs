use crate::chain::{ Encoder, Decoder, Packer };
use antelope_macros::StructPacker;

#[derive(Clone, Eq, PartialEq, StructPacker)]
pub struct BlockId {
    pub bytes: Vec<u8>
}

impl BlockId {

    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err(String::from("BlockId.from_bytes expected bytes length of 32"))
        }
        Ok(Self {
            bytes: bytes.to_vec()
        })
    }
}
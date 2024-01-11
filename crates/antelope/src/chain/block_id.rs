use crate::chain::{Decoder, Encoder, Packer};
use antelope_client_macros::StructPacker;
use std::fmt::{Display, Formatter};

#[derive(Clone, Eq, PartialEq, StructPacker)]
pub struct BlockId {
    pub bytes: Vec<u8>,
}

impl BlockId {
    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err(String::from(
                "BlockId.from_bytes expected bytes length of 32",
            ));
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    pub fn block_num(&self) -> u32 {
        let num_bytes = &self.bytes[0..4];
        (u32::from(num_bytes[0]) << 24)
            | (u32::from(num_bytes[1]) << 16)
            | (u32::from(num_bytes[2]) << 8)
            | u32::from(num_bytes[3])
    }

    pub fn as_string(&self) -> String {
        self.block_num().to_string()
    }
}

impl Display for BlockId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

use crate::chain::{Decoder, Encoder, Packer};
use antelope_client_macros::StructPacker;
use serde::{
    de::{self, Visitor},
    Deserializer,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Default, Eq, PartialEq, StructPacker, Serialize, Deserialize, Debug)]
pub struct BlockId {
    pub bytes: Vec<u8>,
}

impl BlockId {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
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

pub(crate) fn deserialize_block_id<'de, D>(deserializer: D) -> Result<BlockId, D::Error>
where
    D: Deserializer<'de>,
{
    struct BlockIdVisitor;

    impl<'de> Visitor<'de> for BlockIdVisitor {
        type Value = BlockId;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a hex string for BlockId")
        }

        fn visit_str<E>(self, value: &str) -> Result<BlockId, E>
        where
            E: de::Error,
        {
            if value.len() != 64 {
                // 64 hex chars = 32 bytes
                return Err(E::custom(
                    "BlockId hex string must be exactly 64 characters long",
                ));
            }

            let mut bytes = Vec::new();
            for i in 0..32 {
                // Process 32 bytes
                let byte_slice = &value[i * 2..i * 2 + 2];
                match u8::from_str_radix(byte_slice, 16) {
                    Ok(byte) => bytes.push(byte),
                    Err(_) => return Err(E::custom("Invalid hex string for BlockId")),
                }
            }

            Ok(BlockId { bytes })
        }
    }

    deserializer.deserialize_str(BlockIdVisitor)
}

pub(crate) fn deserialize_optional_block_id<'de, D>(
    deserializer: D,
) -> Result<Option<BlockId>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the input as an Option<String>. If it's None, directly return Ok(None).
    let opt: Option<String> = Option::deserialize(deserializer)?;
    // Map the Option<String> to Option<BlockId> by converting the hex string to bytes and then using from_bytes.
    let result = match opt {
        Some(str_val) => {
            let mut bytes = Vec::new();
            for i in 0..(str_val.len() / 2) {
                let byte_slice = &str_val[i * 2..i * 2 + 2];
                match u8::from_str_radix(byte_slice, 16) {
                    Ok(byte) => bytes.push(byte),
                    Err(_) => {
                        return Err(serde::de::Error::custom("Invalid hex string for BlockId"))
                    }
                }
            }
            // Here you use from_bytes, which you should already have implemented in your BlockId struct.
            BlockId::from_bytes(&bytes)
                .map(Some)
                .map_err(serde::de::Error::custom)?
        }
        None => None,
    };
    Ok(result)
}

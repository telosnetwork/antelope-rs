use crate::base58::{decode_public_key, encode_ripemd160_check};
use crate::chain::{key_type::KeyType, Decoder, Encoder, Packer};
use crate::util::bytes_to_hex;
use antelope_client_macros::StructPacker;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Eq, PartialEq, Default, StructPacker, Serialize, Deserialize)]
pub struct PublicKey {
    pub key_type: KeyType,
    pub value: Vec<u8>,
}

impl PublicKey {
    pub fn as_string(&self) -> String {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(
            self.value.to_vec(),
            Option::from(self.key_type.to_string().as_str()),
        );
        format!("PUB_{type_str}_{encoded}")
    }

    pub fn to_hex_string(&self) -> String {
        bytes_to_hex(&self.value.to_vec())
    }

    pub fn to_legacy_string(&self, prefix: Option<&str>) -> Result<String, String> {
        let key_prefix = prefix.unwrap_or("EOS");
        if !matches!(self.key_type, KeyType::K1) {
            return Err(String::from("Unable to legacy key for non-k1 key"));
        }
        let encoded = encode_ripemd160_check(self.value.to_vec(), None);
        Ok(format!("{key_prefix}{encoded}"))
    }

    pub fn new_from_str(value: &str) -> Result<Self, String> {
        match decode_public_key(value) {
            Ok(decoded) => Ok(PublicKey {
                key_type: decoded.0,
                value: decoded.1,
            }),
            Err(err_string) => Err(err_string),
        }
    }

    pub fn from_bytes(value: Vec<u8>, key_type: KeyType) -> Self {
        PublicKey { key_type, value }
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

use crate::base58;
use crate::base58::encode_ripemd160_check;
use crate::chain::key_type::KeyTypeTrait;
use crate::chain::{ABISerializableObject, JSONValue};
use crate::chain::checksum::Checksum256;
use crate::chain::key_type::KeyType;
use crate::chain::public_key::PublicKey;
use crate::crypto::verify::verify;
use crate::serializer::encoder::ABIEncoder;

pub struct Signature {
    pub key_type: KeyType,
    value: Vec<u8>,
}

impl Signature {

    pub fn r(&self) -> Vec<u8> {
        return self.value[1..33].to_vec();
    }

    pub fn s(&self) -> Vec<u8> {
        return self.value[33..65].to_vec();
    }

    pub fn verify_digest(&self, digest: Checksum256, public_key: PublicKey) -> bool {
        return verify(self, digest.checksum.value, public_key.value, self.key_type);
    }

    /** Verify this signature with given message and public key. */
    pub fn verify_message(&self, message: Vec<u8>, public_key: PublicKey) -> bool {
        return self.verify_digest(Checksum256::hash(message), public_key);
    }



    pub fn to_string(&self) -> String {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(self.value.to_vec(), Option::from(self.key_type.to_string().as_str()));
        return format!("SIG_{type_str}_{encoded}");
    }

    pub fn from_string(s: &str) -> Result<Self, String> {
        if !s.starts_with("SIG_") {
            return Err(format!("String did not start with SIG_: {s}"));
        }
        let parts: Vec<&str> = s.split("_").collect();
        let key_type = KeyType::from_string(parts[1]).unwrap();
        let mut size: Option<usize> = None;
        match key_type {
            KeyType::K1 | KeyType::R1 => {
                size = Some(65);
            }
        }

        let value = base58::decode_ripemd160_check(parts[2], size, Option::from(key_type)).unwrap();
        return Ok(Signature {
            key_type,
            value
        })
    }

    pub fn from_bytes(bytes: Vec<u8>, key_type: KeyType) -> Self {
        return Signature {
            key_type,
            value: bytes
        }
    }

}

impl ABISerializableObject for Signature {
    fn get_abi_name(&self) -> String {
        return String::from("signature");
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_byte(self.key_type.to_index());
        encoder.write_array(&self.value.to_vec());
    }

    fn to_json(&self) -> JSONValue {
        return JSONValue::String(self.to_string());
    }
}
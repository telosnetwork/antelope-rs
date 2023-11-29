use crate::base58::{decode_public_key, encode_ripemd160_check};
use crate::chain::{ABISerializableObject, JSONValue};
use crate::chain::key_type::KeyType;
use crate::serializer::encoder::ABIEncoder;
use crate::util::bytes_to_hex;

pub struct PublicKey {
    pub key_type: KeyType,
    pub value: Vec<u8>,
}

impl PublicKey {

    pub fn to_string(&self) -> String {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(self.value.to_vec(), Option::from(self.key_type.to_string().as_str()));
        return format!("PUB_{type_str}_{encoded}");
    }

    pub fn to_hex_string(&self) -> String {
        return bytes_to_hex(&self.value.to_vec());
    }

    pub fn to_legacy_string(&self, prefix: Option<&str>) -> Result<String, String> {
        let key_prefix = prefix.unwrap_or("EOS");
        if !matches!(self.key_type, KeyType::K1) {
            return Err(String::from("Unable to legacy key for non-k1 key"));
        }
        let encoded = encode_ripemd160_check(self.value.to_vec(), None);
        return Ok(format!("{key_prefix}{encoded}"));
    }

    pub fn from_str(value: &str) -> Self {
        let decoded = decode_public_key(value).unwrap();
        return PublicKey {
            key_type: decoded.0,
            value: decoded.1
        }
    }

    pub fn from_bytes(value: Vec<u8>, key_type: KeyType) -> Self {
        return PublicKey {
            key_type,
            value
        }
    }

}

impl ABISerializableObject for PublicKey {
    fn get_abi_name(&self) -> String {
        return "public_key".to_string();
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_array(&self.value.to_vec());
    }

    fn to_json(&self) -> JSONValue {
        return JSONValue::String(self.to_string());
    }

    fn equals(&self, obj: Box<dyn ABISerializableObject>) -> bool {
        todo!()
    }
}
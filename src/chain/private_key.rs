use crate::base58::{decode_key, encode_check};
use crate::chain::{ABISerializableObject, JSONValue};
use crate::serializer::encoder::ABIEncoder;

#[derive(Clone, Copy)]
pub enum KeyType {
    K1,
    R1,
    // ... other variants ...
}

pub struct PrivateKey {
    pub key_type: KeyType,
    value: Vec<u8>,
}

impl PrivateKey {

    pub fn to_wif(&self) -> Result<String, String> {
        if !matches!(self.key_type, KeyType::K1) {
            return Err(String::from("Unable to generate WIF for non-k1 key"));
        }
        let mut to_encode = Vec::new();
        to_encode.push(0x80);
        to_encode.append(&mut self.value.to_vec());

        return Ok(encode_check(to_encode));
    }

    pub fn from(key: &str) -> Self {
        let decoded = decode_key(key).unwrap();
        return PrivateKey {
            key_type: decoded.0,
            value: decoded.1,
        }
    }

}

impl ABISerializableObject for PrivateKey {
    fn get_abi_name(&self) -> String {
        return "private_key".to_string();
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_array(self.value.to_vec());
    }

    fn to_json(&self) -> JSONValue {
        todo!()
    }

    fn equals(&self, obj: Box<dyn ABISerializableObject>) -> bool {
        todo!()
    }
}
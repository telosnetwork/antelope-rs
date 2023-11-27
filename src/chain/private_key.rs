use std::fmt::{Display, Formatter};
use crate::base58::{decode_key, encode_check, encode_ripemd160_check};
use crate::chain::{ABISerializableObject, JSONValue};
use crate::chain::key_type::KeyType;
use crate::serializer::encoder::ABIEncoder;


pub struct PrivateKey {
    pub key_type: KeyType,
    value: Vec<u8>,
}

impl PrivateKey {

    // TODO: should this be done via the ToString trait?
    //   If so, should other structs also do that?
    //   Also if so, should from on this and other structs use the From trait?
    pub fn to_string(&self) -> String {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(self.value.to_vec(), Option::from(self.key_type.to_string().as_str()));
        return format!("PVT_{type_str}_{encoded}");
    }

    pub fn to_hex(&self) -> String {
        return hex::encode(&self.value);
    }

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
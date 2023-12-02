use crate::base58::{decode_key, encode_check, encode_ripemd160_check};
use crate::chain::{ABISerializableObject, JSONValue};
use crate::chain::key_type::KeyType;
use crate::chain::public_key::PublicKey;
use crate::chain::signature::Signature;
use crate::crypto::get_public::get_public;
use crate::crypto::sign::sign;
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

    pub fn to_bytes(&self) -> Vec<u8> {
        return self.value.to_vec();
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

    pub fn to_public(&self) -> PublicKey {
        let compressed = get_public(self.value.to_vec(), self.key_type).unwrap();
        return PublicKey::from_bytes(compressed, self.key_type);
    }

    pub fn from_bytes(bytes: Vec<u8>, key_type: KeyType) -> Self {
        return PrivateKey {
            key_type,
            value: bytes
        }
    }

    pub fn from_str(key: &str) -> Self {
        let decoded = decode_key(key).unwrap();
        return PrivateKey {
            key_type: decoded.0,
            value: decoded.1,
        }
    }

    pub fn sign_message(&self, message: &Vec<u8>) -> Signature {
        return sign(self.value.to_vec(), message, self.key_type).unwrap();
    }

}

impl ABISerializableObject for PrivateKey {
    fn get_abi_name(&self) -> String {
        return "private_key".to_string();
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_array(&self.value.to_vec());
    }

    fn to_json(&self) -> JSONValue {
        todo!()
    }
}
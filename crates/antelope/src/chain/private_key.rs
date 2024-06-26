use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    base58::{decode_key, encode_check, encode_ripemd160_check},
    chain::{
        checksum::Checksum512, key_type::KeyType, public_key::PublicKey, signature::Signature,
    },
    crypto::{
        generate::generate, get_public::get_public, shared_secrets::shared_secret, sign::sign,
    },
};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PrivateKey {
    pub key_type: KeyType,
    value: Vec<u8>,
}

impl PrivateKey {
    // TODO: should this be done via the ToString trait?
    //   If so, should other structs also do that?
    //   Also if so, should from on this and other structs use the From trait?
    pub fn as_string(&self) -> String {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(
            self.value.to_vec(),
            Option::from(self.key_type.to_string().as_str()),
        );
        format!("PVT_{type_str}_{encoded}")
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.value.to_vec()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.value)
    }

    pub fn to_wif(&self) -> Result<String, String> {
        if !matches!(self.key_type, KeyType::K1) {
            return Err(String::from("Unable to generate WIF for non-k1 key"));
        }
        let mut to_encode = Vec::new();
        to_encode.push(0x80);
        to_encode.append(&mut self.value.to_vec());

        Ok(encode_check(to_encode))
    }

    pub fn to_public(&self) -> PublicKey {
        let compressed = get_public(self.value.to_vec(), self.key_type).unwrap();
        PublicKey::from_bytes(compressed, self.key_type)
    }

    pub fn from_bytes(bytes: Vec<u8>, key_type: KeyType) -> Self {
        PrivateKey {
            key_type,
            value: bytes,
        }
    }

    pub fn from_str(key: &str, ignore_checksum: bool) -> Result<Self, String> {
        let decode_result = decode_key(key, ignore_checksum);
        if decode_result.is_err() {
            let err_message = decode_result.err().unwrap_or(String::from("Unknown error"));
            return Err(format!("Failed to decode private key: {err_message}"));
        }

        let decoded = decode_result.unwrap();
        Ok(PrivateKey {
            key_type: decoded.0,
            value: decoded.1,
        })
    }

    pub fn sign_message(&self, message: &Vec<u8>) -> Signature {
        sign(self.value.to_vec(), message, self.key_type).unwrap()
    }

    pub fn shared_secret(&self, their_pub: &PublicKey) -> Checksum512 {
        Checksum512::hash(shared_secret(&self.to_bytes(), &their_pub.value, self.key_type).unwrap())
    }

    pub fn random(key_type: KeyType) -> Result<Self, String> {
        let secret_bytes = generate(key_type);
        Ok(Self::from_bytes(secret_bytes.unwrap(), key_type))
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

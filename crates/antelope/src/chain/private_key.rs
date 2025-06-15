use std::fmt::{Debug, Display, Formatter, LowerHex};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::crypto::shared_secrets::SharedSecretError;
use crate::crypto::sign::SignError;
use crate::{base58::{decode_key, encode_check, encode_ripemd160_check}, chain::{
    checksum::Checksum512, key_type::KeyType, public_key::PublicKey, signature::Signature,
}, crypto::{
    generate::generate, shared_secrets::shared_secret, sign::sign,
}, define_error};

use super::public_key::PublicKeyParsingError;

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrivateKey {
    pub key_type: KeyType,
    pub value: Vec<u8>,
}

impl PrivateKey {
    pub fn to_wif(&self) -> Result<String, String> {
        if !matches!(self.key_type, KeyType::K1) {
            return Err(String::from("Unable to generate WIF for non-k1 key"));
        }
        let mut to_encode = Vec::new();
        to_encode.push(0x80);
        to_encode.append(&mut self.value.to_vec());

        Ok(encode_check(to_encode))
    }

    pub fn to_public(&self) -> Result<PublicKey, PublicKeyParsingError> {
        PublicKey::try_from(self)
    }

    /// # Safety
    /// Only call if you know key is a valid PrivateKey if not an invalid key will be
    /// instantiated with no errors
    pub unsafe fn from_str_unchecked(key: &str) -> PrivateKey {
        let decoded = decode_key(key, true)
            .unwrap_unchecked();
        PrivateKey {
            key_type: decoded.0,
            value: decoded.1,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Result<Signature, SignError> {
        sign(&self.value, message, self.key_type)
    }

    pub fn shared_secret(&self, their_pub: &PublicKey) -> Result<Checksum512, SharedSecretError> {
        Ok(Checksum512::hash(
            shared_secret(&self.value, &their_pub.value, self.key_type)?
        ))
    }

    pub fn random(key_type: KeyType) -> Result<Self, String> {
        let secret_bytes = generate(key_type)?;
        Ok(Self::from((secret_bytes[1..].to_vec(), key_type)))
    }
}

impl From<(Vec<u8>, KeyType)> for PrivateKey {
    fn from(value: (Vec<u8>, KeyType)) -> Self {
        let (value, key_type) = value;
        PrivateKey { key_type, value }
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let key_type = KeyType::try_from(data[0])?;
        Ok(PrivateKey::from((data[1..].to_vec(), key_type)))
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(
            self.value.to_vec(),
            Option::from(self.key_type.to_string().as_str()),
        );
        write!(f, "PVT_{type_str}_{encoded}")
    }
}

impl LowerHex for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.value))
    }
}

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

define_error!(PrivateKeyParsingError);

impl FromStr for PrivateKey {
    type Err = PrivateKeyParsingError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (key_type, value) = decode_key(value, false)
            .map_err(|e| PrivateKeyParsingError::new(e.to_string()))?;

        Ok(PrivateKey {
            key_type, value
        })
    }
}

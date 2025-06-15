use crate::{base58::{decode_public_key, encode_ripemd160_check}, chain::key_type::KeyType, crypto::get_public::get_public, define_error};
use hex::encode;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{self, Debug, LowerHex};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::serializer::{Decoder, Encoder, Packer, PackerError};

use super::private_key::PrivateKey;

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct PublicKey {
    pub key_type: KeyType,
    pub value: Vec<u8>,
}

impl Packer for PublicKey {
    fn size(&self) -> usize {
        34usize
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        let pos = enc.get_size();
        self.key_type.pack(enc);
        for v in self.value.iter() {
            v.pack(enc);
        }
        enc.get_size() - pos
    }

    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        let mut dec = Decoder::new(data);
        let mut key_type = KeyType::default();
        dec.unpack(&mut key_type)?;
        self.value.reserve(32usize);
        for _ in 0..33 {
            let mut v: u8 = Default::default();
            dec.unpack(&mut v)?;
            self.value.push(v);
        }
        Ok(dec.get_pos())
    }
}

impl PublicKey {
    pub fn to_legacy_string(&self, prefix: Option<&str>) -> Result<String, String> {
        let key_prefix = prefix.unwrap_or("EOS");
        if !matches!(self.key_type, KeyType::K1) {
            return Err(String::from("Unable to legacy key for non-k1 key"));
        }
        let encoded = encode_ripemd160_check(self.value.to_vec(), None);
        Ok(format!("{key_prefix}{encoded}"))
    }
}

impl From<(Vec<u8>, KeyType)> for PublicKey {
    fn from(value: (Vec<u8>, KeyType)) -> Self {
        let (value, key_type) = value;
        PublicKey { key_type, value }
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = PackerError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut dec = Decoder::new(data);
        let mut key = PublicKey::default();
        dec.unpack(&mut key)?;
        Ok(key)
    }
}

define_error!(PublicKeyParsingError);

impl FromStr for PublicKey {
    type Err = PublicKeyParsingError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match decode_public_key(value) {
            Ok(decoded) => Ok(PublicKey {
                key_type: decoded.0,
                value: decoded.1,
            }),
            Err(err_string) => Err(PublicKeyParsingError::new(err_string)),
        }
    }
}

impl TryFrom<&PrivateKey> for PublicKey {
    type Error = PublicKeyParsingError;

    fn try_from(k: &PrivateKey) -> Result<Self, Self::Error> {
        let compressed = get_public(k.value.clone(), k.key_type)
            .map_err(PublicKeyParsingError::new)?;

        Ok(PublicKey::from((compressed, k.key_type)))
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(
            self.value.to_vec(),
            Option::from(type_str.as_str()),
        );
        write!(f, "PUB_{type_str}_{encoded}")
    }
}

impl LowerHex for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", encode(&self.value))
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

pub fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    struct PublicKeyVisitor;

    impl serde::de::Visitor<'_> for PublicKeyVisitor {
        type Value = PublicKey;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string representing a PublicKey")
        }

        fn visit_str<E>(self, value: &str) -> Result<PublicKey, E>
        where
            E: serde::de::Error,
        {
            PublicKey::from_str(value)
                .map_err(E::custom)
        }
    }

    deserializer.deserialize_str(PublicKeyVisitor)
}

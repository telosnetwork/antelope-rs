use crate::{
    base58::{decode_public_key, encode_ripemd160_check},
    chain::{key_type::KeyType, Decoder, Encoder, Packer},
    util::bytes_to_hex,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
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

    fn unpack(&mut self, data: &[u8]) -> usize {
        let mut dec = Decoder::new(data);
        let mut key_type = KeyType::default();
        dec.unpack(&mut key_type);
        self.value.reserve(32usize);
        for _ in 0..33 {
            let mut v: u8 = Default::default();
            dec.unpack(&mut v);
            self.value.push(v);
        }
        dec.get_pos()
    }
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

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

pub fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    struct PublicKeyVisitor;

    impl<'de> serde::de::Visitor<'de> for PublicKeyVisitor {
        type Value = PublicKey;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string representing a PublicKey")
        }

        fn visit_str<E>(self, value: &str) -> Result<PublicKey, E>
        where
            E: serde::de::Error,
        {
            match PublicKey::new_from_str(value) {
                Ok(pub_key) => Ok(pub_key),
                Err(err) => Err(E::custom(err)),
            }
        }
    }

    deserializer.deserialize_str(PublicKeyVisitor)
}

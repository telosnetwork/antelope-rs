use core::fmt;
use std::fmt::{Display, Formatter};

use ecdsa::RecoveryId;
use k256::Secp256k1;
use p256::NistP256;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{
    base58,
    base58::encode_ripemd160_check,
    chain::{
        key_type::{KeyType, KeyTypeTrait},
        public_key::PublicKey,
        Encoder, Packer,
    },
    crypto::{recover::recover_message, verify::verify_message},
    util::slice_copy,
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub key_type: KeyType,
    value: Vec<u8>,
}

impl Signature {
    pub const RECOVERY_ID_ADDITION: u8 = 27;

    pub fn recovery_id(&self) -> u8 {
        self.value[0]
    }

    pub fn r(&self) -> Vec<u8> {
        self.value[1..33].to_vec()
    }

    pub fn s(&self) -> Vec<u8> {
        self.value[33..65].to_vec()
    }

    /*
    // TODO: Figure out how to reconstruct a Digest from a byte array
    //   currently there is no simple/clear way to do this and in verify.rs
    //   the VerifyingKey has either verify(message_bytes) or verify_digest(Digest)
    pub fn verify_digest(&self, digest: Checksum256, public_key: PublicKey) -> bool {
        return verify(self, digest.checksum.value, public_key.value, self.key_type);
    }
     */

    pub fn verify_message(&self, message: &Vec<u8>, public_key: &PublicKey) -> bool {
        verify_message(self, message, &public_key.value)
    }

    pub fn recover_message(&self, message: &Vec<u8>) -> PublicKey {
        recover_message(self, message)
    }

    pub fn as_string(&self) -> String {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(
            self.value.to_vec(),
            Option::from(self.key_type.to_string().as_str()),
        );
        format!("SIG_{type_str}_{encoded}")
    }

    pub fn from_string(s: &str) -> Result<Self, String> {
        if !s.starts_with("SIG_") {
            return Err(format!("String did not start with SIG_: {s}"));
        }
        let parts: Vec<&str> = s.split('_').collect();
        let key_type = KeyType::from_string(parts[1]).unwrap();
        let size: Option<usize> = Some(65);
        // TODO: add back this logic when other key types are supported and have a
        // different length match key_type {
        //     KeyType::K1 | KeyType::R1 => {
        //         size = Some(65);
        //     }
        // }

        let value =
            base58::decode_ripemd160_check(parts[2], size, Option::from(key_type), false).unwrap();
        Ok(Signature { key_type, value })
    }

    pub fn from_k1_signature(
        signature: ecdsa::Signature<Secp256k1>,
        recovery: RecoveryId,
    ) -> Result<Self, String> {
        let r = signature.r().to_bytes().to_vec();
        let s = signature.s().to_bytes().to_vec();
        let mut data: Vec<u8> = Vec::new();
        let recid = recovery.to_byte() + Signature::RECOVERY_ID_ADDITION;

        if r.len() != 32 || s.len() != 32 {
            return Err(String::from("r and s values should both have a size of 32"));
        }

        if !Signature::is_canonical(&r, &s) {
            return Err(String::from("Signature values are not canonical"));
        }

        data.push(recid);
        data.extend(r.to_vec());
        data.extend(s.to_vec());

        Ok(Signature {
            key_type: KeyType::K1,
            value: data,
        })
    }

    pub fn from_r1_signature(
        signature: ecdsa::Signature<NistP256>,
        recovery: RecoveryId,
    ) -> Result<Self, String> {
        let r = signature.r().to_bytes().to_vec();
        let s = signature.s().to_bytes().to_vec();
        let mut data: Vec<u8> = Vec::new();
        let recid = recovery.to_byte();

        if r.len() != 32 || s.len() != 32 {
            return Err(String::from("r and s values should both have a size of 32"));
        }

        data.push(recid);
        data.extend(r.to_vec());
        data.extend(s.to_vec());

        Ok(Signature {
            key_type: KeyType::R1,
            value: data,
        })
    }

    pub fn from_bytes(bytes: Vec<u8>, key_type: KeyType) -> Self {
        Signature {
            key_type,
            value: bytes,
        }
    }

    pub fn is_canonical(r: &[u8], s: &[u8]) -> bool {
        !((r[0] & 0x80 != 0)
            || (s[0] & 0x80 != 0)
            || r[0] == 0 && r[1] & 0x80 == 0
            || s[0] == 0 && s[1] & 0x80 == 0)
    }
}

pub(crate) fn deserialize_signature<'de, D>(deserializer: D) -> Result<Signature, D::Error>
where
    D: Deserializer<'de>,
{
    struct SignatureVisitor;

    impl<'de> Visitor<'de> for SignatureVisitor {
        type Value = Signature;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a hex string of length 64 (for 32 bytes)")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Signature::from_string(value).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(SignatureVisitor)
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self {
            key_type: KeyType::K1,
            value: vec![0; 65],
        }
    }
}

impl Packer for Signature {
    fn size(&self) -> usize {
        66
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        self.key_type.pack(enc);
        let data = enc.alloc(self.value.len());
        slice_copy(data, &self.value);
        self.size()
    }

    fn unpack(&mut self, data: &[u8]) -> usize {
        let size = self.size();
        assert!(data.len() >= size, "Signature::unpack: buffer overflow");
        self.key_type = KeyType::from_index(data[0]).unwrap();
        self.value = data[1..size].to_vec();
        self.size()
    }
}

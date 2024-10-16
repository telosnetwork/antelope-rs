use core::fmt;
use std::fmt::{Display, Formatter};

use ecdsa::RecoveryId;
use k256::Secp256k1;
use p256::NistP256;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::chain::varint::VarUint32;
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
        let size = match key_type {
            KeyType::K1 | KeyType::R1 => Some(65),
            KeyType::WA => None,
        };

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
        1 + self.value.len()
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        self.key_type.pack(enc);
        let data = enc.alloc(self.value.len());
        slice_copy(data, &self.value);
        self.size()
    }

    fn unpack(&mut self, data: &[u8]) -> usize {
        self.key_type = KeyType::from_index(data[0]).unwrap();
        match self.key_type {
            KeyType::K1 | KeyType::R1 => {
                self.value = data[1..66].to_vec();
            }
            KeyType::WA => {
                let mut size = 66; // size to start = 1 byte for key type, 65 bytes for compact signature
                let mut auth_data = VarUint32::default();
                // unpack() returns how many bytes were read to unpack the value
                size += auth_data.unpack(&data[size..]); // after the compact sig comes a varuint32 to tell us the size of the auth data
                size += auth_data.value() as usize; // add the auth data size
                let mut client_json = VarUint32::default();
                size += client_json.unpack(&data[size..]); // read the varuint32 size of the client_json
                size += client_json.value() as usize; // add the client_json size
                                                      // set value to be the whole payload (after the key type byte):
                                                      //      compact sig,
                                                      //      varuint32 auth data size,
                                                      //      auth data,
                                                      //      varuint32 client_json size,
                                                      //      client_json
                self.value = data[1..size].to_vec();
            }
        }
        let size = self.size();
        assert!(data.len() >= size, "Signature::unpack: buffer overflow");
        self.size()
    }
}

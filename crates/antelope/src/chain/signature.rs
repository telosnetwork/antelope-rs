use core::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use ecdsa::RecoveryId;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::chain::varint::VarUint32;
use crate::crypto::recover::RecoverMessageError;
use crate::crypto::verify::VerifyMessageError;
use crate::packer_error;
use crate::{
    base58,
    base58::encode_ripemd160_check,
    chain::{
        key_type::KeyType,
        public_key::PublicKey,
    },
    check_unpack_len,
    crypto::{recover::recover_message, verify::verify_message},
    util::slice_copy
};
use crate::serializer::{Decoder, Encoder, Packer, PackerError};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub key_type: KeyType,
    pub value: Vec<u8>,
}

impl Signature {
    pub const RECOVERY_ID_ADDITION: u8 = 31;

    pub fn recovery_id(&self) -> u8 {
        self.value[0]
    }

    pub fn r(&self) -> Vec<u8> {
        self.value[1..33].to_vec()
    }

    pub fn s(&self) -> Vec<u8> {
        self.value[33..65].to_vec()
    }

    pub fn verify_message(&self, message: &[u8], public_key: &PublicKey) -> Result<(), VerifyMessageError> {
        verify_message(self, message, &public_key.value)
    }

    pub fn recover_message(&self, message: &[u8]) -> Result<PublicKey, RecoverMessageError> {
        recover_message(self, message)
    }

    pub fn is_canonical(r: &[u8], s: &[u8]) -> bool {
        !((r[0] & 0x80 != 0)
            || (s[0] & 0x80 != 0)
            || (r[0] == 0 && (r[1] & 0x80) == 0)
            || (s[0] == 0 && (s[1] & 0x80) == 0))
    }
}

impl From<(Vec<u8>, KeyType)> for Signature {
    fn from(value: (Vec<u8>, KeyType)) -> Self {
        let (value, key_type) = value;
        Signature { key_type, value }
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = PackerError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let mut dec = Decoder::new(data);
        let mut sig = Signature::default();
        dec.unpack(&mut sig)?;
        Ok(sig)
    }
}

impl FromStr for Signature {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(3, '_');
        if parts.next() != Some("SIG") {
            return Err(format!("invalid signature prefix: {s}"));
        }
        let ty = parts.next().ok_or_else(|| "missing key type".to_string())?;
        let payload = parts.next().ok_or_else(|| "missing payload".to_string())?;
        let key_type = KeyType::from_str(ty)?;
        let size = match key_type {
            KeyType::K1 | KeyType::R1 => Some(65),
            KeyType::WA => None,
        };
        let value = base58::decode_ripemd160_check(payload, size, Some(key_type), false)
            .map_err(|e| e.to_string())?;
        Ok(Signature { key_type, value })
    }
}

impl TryFrom<(ecdsa::Signature<k256::Secp256k1>, RecoveryId)> for Signature {
    type Error = String;
    fn try_from(value: (ecdsa::Signature<k256::Secp256k1>, RecoveryId)) -> Result<Self, Self::Error> {
        let (signature, recovery) = value;
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
}

impl TryFrom<(ecdsa::Signature<p256::NistP256>, RecoveryId)> for Signature {
    type Error = String;
    fn try_from(value: (ecdsa::Signature<p256::NistP256>, RecoveryId)) -> Result<Self, Self::Error> {
        let (signature, recovery) = value;
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
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(
            self.value.to_vec(),
            Some(type_str.as_str()),
        );
        write!(f, "SIG_{type_str}_{encoded}")
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Default for Signature {
    fn default() -> Self {
        Signature {
            key_type: KeyType::K1,
            value: vec![0; 65],
        }
    }
}

pub(crate) fn deserialize_signature<'de, D>(deserializer: D) -> Result<Signature, D::Error>
where
    D: Deserializer<'de>,
{
    struct SignatureVisitor;

    impl Visitor<'_> for SignatureVisitor {
        type Value = Signature;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a SIG_<type>_<base58> formatted string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.parse().map_err(E::custom)
        }
    }

    deserializer.deserialize_str(SignatureVisitor)
}

impl Packer for Signature {
    fn size(&self) -> usize {
        1 + self.value.len()
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        self.key_type.pack(enc);
        let buf = enc.alloc(self.value.len());
        slice_copy(buf, &self.value);
        self.size()
    }

    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        self.key_type = KeyType::try_from(data[0])
            .map_err(|e| packer_error!("KeyType::try_from: {e}"))?;
        match self.key_type {
            KeyType::K1 | KeyType::R1 => {
                self.value = data[1..66].to_vec();
            }
            KeyType::WA => {
                // skip key_type(1) + compact sig(65)
                let mut offset = 1 + 65;
                let mut auth_sz = VarUint32::default();
                let n = auth_sz.unpack(&data[offset..])?;
                offset += n + usize::from(auth_sz);

                let mut client_sz = VarUint32::default();
                let m = client_sz.unpack(&data[offset..])?;
                offset += m + usize::from(client_sz);

                // capture everything after the key_type byte
                self.value = data[1..offset].to_vec();
            }
        }
        let total = 1 + self.value.len();
        check_unpack_len!(self, data, total);
        Ok(total)
    }
}

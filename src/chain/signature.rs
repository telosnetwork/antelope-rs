use ecdsa::RecoveryId;
use k256::Secp256k1;
use p256::NistP256;
use crate::base58;
use crate::base58::encode_ripemd160_check;
use crate::chain::key_type::KeyTypeTrait;
use crate::chain::{ABISerializableObject, JSONValue};
use crate::chain::key_type::KeyType;
use crate::chain::public_key::PublicKey;
use crate::crypto::verify::{verify_message};
use crate::serializer::encoder::ABIEncoder;

pub struct Signature {
    pub key_type: KeyType,
    value: Vec<u8>,
}

impl Signature {

    pub fn r(&self) -> Vec<u8> {
        return self.value[1..33].to_vec();
    }

    pub fn s(&self) -> Vec<u8> {
        return self.value[33..65].to_vec();
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
        return verify_message(self, message, &public_key.value);
    }

    pub fn to_string(&self) -> String {
        let type_str = self.key_type.to_string();
        let encoded = encode_ripemd160_check(self.value.to_vec(), Option::from(self.key_type.to_string().as_str()));
        return format!("SIG_{type_str}_{encoded}");
    }

    pub fn from_string(s: &str) -> Result<Self, String> {
        if !s.starts_with("SIG_") {
            return Err(format!("String did not start with SIG_: {s}"));
        }
        let parts: Vec<&str> = s.split("_").collect();
        let key_type = KeyType::from_string(parts[1]).unwrap();
        let mut size: Option<usize> = None;
        match key_type {
            KeyType::K1 | KeyType::R1 => {
                size = Some(65);
            }
        }

        let value = base58::decode_ripemd160_check(parts[2], size, Option::from(key_type)).unwrap();
        return Ok(Signature {
            key_type,
            value
        })
    }

    pub fn from_k1_signature(signature: ecdsa::Signature<Secp256k1>, recovery: RecoveryId) -> Result<Self, String> {
        let r = signature.r().to_bytes().to_vec();
        let s = signature.s().to_bytes().to_vec();
        let mut data: Vec<u8> = Vec::new();
        let recid = recovery.to_byte();

        if r.len() != 32 || s.len() != 32 {
            return Err(String::from("r and s values should both have a size of 32"));
        }

        if !Signature::is_canonical(&r, &s) {
            return Err(String::from("Signature values are not canonical"));
        }

        data.push(recid);
        data.extend(r.to_vec());
        data.extend(s.to_vec());

        return Ok(Signature {
            key_type: KeyType::K1,
            value: data
        })
    }

    pub fn from_r1_signature(signature: ecdsa::Signature<NistP256>, recovery: RecoveryId) -> Result<Self, String> {
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

        return Ok(Signature {
            key_type: KeyType::R1,
            value: data
        })
    }

    pub fn from_bytes(bytes: Vec<u8>, key_type: KeyType) -> Self {
        return Signature {
            key_type,
            value: bytes
        }
    }

    pub fn is_canonical(r: &Vec<u8>, s: &Vec<u8>) -> bool {
        return !(r[0] & 0x80 != 0)
            && !(r[0] == 0 && r[1] & 0x80 == 0)
            && !(s[0] & 0x80 != 0)
            && !(s[0] == 0 && s[1] & 0x80 == 0);
    }

}

impl ABISerializableObject for Signature {
    fn get_abi_name(&self) -> String {
        return String::from("signature");
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_byte(self.key_type.to_index());
        encoder.write_array(&self.value.to_vec());
    }

    fn to_json(&self) -> JSONValue {
        return JSONValue::String(self.to_string());
    }
}

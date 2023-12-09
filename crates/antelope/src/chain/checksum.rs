use ripemd::{Digest as Ripemd160Digest, Ripemd160};
use sha2::{Sha256, Sha512};
use crate::chain::{ABISerializableObject, JSONValue};
use crate::serializer::encoder::ABIEncoder;
use crate::util::bytes_to_hex;

pub struct Checksum {
    pub value: Vec<u8>,
    pub byte_size: usize,
}

impl Checksum {
    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_array(&self.value.to_vec());
    }

    fn to_string(&self) -> String {
        return bytes_to_hex(&self.value.to_vec());
    }

    fn to_json(&self) -> JSONValue {
        return JSONValue::String(self.to_string());
    }
}

// TODO: Make all this less duplicated, somehow use generic implemetation and only specify size and hash function

pub struct Checksum160 {
    pub checksum: Checksum
}

impl Checksum160 {
    pub const BYTE_SIZE: usize = 20;

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() != Checksum160::BYTE_SIZE {
            let len = bytes.len();
            let expected_len = Checksum160::BYTE_SIZE;
            return Err(format!("Bytes len should be {expected_len} for Checksum160 but was {len}"));
        }

        return Ok(Checksum160 {
            checksum: Checksum {
                value: bytes,
                byte_size: Checksum160::BYTE_SIZE
            }
        });
    }

    pub fn hash(bytes: Vec<u8>) -> Self {
        let mut hasher = Ripemd160::new();
        hasher.update(bytes);
        let ripe_hash = hasher.finalize();
        return Checksum160::from_bytes(ripe_hash.to_vec()).unwrap();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        return self.checksum.value.to_vec();
    }

    pub fn to_string(&self) -> String {
        return self.checksum.to_string();
    }
}

impl ABISerializableObject for Checksum160 {
    fn get_abi_name(&self) -> String {
        return String::from("checksum160");
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        self.checksum.to_abi(encoder);
    }

    fn to_json(&self) -> JSONValue {
        return self.checksum.to_json();
    }
}


pub struct Checksum256 {
    pub checksum: Checksum
}

impl Checksum256 {
    pub const BYTE_SIZE: usize = 32;

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() != Checksum256::BYTE_SIZE {
            let len = bytes.len();
            let expected_len = Checksum256::BYTE_SIZE;
            return Err(format!("Bytes len should be {expected_len} for Checksum256 but was {len}"));
        }

        return Ok(Checksum256 {
            checksum: Checksum {
                value: bytes,
                byte_size: Checksum256::BYTE_SIZE
            }
        });
    }

    pub fn hash(bytes: Vec<u8>) -> Self {
        return Checksum256::from_bytes(Sha256::digest(bytes).to_vec()).unwrap();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        return self.checksum.value.to_vec();
    }

    pub fn to_string(&self) -> String {
        return self.checksum.to_string();
    }
}

impl ABISerializableObject for Checksum256 {
    fn get_abi_name(&self) -> String {
        return String::from("checksum256");
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        self.checksum.to_abi(encoder);
    }

    fn to_json(&self) -> JSONValue {
        return self.checksum.to_json();
    }
}


pub struct Checksum512 {
    pub checksum: Checksum
}

impl Checksum512 {
    pub const BYTE_SIZE: usize = 64;

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() != Checksum512::BYTE_SIZE {
            let len = bytes.len();
            let expected_len = Checksum512::BYTE_SIZE;
            return Err(format!("Bytes len should be {expected_len} for Checksum512 but was {len}"));
        }

        return Ok(Checksum512 {
            checksum: Checksum {
                value: bytes,
                byte_size: Checksum512::BYTE_SIZE
            }
        });
    }

    pub fn hash(bytes: Vec<u8>) -> Self {
        return Checksum512::from_bytes(Sha512::digest(bytes).to_vec()).unwrap();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        return self.checksum.value.to_vec();
    }

    pub fn to_string(&self) -> String {
        return self.checksum.to_string();
    }
}

impl ABISerializableObject for Checksum512 {
    fn get_abi_name(&self) -> String {
        return String::from("checksum512");
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        self.checksum.to_abi(encoder);
    }

    fn to_json(&self) -> JSONValue {
        return self.checksum.to_json();
    }
}


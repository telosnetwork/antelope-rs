use crate::util::array_equals;
use base64::engine::general_purpose::PAD;
use base64::engine::GeneralPurpose;
use base64::{alphabet, Engine as _};

#[derive(Debug, PartialEq, Eq)]
pub enum BlobType {
    Bytes(Vec<u8>),
    String(String),
}

pub struct Blob {
    pub array: Vec<u8>,
}

pub const STANDARD: GeneralPurpose = GeneralPurpose::new(&alphabet::STANDARD, PAD);

impl Blob {
    pub fn from(value: BlobType) -> Result<Blob, &'static str> {
        match value {
            BlobType::Bytes(bytes) => Ok(Blob { array: bytes }),
            BlobType::String(string) => Self::from_string(&string),
        }
    }

    pub fn from_string(value: &str) -> Result<Blob, &'static str> {
        // Remove padding characters '=' from the end of the string
        let value_without_padding: String = value.trim_end_matches('=').to_string();

        // Convert base64 string to bytes
        match STANDARD.decode(value_without_padding) {
            Ok(bytes) => Ok(Blob { array: bytes }),
            Err(_) => Err("Invalid base64 string"),
        }
    }

    pub fn equals(&self, other: &BlobType) -> bool {
        if let BlobType::Bytes(bytes) = other {
            array_equals(&self.array, bytes)
        } else {
            false
        }
    }

    pub fn base64_string(&self) -> String {
        STANDARD.encode(&self.array)
    }

    pub fn utf8_string(&self) -> Result<String, &'static str> {
        match String::from_utf8(self.array.clone()) {
            Ok(utf8_string) => Ok(utf8_string),
            Err(_) => Err("Invalid UTF-8 string"),
        }
    }
}

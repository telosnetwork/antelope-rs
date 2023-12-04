use crate::chain::{ABISerializableObject, JSONValue};
use crate::serializer::encoder::ABIEncoder;
use crate::util::array_equals;
use crate::util::is_instance_of;

#[derive(Debug, PartialEq, Eq)]
pub enum BlobType {
    Bytes(Vec<u8>),
    String(String),
}

pub struct Blob {
    pub array: Vec<u8>,
}

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
        match base64::decode(&value_without_padding) {
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
        base64::encode(&self.array)
    }

    pub fn utf8_string(&self) -> Result<String, &'static str> {
        match String::from_utf8(self.array.clone()) {
            Ok(utf8_string) => Ok(utf8_string),
            Err(_) => Err("Invalid UTF-8 string"),
        }
    }
}

impl ABISerializableObject for Blob {
    fn get_abi_name(&self) -> String {
        "blob".to_string()
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_array(&self.array);
    }

    fn to_json(&self) -> JSONValue {
        JSONValue::String(self.base64_string())
    }

    // fn equals(&self, other: Box<dyn ABISerializableObject>) -> bool {
    //     if let Some(blob) = other.downcast_ref::<Blob>() {
    //         self.equals(&BlobType::Bytes(blob.array.clone()))
    //     } else {
    //         false
    //     }
    // }
}

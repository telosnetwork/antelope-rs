use crate::chain::{ABISerializableObject, JSONValue};
use crate::serializer::encoder::ABIEncoder;
use crate::util::{bytes_to_hex, hex_to_bytes};

pub enum BytesEncoding {
    HEX,
    UTF8
}

pub struct Bytes {
    value: Vec<u8>
}

impl Bytes {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        return Bytes {
            value: bytes
        };
    }

    pub fn from_str(s: &str, encoding: BytesEncoding) -> Self {
        match encoding {
            BytesEncoding::HEX => {
                return Bytes {
                    value: hex_to_bytes(s)
                }
            }
            BytesEncoding::UTF8 => {
                return Bytes {
                    value: s.as_bytes().to_vec()
                }
            }
        }
    }

    pub fn to_hex_string(&self) -> String {
        return bytes_to_hex(&self.value);
    }

    pub fn to_utf8_string(&self) -> String {
        return String::from_utf8(self.value.to_vec()).unwrap();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        return self.value.to_vec();
    }
}

impl ABISerializableObject for Bytes {
    fn get_abi_name(&self) -> String {
        return "bytes".to_string();
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_varuint32(self.value.len().try_into().unwrap());
        encoder.write_array(&self.value);
    }

    fn to_json(&self) -> JSONValue {
        return JSONValue::String(self.to_hex_string());
    }
}
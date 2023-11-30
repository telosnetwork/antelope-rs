use std::fmt;
use std::collections::BTreeMap;
use fmt::{Display, Formatter};
use crate::serializer::encoder::ABIEncoder;

pub mod bytes;
pub mod checksum;
pub mod integer;
pub mod key_type;
pub mod name;
pub mod permission_level;
pub mod private_key;
pub mod public_key;
pub mod signature;
pub mod string;

pub trait ABISerializableObject {
    fn get_abi_name(&self) -> String;
    fn to_abi(&self, encoder: &mut ABIEncoder);
    fn to_json(&self) -> JSONValue;
    fn equals(&self, other: Box<dyn ABISerializableObject>) -> bool {
        if self.get_abi_name() != other.get_abi_name() {
            return false;
        }

        let my_json = self.to_json();
        let other_json = other.to_json();
        return my_json.to_string() == other_json.to_string();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum JSONValue {
    String(String),
    Bool(bool),
    UInt64(u64),
    Object(Box<BTreeMap<String, JSONValue>>),
}

impl Display for JSONValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            JSONValue::String(s) => write!(f, "String({})", s),
            JSONValue::Bool(b) => write!(f, "Bool({})", b),
            JSONValue::UInt64(u) => write!(f, "UInt64({})", u),
            JSONValue::Object(map) => {
                write!(f, "Object(")?;
                for (key, value) in map.iter() {
                    write!(f, "{}: {}, ", key, value)?;
                }
                write!(f, ")")
            }
        }
    }
}

pub fn to_str(v: &JSONValue) -> Result<String, String> {
    if let JSONValue::String(s) = v {
        return Ok(s.to_string());
    }

    let value_type = v.to_string();

    return Err(format!("Cannot get string from type {value_type}"));
}

use std::fmt;
use fmt::{Display, Formatter};
use crate::serializer::encoder::ABIEncoder;

pub mod integer;
pub mod name;
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
}

impl Display for JSONValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            JSONValue::String(_) => write!(f, "String"),
            JSONValue::Bool(_) => write!(f, "Bool"),
            JSONValue::UInt64(_) => write!(f, "UInt64"),
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
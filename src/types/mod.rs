use std::fmt;
use fmt::{Display, Formatter};

pub mod name;
pub mod uint64;

pub trait AntelopeType {
    fn deserialize(&self) -> AntelopeValue;
    fn serialize(&self) -> Vec<u8>;
}

pub enum AntelopeValue {
    String(String),
    Bool(bool),
    UInt64(u64),
}

impl Display for AntelopeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AntelopeValue::String(_) => write!(f, "String"),
            AntelopeValue::Bool(_) => write!(f, "Bool"),
            AntelopeValue::UInt64(_) => write!(f, "UInt64"),
        }
    }
}

pub fn to_str(v: &AntelopeValue) -> Result<String, String> {
    if let AntelopeValue::String(s) = v {
        return Ok(s.to_string());
    }

    let value_type = v.to_string();

    return Err(format!("Cannot get string from type {value_type}"));
}
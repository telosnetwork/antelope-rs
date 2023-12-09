use std::collections::HashMap;
use crate::chain::ABISerializableObject;

pub enum ABISerializable {
    ABISerializableObject(Box<dyn ABISerializableObject>),
    String(String),
    Bool(bool),
    ABISerializableArray(Vec<ABISerializable>),
    ABISerializableMap(HashMap<String, ABISerializable>)
}
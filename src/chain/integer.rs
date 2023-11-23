use std::string::ToString;
use crate::chain::{ABISerializableObject, JSONValue};
use crate::serializer::encoder::ABIEncoder;

pub struct UInt64 {
    ABI_NAME: String,
    value: u64
}

impl UInt64 {
}

impl ABISerializableObject for UInt64 {
    fn get_abi_name(&self) -> String {
        return "uint64".to_string();
    }

    fn to_abi(&self, mut encoder: &mut ABIEncoder) {
        encoder.write_array(self.value.to_le_bytes().to_vec());
    }

    fn to_json(&self) -> JSONValue {
        return JSONValue::UInt64(self.value);
    }

    fn equals(&self, other: Box<dyn ABISerializableObject>) -> bool {
        return self.get_abi_name() == other.get_abi_name() && self.to_json() == other.to_json();
    }
}

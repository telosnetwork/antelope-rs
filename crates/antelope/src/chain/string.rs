use crate::chain::{ABISerializableObject, JSONValue};
use crate::serializer::encoder::ABIEncoder;

pub struct StringType {
    value: String
}

impl StringType {
    pub fn from(s: &str) -> Self {
        return StringType {
            value: s.to_string()
        }
    }
}

impl ABISerializableObject for StringType {
    fn get_abi_name(&self) -> String {
        return "string".to_string();
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_string(self.value.to_string());
    }

    fn to_json(&self) -> JSONValue {
        return JSONValue::String(self.value.to_string());
    }
}
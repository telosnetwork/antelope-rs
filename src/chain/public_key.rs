use crate::chain::{ABISerializableObject, JSONValue};
use crate::serializer::encoder::ABIEncoder;

pub struct PublicKey {
    value: Vec<u8>,
}

impl PublicKey {

    pub fn from(key: &str) -> Self {
        return PublicKey {
            value: Vec::new()
        }
    }

}

impl ABISerializableObject for PublicKey {
    fn get_abi_name(&self) -> String {
        return "public_key".to_string();
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        encoder.write_array(self.value.to_vec());
    }

    fn to_json(&self) -> JSONValue {
        todo!()
    }

    fn equals(&self, obj: Box<dyn ABISerializableObject>) -> bool {
        todo!()
    }
}
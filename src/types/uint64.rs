use crate::types::{AntelopeType, AntelopeValue};

pub struct UInt64 {
    pub value: u64,
}

impl UInt64 {

}

impl AntelopeType for UInt64 {
    fn deserialize(&self) -> AntelopeValue {
        return AntelopeValue::UInt64(self.value);
    }

    fn serialize(&self) -> Vec<u8> {
        return self.value.to_le_bytes().to_vec();
    }
}
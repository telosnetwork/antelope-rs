use crate::types::AntelopeType;

pub fn encode(values: Vec<dyn AntelopeType>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for v in values.iter() {
        bytes.append(&mut v.serialize());
    }
    return bytes;
}
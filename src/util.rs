use hex::decode;

pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    return decode(hex).unwrap();
}
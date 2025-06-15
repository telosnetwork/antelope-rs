use hex::decode;
pub mod mock_provider;
pub mod ship_types;

/// used in tests
#[allow(dead_code)]
pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    decode(hex).unwrap()
}

use antelope_rs::chain::{AntelopeType, to_str};
use antelope_rs::chain::uint64::UInt64;

#[test]
fn encode_uint64() {
    let expected_int: u64 = 6712742083569909760;
    let uint64 = UInt64 {
        value: expected_int
    };
    let encoded = uint64.serialize();
    let decoded = uint64.deserialize();
    let expected_bytes = expected_int.to_le_bytes().to_vec();
    assert_eq!(encoded, expected_bytes, "Bytes did not match");
    let decoded_as_str = to_str(&decoded);
    assert_eq!(uint64.value, expected_int, "UInt64's value did not match expected u64 value");
    assert!(decoded_as_str.is_err(), "Converting AntelopeType::UInt64 to_str should fail");
}
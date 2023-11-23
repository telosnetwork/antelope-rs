use antelope_rs::chain::{AntelopeType, to_str};
use antelope_rs::chain::string::AntelopeString;
use antelope_rs::util;

#[test]
fn encode_name() {
    let test_str = "hello world";
    let string_hex = "0b68656c6c6f20776f726c64";
    let expected_bytes = util::hex_to_bytes(string_hex);

    let str_type = AntelopeString {
        value: test_str.to_string()
    };
    let encoded = str_type.serialize();
    let decoded = str_type.deserialize();
    assert_eq!(encoded, expected_bytes, "Encoded bytes did not equal expected bytes");
    let decoded_as_str = to_str(&decoded);
    assert!(decoded_as_str.is_ok(), "Failure converting AntelopeValue to_str");
    assert_eq!(decoded_as_str.unwrap(), test_str.to_string(), "Deserizalized string did not match");
}
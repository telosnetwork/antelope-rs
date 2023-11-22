use antelope_rs::types::{AntelopeType, to_str};
use antelope_rs::types::name::Name;
use antelope_rs::util;

#[test]
fn encode_name() {
    let name_str = "foobar";
    let name_hex = "000000005c73285d";
    let expected_int: u64 = 6712742083569909760;
    let name = Name::from_str(name_str);
    let encoded = name.serialize();
    let decoded = name.deserialize();
    let expected_bytes = util::hex_to_bytes(name_hex);
    assert_eq!(encoded, expected_bytes, "Bytes did not match");
    let decoded_as_str = to_str(&decoded);
    assert_eq!(name.uint64.value, expected_int, "Name's u64 value did not match expected u64 value");
    assert!(decoded_as_str.is_ok(), "Failure converting AntelopeValue to_str");
    assert_eq!(decoded_as_str.unwrap(), name_str.to_string(), "Deserizalized string did not match");
}
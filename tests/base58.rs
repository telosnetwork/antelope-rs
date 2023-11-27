use antelope_rs::{base58, util};
use antelope_rs::chain::key_type::KeyType;
use antelope_rs::util::hex_to_bytes;

#[test]
fn decode () {
    assert_eq!(base58::decode("StV1DL6CwTryKyV", None).unwrap(), hex::decode("68656c6c6f20776f726c64").unwrap());
    assert_eq!(base58::decode("1111", None).unwrap(), hex::decode("00000000").unwrap());
    let d1 = base58::decode("000", None);
    assert!(d1.is_err());
    let d2 = base58::decode("0", Some(1));
    assert!(d2.is_err());
    let d3 = base58::decode("zzz", Some(2));
    assert!(d3.is_err());
}

#[test]
fn encode() {
    assert_eq!(base58::encode(String::from("hello world").into_bytes()), String::from("StV1DL6CwTryKyV"));
    assert_eq!(base58::encode(util::hex_to_bytes("0000")), String::from("11"));
}

#[test]
fn decode_check() {
    assert_eq!(
        base58::decode_check("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu").unwrap(),
        hex_to_bytes("80d25968ebfce6e617bdb839b5a66cfc1fdd051d79a91094f7baceded449f84333")
    );
    let decode_result = base58::decode_check("5KQVfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu");
    assert!(decode_result.is_err());
}

#[test]
fn decode_ripemd160_check() {
    assert_eq!(
        base58::decode_ripemd160_check("6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin", None, None).unwrap(),
        hex_to_bytes("02caee1a02910b18dfd5d9db0e8a4bc90f8dd34cedbbfb00c6c841a2abb2fa28cc"),
    );
    let decode_result_1 = base58::decode_ripemd160_check("6RrVujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin", None, None);
    assert!(decode_result_1.is_err());
    assert_eq!(
        base58::decode_ripemd160_check(
            "6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs",
            Some(33),
            Some(KeyType::K1)
        ).unwrap(),
        hex_to_bytes("02caee1a02910b18dfd5d9db0e8a4bc90f8dd34cedbbfb00c6c841a2abb2fa28cc")
    );
    assert_eq!(
        base58::decode_ripemd160_check(
            "6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs",
            Some(33),
            Some(KeyType::K1)
        ).unwrap(),
        hex_to_bytes("02caee1a02910b18dfd5d9db0e8a4bc90f8dd34cedbbfb00c6c841a2abb2fa28cc")
    );
    let decode_result_2 = base58::decode_ripemd160_check(
        "6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs",
        None,
        None
    );

    assert!(decode_result_2.is_err());
}
#[test]
fn encode_check () {
    assert_eq!(
        base58::encode_check(
            hex_to_bytes("80d25968ebfce6e617bdb839b5a66cfc1fdd051d79a91094f7baceded449f84333")
        ),
        "5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu"
    );
}

#[test]
fn encode_ripemd160_check() {
    assert_eq!(
        base58::encode_ripemd160_check(
            hex_to_bytes("02caee1a02910b18dfd5d9db0e8a4bc90f8dd34cedbbfb00c6c841a2abb2fa28cc"),
            None
        ),
        "6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin"
    );
    assert_eq!(
        base58::encode_ripemd160_check(
            hex_to_bytes("02caee1a02910b18dfd5d9db0e8a4bc90f8dd34cedbbfb00c6c841a2abb2fa28cc"),
            Some(KeyType::K1.to_string().as_str())
        ),
        "6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs"
    )
}
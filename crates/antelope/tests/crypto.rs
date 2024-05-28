use antelope::chain::{Decoder, Encoder};
use antelope::util::bytes_to_hex;
use antelope::{
    chain::{key_type::KeyType, private_key::PrivateKey, public_key::PublicKey},
    util::hex_to_bytes,
};

#[test]
fn private_key_encoding() {
    let k1_key =
        PrivateKey::from_str("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu", false).unwrap();
    assert!(matches!(k1_key.key_type, KeyType::K1));
    assert_eq!(
        k1_key.to_wif().unwrap(),
        String::from("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu")
    );
    assert_eq!(
        k1_key.to_string(),
        "PVT_K1_2be6BwD56MHeVD4P95bRLdnP3oB3P4QRAXAsSKh4N8Xu6d4Aux"
    );
    assert_eq!(
        k1_key.to_hex(),
        "d25968ebfce6e617bdb839b5a66cfc1fdd051d79a91094f7baceded449f84333"
    );

    let r1_key = PrivateKey::from_str(
        "PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm",
        false,
    )
    .unwrap();
    assert_eq!(
        r1_key.to_string(),
        "PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm"
    );

    let result = r1_key.to_wif();
    assert!(result.is_err(), "R1 Key should Err when to_wif is called");
}

#[test]
fn public_key_encoding() {
    let k1_key =
        PublicKey::new_from_str("PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs")
            .unwrap();
    assert!(matches!(k1_key.key_type, KeyType::K1));
    assert_eq!(
        k1_key.to_string(),
        "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs"
    );
    assert_eq!(
        k1_key.to_legacy_string(Option::from("EOS")).unwrap(),
        "EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin"
    );
    assert_eq!(
        PublicKey::new_from_str("EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin")
            .unwrap()
            .to_string(),
        "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs"
    );
    assert_eq!(
        k1_key.to_hex_string(),
        "02caee1a02910b18dfd5d9db0e8a4bc90f8dd34cedbbfb00c6c841a2abb2fa28cc"
    );
    let r1_key =
        PublicKey::new_from_str("PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu")
            .unwrap();
    assert_eq!(
        r1_key.to_string(),
        "PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu"
    );
    let legacy_result = r1_key.to_legacy_string(None);
    assert!(legacy_result.is_err());

    let public_key =
        PublicKey::new_from_str("EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin").unwrap();
    let encoded_key = bytes_to_hex(&Encoder::pack(&public_key));
    assert_eq!(
        encoded_key,
        "0002caee1a02910b18dfd5d9db0e8a4bc90f8dd34cedbbfb00c6c841a2abb2fa28cc"
    );
    let data_bytes = hex_to_bytes(encoded_key.as_str());
    let mut decoder = Decoder::new(data_bytes.as_slice());
    let mut decoded_key = PublicKey::default();
    decoder.unpack(&mut decoded_key);
    assert_eq!(decoded_key.to_string(), public_key.to_string());
}

#[test]
fn public_key_prefix() {
    let priv_key =
        PrivateKey::from_str("5J4zo6Af9QnAeJmNEQeAR4MNhaG7SKVReAYgZC8655hpkbbBscr", false).unwrap();
    let pub_key = priv_key.to_public();
    assert_eq!(
        pub_key.to_string(),
        "PUB_K1_87DUhBcZrLhyFfBVDyu1iWZJUGURqbk6CQxwv5g6iWUD2X45Hv"
    );
    assert_eq!(
        pub_key.to_legacy_string(None).unwrap(),
        "EOS87DUhBcZrLhyFfBVDyu1iWZJUGURqbk6CQxwv5g6iWUCy9dCUJ"
    );
    assert_eq!(
        pub_key.to_legacy_string(Option::from("FIO")).unwrap(),
        "FIO87DUhBcZrLhyFfBVDyu1iWZJUGURqbk6CQxwv5g6iWUCy9dCUJ"
    )
}

#[test]
fn public_from_private() {
    let priv_key =
        PrivateKey::from_str("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu", false).unwrap();
    let pub_key = priv_key.to_public();
    assert_eq!(
        pub_key.to_string(),
        "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs"
    );
    let r1_priv_key = PrivateKey::from_str(
        "PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm",
        false,
    )
    .unwrap();
    let r1_pub_key = r1_priv_key.to_public();
    assert_eq!(
        r1_pub_key.to_string(),
        "PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu"
    )
}

#[test]
fn sign_and_verify() {
    let priv_key =
        PrivateKey::from_str("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu", false).unwrap();
    let pub_key =
        PublicKey::new_from_str("PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs")
            .unwrap();
    let message = String::from("I like turtles").into_bytes();
    let signature = priv_key.sign_message(&message);
    assert!(signature.verify_message(&message, &pub_key));
    assert!(!signature.verify_message(&b"beef".to_vec(), &pub_key));
    assert!(!signature.verify_message(
        &message,
        &PublicKey::new_from_str("EOS7HBX4f8UknP5NNoX8ixCx4YrA8JcPhGbuQ7Xem8gmWg1nviTqR").unwrap()
    ));
    // r1
    let priv_key2 = PrivateKey::from_str(
        "PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm",
        false,
    )
    .unwrap();
    let pub_key2 =
        PublicKey::new_from_str("PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu")
            .unwrap();
    let signature2 = priv_key2.sign_message(&message);
    assert_eq!(signature2.verify_message(&message, &pub_key2), true);
}

#[test]
fn sign_and_recover() {
    let key =
        PrivateKey::from_str("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu", false).unwrap();
    let message = b"I like turtles".to_vec();
    let signature = key.sign_message(&message);
    let recovered_key = signature.recover_message(&message);
    let recovered_key_failure = signature.recover_message(&b"beef".to_vec());
    assert_eq!(
        recovered_key.to_string(),
        "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs"
    );
    assert_eq!(
        recovered_key.to_legacy_string(Some("EOS")).unwrap(),
        "EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin"
    );
    assert_eq!(
        recovered_key.to_legacy_string(Some("FIO")).unwrap(),
        "FIO6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin"
    );
    assert_ne!(
        recovered_key_failure.to_string(),
        "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs"
    );

    let r1_private_key = PrivateKey::from_str(
        "PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm",
        false,
    )
    .unwrap();
    let r1_signature = r1_private_key.sign_message(&message);
    let recovered_r1_key = r1_signature.recover_message(&message);
    assert_eq!(
        recovered_r1_key.to_string(),
        "PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu"
    );
}
#[test]
fn shared_secrets() {
    let priv1 =
        PrivateKey::from_str("5KGNiwTYdDWVBc9RCC28hsi7tqHGUsikn9Gs8Yii93fXbkYzxGi", false).unwrap();
    let priv2 =
        PrivateKey::from_str("5Kik3tbLSn24ScHFsj6GwLkgd1H4Wecxkzt1VX7PBBRDQUCdGFa", false).unwrap();
    let pub1 = PublicKey::new_from_str("PUB_K1_7Wp9pzhtTfN3jSyQDCktKLqxdTAcAfgT2RrVpE6KThZraa381H")
        .unwrap();
    let pub2 = PublicKey::new_from_str("PUB_K1_6P8aGPEP79815rKGQ1dbc9eDxoEjatX7Lp696ve5tinnfwJ6nt")
        .unwrap();
    let expected =
        "def2d32f6b849198d71118ef53dbc3b679fe2b2c174ee4242a33e1a3f34c46fcbaa698fb599ca0e36f555dde2ac913a10563de2c33572155487cd8b34523de9e";
    let secret1 = priv1.shared_secret(&pub2);
    assert_eq!(secret1.data.to_vec(), hex_to_bytes(expected));
    let secret2 = priv2.shared_secret(&pub1);
    assert_eq!(secret2.data.to_vec(), hex_to_bytes(expected));
}

#[test]
fn key_generation() {
    let k1_key = PrivateKey::random(KeyType::K1);
    let r1_key = PrivateKey::random(KeyType::R1);

    assert!(k1_key.is_ok(), "Failed to generate k1 key");
    assert!(r1_key.is_ok(), "Failed to generate r1 key");

    /*
    let bad_key = PrivateKey::random(KeyType::??
    assert.throws(() => {
        PrivateKey.generate("XX")
    })
     */
}

#[test]
fn key_errors() {
    let invalid_private_key_result = PrivateKey::from_str(
        "PVT_K1_2be6BwD56MHeVD4P95bRLdnP3oB3P4QRAXAsSKh4N8Xu6d4Auz",
        false,
    );
    assert!(
        invalid_private_key_result.is_err(),
        "Invalid private key checksum should fail"
    );

    let empty_private_key_result = PrivateKey::from_str("", false);
    assert!(
        empty_private_key_result.is_err(),
        "Empty private key should fail"
    );
    let invalid_ok_private_key_result = PrivateKey::from_str(
        "PVT_K1_2be6BwD56MHeVD4P95bRLdnP3oB3P4QRAXAsSKh4N8Xu6d4Auz",
        true,
    );
    assert!(
        invalid_ok_private_key_result.is_ok(),
        "Should not fail if ignore_checksum = true"
    );
    assert_eq!(
        invalid_ok_private_key_result.unwrap().to_string(),
        "PVT_K1_2be6BwD56MHeVD4P95bRLdnP3oB3P4QRAXAsSKh4N8Xu6d4Aux"
    );
    let invalid_wif_private_key_result_enforce_checksum =
        PrivateKey::from_str("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zxx", false);
    assert!(
        invalid_wif_private_key_result_enforce_checksum.is_err(),
        "Should fail with invalid wif key"
    );

    let invalid_wif_private_key_result_no_checksum =
        PrivateKey::from_str("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zxx", true);
    assert!(
        invalid_wif_private_key_result_no_checksum.is_ok(),
        "Should not fail with invalid wif key if ignore_checksum = true"
    );
    assert_eq!(
        invalid_wif_private_key_result_no_checksum
            .unwrap()
            .to_wif()
            .unwrap(),
        "5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu"
    );
    let valid_failing_checksum = PrivateKey::from_str(
        "PVT_K1_ApBgGcJ2HeGR3szXA9JJptGCWUbSwewtGsxm3DVr86pJtb5V",
        true,
    );
    assert!(
        valid_failing_checksum.is_ok(),
        "Invalid checksum should pass if ignore_checksum = false"
    );
    let failing_checksum = PrivateKey::from_str(
        "PVT_K1_ApBgGcJ2HeGR3szXA9JJptGCWUbSwewtGsxm3DVr86pJtb5V",
        false,
    );
    assert!(failing_checksum.is_err(), "Invalid checksum should fail");
}

#[test]
fn key_generation2() {
    let key = PrivateKey::random(KeyType::K1).unwrap();
    let key_from_key = PrivateKey::from_str(key.to_string().as_str(), false);
    assert!(key_from_key.is_ok());
}

use antelope_rs::chain::private_key::PrivateKey;
use antelope_rs::chain::public_key::PublicKey;
use antelope_rs::chain::key_type::KeyType;

#[test]
fn private_key_encoding() {
    let k1_key = PrivateKey::from_str("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu");
    assert!(matches!(k1_key.key_type, KeyType::K1));
    assert_eq!(k1_key.to_wif().unwrap(), String::from("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu"));
    assert_eq!(k1_key.to_string(), "PVT_K1_2be6BwD56MHeVD4P95bRLdnP3oB3P4QRAXAsSKh4N8Xu6d4Aux");
    assert_eq!(k1_key.to_hex(), "d25968ebfce6e617bdb839b5a66cfc1fdd051d79a91094f7baceded449f84333");

    let r1_key = PrivateKey::from_str("PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm");
    assert_eq!(r1_key.to_string(), "PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm");

    let result = r1_key.to_wif();
    assert!(result.is_err(), "R1 Key should Err when to_wif is called");
}

#[test]
fn public_key_encoding() {
    let k1_key = PublicKey::from_str("PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs");
    assert!(matches!(k1_key.key_type, KeyType::K1));
    assert_eq!(k1_key.to_string(), "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs");
    assert_eq!(k1_key.to_legacy_string(Option::from("EOS")).unwrap(), "EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin");
    assert_eq!(
        PublicKey::from_str("EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin").to_string(),
        "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs"
    );
    assert_eq!(k1_key.to_hex_string(), "02caee1a02910b18dfd5d9db0e8a4bc90f8dd34cedbbfb00c6c841a2abb2fa28cc");
    let r1_key = PublicKey::from_str("PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu");
    assert_eq!(r1_key.to_string(), "PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu");
    let legacy_result = r1_key.to_legacy_string(None);
    assert!(legacy_result.is_err());
}



#[test]
fn public_key_prefix() {
    let priv_key = PrivateKey::from_str("5J4zo6Af9QnAeJmNEQeAR4MNhaG7SKVReAYgZC8655hpkbbBscr");
    let pub_key = priv_key.to_public();
    assert_eq!(pub_key.to_string(), "PUB_K1_87DUhBcZrLhyFfBVDyu1iWZJUGURqbk6CQxwv5g6iWUD2X45Hv");
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
    let priv_key = PrivateKey::from_str("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu");
    let pub_key = priv_key.to_public();
    assert_eq!(pub_key.to_string(), "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs");
    let r1_priv_key = PrivateKey::from_str("PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm");
    let r1_pub_key = r1_priv_key.to_public();
    assert_eq!(
        r1_pub_key.to_string(),
        "PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu"
    )
}

#[test]
fn sign_and_verify() {
    let priv_key = PrivateKey::from_str("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu");
    let pub_key = PublicKey::from_str("PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs");
    let message = String::from("I like turtles").into_bytes();
    let signature = priv_key.sign_message(&message);
    //assert_eq!(signature.verify_message(message, pub_key), true);
    /*
    assert.equal(signature.verifyMessage("beef", pub_key), false)
    assert.equal(
        signature.verifyMessage(
            message,
            PublicKey.from("EOS7HBX4f8UknP5NNoX8ixCx4YrA8JcPhGbuQ7Xem8gmWg1nviTqR")
        ),
        false
    )
    // r1
    const privKey2 = PrivateKey.from(
        "PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm"
    )
    const pubKey2 = PublicKey.from("PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu")
    const signature2 = privKey2.signMessage(message)
    assert.equal(signature2.verifyMessage(message, pubKey2), true)
    */
}

/*
        test("sign and recover", function () {
            const key = PrivateKey.from("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu")
            const message = Bytes.from("I like turtles", "utf8")
            const signature = key.signMessage(message)
            const recoveredKey = signature.recoverMessage(message)
            assert.equal(
                recoveredKey.toString(),
                "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs"
            )
            assert.equal(
                recoveredKey.toLegacyString(),
                "EOS6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin"
            )
            assert.equal(
                recoveredKey.toLegacyString("FIO"),
                "FIO6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeABhJRin"
            )
            assert.notEqual(
                signature.recoverMessage("beef").toString(),
                "PUB_K1_6RrvujLQN1x5Tacbep1KAk8zzKpSThAQXBCKYFfGUYeACcSRFs"
            )
            const r1Key = PrivateKey.from("PVT_R1_2dSFGZnA4oFvMHwfjeYCtK2MLLPNYWgYRXrPTcnTaLZFkDSELm")
            const r1Signature = r1Key.signMessage(message)
            assert.equal(
                r1Signature.recoverMessage(message).toString(),
                "PUB_R1_8E46r5HiQF84o6V8MWQQg1vPpgfjYA4XDqT6xbtaaebxw7XbLu"
            )
        })

        test("shared secrets", function () {
            const priv1 = PrivateKey.from("5KGNiwTYdDWVBc9RCC28hsi7tqHGUsikn9Gs8Yii93fXbkYzxGi")
            const priv2 = PrivateKey.from("5Kik3tbLSn24ScHFsj6GwLkgd1H4Wecxkzt1VX7PBBRDQUCdGFa")
            const pub1 = PublicKey.from("PUB_K1_7Wp9pzhtTfN3jSyQDCktKLqxdTAcAfgT2RrVpE6KThZraa381H")
            const pub2 = PublicKey.from("PUB_K1_6P8aGPEP79815rKGQ1dbc9eDxoEjatX7Lp696ve5tinnfwJ6nt")
            const expected =
                "def2d32f6b849198d71118ef53dbc3b679fe2b2c174ee4242a33e1a3f34c46fc" +
                "baa698fb599ca0e36f555dde2ac913a10563de2c33572155487cd8b34523de9e"
            assert.equal(priv1.sharedSecret(pub2), expected)
            assert.equal(priv2.sharedSecret(pub1), expected)
        })

        test("key generation", function () {
            assert.doesNotThrow(() => {
                PrivateKey.generate("R1")
            })
            assert.doesNotThrow(() => {
                PrivateKey.generate("K1")
            })
            assert.throws(() => {
                PrivateKey.generate("XX")
            })
        })

        test("key errors", function () {
            try {
                PrivateKey.from("PVT_K1_2be6BwD56MHeVD4P95bRLdnP3oB3P4QRAXAsSKh4N8Xu6d4Auz")
                assert.fail()
            } catch (error) {
                assert.ok(error instanceof Base58.DecodingError)
                assert.equal(error.code, Base58.ErrorCode.E_CHECKSUM)
                assert.equal(error.info.hash, "ripemd160")
                assert.deepEqual(Array.from(error.info.actual), [236, 129, 232, 27])
                assert.deepEqual(Array.from(error.info.expected), [236, 129, 232, 29])
            }
            const key1 = PrivateKey.fromString(
                "PVT_K1_2be6BwD56MHeVD4P95bRLdnP3oB3P4QRAXAsSKh4N8Xu6d4Auz",
                true
            )
            assert.equal(key1.toString(), "PVT_K1_2be6BwD56MHeVD4P95bRLdnP3oB3P4QRAXAsSKh4N8Xu6d4Aux")
            try {
                PrivateKey.from("5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zxx")
                assert.fail()
            } catch (error) {
                assert.ok(error instanceof Base58.DecodingError)
                assert.equal(error.code, Base58.ErrorCode.E_CHECKSUM)
                assert.equal(error.info.hash, "double_sha256")
            }
            const key2 = PrivateKey.fromString(
                "5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zxx",
                true
            )
            assert.equal(key2.toWif(), "5KQvfsPJ9YvGuVbLRLXVWPNubed6FWvV8yax6cNSJEzB4co3zFu")
            assert.doesNotThrow(() => {
                PrivateKey.fromString("PVT_K1_ApBgGcJ2HeGR3szXA9JJptGCWUbSwewtGsxm3DVr86pJtb5V", true)
            })
            assert.throws(() => {
                PrivateKey.fromString("PVT_K1_ApBgGcJ2HeGR3szXA9JJptGCWUbSwewtGsxm3DVr86pJtb5V")
            }, /Checksum mismatch/)
        })

        test("key generation", function () {
            assert.doesNotThrow(() => {
                const k = PrivateKey.generate("K1")
                PrivateKey.fromString(String(k))
            })
            assert.throws(() => {
                new PrivateKey(KeyType.K1, Bytes.random(31))
            })
            assert.throws(() => {
                const k = PrivateKey.generate("K1")
                k.data = Bytes.random(31)
                PrivateKey.fromString(String(k))
            })
        })

     */
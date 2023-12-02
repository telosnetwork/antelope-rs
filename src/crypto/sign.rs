use ecdsa::RecoveryId;
use sha2::{Sha256, Digest};
use crate::chain::key_type::KeyType;
use ecdsa::signature::{DigestSigner};
use k256::Secp256k1;
use p256::NistP256;
use crate::chain::signature::Signature;
use crate::crypto::curves::create_k1_field_bytes;

pub fn sign(secret: Vec<u8>, message: &Vec<u8>, key_type: KeyType) -> Result<Signature, String> {
    match key_type {
        KeyType::K1 => {
            let mut attempt = 1;
            loop {
                let signing_key = k256::ecdsa::SigningKey::from_bytes(&create_k1_field_bytes(secret.to_vec())).expect("invalid private key");

                let digest = Sha256::new().chain_update(&message);

                //  TODO: Explore further how to follow more closely the typescript model with canonical flag
                //    and personalization string being passed to sign method:
                //      sig = key.sign(message, {canonical: true, pers: [attempt++]})
                let signed: (ecdsa::Signature<Secp256k1>, RecoveryId) = signing_key.sign_digest(digest);

                let signature = signed.0;
                let recovery = signed.1;

                let r = signature.r().to_bytes().to_vec();
                let s = signature.s().to_bytes().to_vec();

                if Signature::is_canonical(&r, &s) {
                    return Signature::from_k1_signature(signature, recovery);
                }

                if attempt > 50 {
                    return Err(String::from("Failed over 50 times to find canonical signature"));
                }

                attempt += 1;
            }
        }
        KeyType::R1 => {
            let signing_key = p256::ecdsa::SigningKey::from_bytes(&create_k1_field_bytes(secret.to_vec())).expect("invalid private key");

            let digest = Sha256::new().chain_update(&message);

            //  TODO: Explore further how to follow more closely the typescript model with canonical flag
            //    and personalization string being passed to sign method:
            //      sig = key.sign(message, {canonical: true, pers: [attempt++]})
            let signed: (ecdsa::Signature<NistP256>, RecoveryId) = signing_key.sign_digest(digest);

            let signature = signed.0;
            let recovery = signed.1;

            return Signature::from_r1_signature(signature, recovery);
        }
    }
}

/*
export function sign(secret: Uint8Array, message: Uint8Array, type: string) {
    const curve = getCurve(type)
    const key = curve.keyFromPrivate(secret)
    let sig: ec.Signature
    let r: Uint8Array
    let s: Uint8Array
    if (type === 'K1') {
        let attempt = 1
        do {
            sig = key.sign(message, {canonical: true, pers: [attempt++]})
            r = sig.r.toArrayLike(Uint8Array as any, 'be', 32)
            s = sig.s.toArrayLike(Uint8Array as any, 'be', 32)
        } while (!isCanonical(r, s))
    } else {
        sig = key.sign(message, {canonical: true})
        r = sig.r.toArrayLike(Uint8Array as any, 'be', 32)
        s = sig.s.toArrayLike(Uint8Array as any, 'be', 32)
    }
    return {type, r, s, recid: sig.recoveryParam || 0}
}

/**
 * Here be dragons
 * - https://github.com/steemit/steem/issues/1944
 * - https://github.com/EOSIO/eos/issues/6699
 * @internal
 */
function isCanonical(r: Uint8Array, s: Uint8Array) {
    return (
        !(r[0] & 0x80) &&
        !(r[0] === 0 && !(r[1] & 0x80)) &&
        !(s[0] & 0x80) &&
        !(s[0] === 0 && !(s[1] & 0x80))
    )
}
 */
use ecdsa::signature::Verifier;
use crate::chain::key_type::KeyType;
use crate::chain::signature::Signature;
use crate::crypto::curves::{create_k1_field_bytes, create_r1_field_bytes};

pub fn verify(signature: &Signature, message: Vec<u8>, pub_key: Vec<u8>, key_type: KeyType) -> bool {
    // TODO: This more generic
    match key_type {
        KeyType::K1 => {
            let verifying_key = k256::ecdsa::VerifyingKey::from_sec1_bytes(pub_key.as_slice()).unwrap();
            let r_scalar = create_k1_field_bytes(signature.r());
            let s_scalar = create_k1_field_bytes(signature.s());

            let sig_result = k256::ecdsa::Signature::from_scalars(r_scalar, s_scalar).unwrap();
            return !verifying_key.verify(message.as_slice(), &sig_result).is_err();
        }
        KeyType::R1 => {
            let verifying_key = p256::ecdsa::VerifyingKey::from_sec1_bytes(pub_key.as_slice()).unwrap();
            let r_scalar = create_r1_field_bytes(signature.r());
            let s_scalar = create_r1_field_bytes(signature.s());

            let sig_result = p256::ecdsa::Signature::from_scalars(r_scalar, s_scalar).unwrap();
            return !verifying_key.verify(message.as_slice(), &sig_result).is_err();
        }
    }
}

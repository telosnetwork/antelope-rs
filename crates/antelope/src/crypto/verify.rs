use ecdsa::signature::{Verifier};
use k256::elliptic_curve::sec1::{ToEncodedPoint};
use crate::chain::key_type::KeyType;
use crate::chain::signature::Signature;
use crate::crypto::curves::{create_k1_field_bytes, create_r1_field_bytes};

pub fn verify_message(signature: &Signature, message_bytes: &Vec<u8>, pub_key: &Vec<u8>) -> bool {
    // TODO: This more generic
    let key_type = signature.key_type;
    match key_type {
        KeyType::K1 => {
            let public_key_point = k256::PublicKey::from_sec1_bytes(pub_key.as_slice()).unwrap().to_encoded_point(false);
            let verifying_key = k256::ecdsa::VerifyingKey::from_encoded_point(&public_key_point).unwrap();
            let r_scalar = create_k1_field_bytes(&signature.r());
            let s_scalar = create_k1_field_bytes(&signature.s());
            let sig_result = k256::ecdsa::Signature::from_scalars(r_scalar, s_scalar).unwrap();
            let verification = verifying_key.verify(message_bytes.as_slice(), &sig_result);
            verification.is_ok()
        }
        KeyType::R1 => {
            let public_key_point = p256::PublicKey::from_sec1_bytes(pub_key.as_slice()).unwrap().to_encoded_point(false);
            let verifying_key = p256::ecdsa::VerifyingKey::from_encoded_point(&public_key_point).unwrap();
            let r_scalar = create_r1_field_bytes(&signature.r());
            let s_scalar = create_r1_field_bytes(&signature.s());
            let sig_result = p256::ecdsa::Signature::from_scalars(r_scalar, s_scalar).unwrap();
            let verification = verifying_key.verify(message_bytes.as_slice(), &sig_result);
            verification.is_ok()
        }
    }
}

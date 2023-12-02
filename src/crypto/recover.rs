use ecdsa::RecoveryId;
use sha2::{Sha256, Digest};
use crate::chain::key_type::KeyType;
use crate::chain::public_key::PublicKey;
use crate::chain::signature::Signature;
use crate::crypto::curves::{create_k1_field_bytes, create_r1_field_bytes};

pub fn recover_message(signature: &Signature, message_bytes: &Vec<u8>) -> PublicKey {
    // TODO: This more generic
    let key_type = signature.key_type;
    match key_type {
        KeyType::K1 => {
            let r_scalar = create_k1_field_bytes(signature.r());
            let s_scalar = create_k1_field_bytes(signature.s());
            let sig = k256::ecdsa::Signature::from_scalars(r_scalar, s_scalar).unwrap();
            let digest = Sha256::new().chain_update(&message_bytes);
            let recovery_id= RecoveryId::from_byte(signature.recovery_id()).unwrap();
            let verifying_key = k256::ecdsa::VerifyingKey::recover_from_digest(
                digest,
                &sig,
                recovery_id
            ).unwrap();
            let compressed = verifying_key.to_encoded_point(true);
            let compressed_bytes = compressed.as_bytes();
            return PublicKey::from_bytes(compressed_bytes.to_vec(), key_type);
        }
        KeyType::R1 => {
            let r_scalar = create_r1_field_bytes(signature.r());
            let s_scalar = create_r1_field_bytes(signature.s());
            let sig = p256::ecdsa::Signature::from_scalars(r_scalar, s_scalar).unwrap();
            let digest = Sha256::new().chain_update(&message_bytes);
            let recovery_id= RecoveryId::from_byte(signature.recovery_id()).unwrap();
            let verifying_key = p256::ecdsa::VerifyingKey::recover_from_digest(
                digest,
                &sig,
                recovery_id
            ).unwrap();
            let compressed = verifying_key.to_encoded_point(true);
            let compressed_bytes = compressed.as_bytes();
            return PublicKey::from_bytes(compressed_bytes.to_vec(), key_type);
        }
    }
}

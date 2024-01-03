use crate::chain::key_type::KeyType;
use crate::crypto::curves::{create_k1_field_bytes, create_r1_field_bytes};

pub fn shared_secret(my_secret: &[u8], their_pub_key: &Vec<u8>, key_type: KeyType) -> Result<Vec<u8>, String> {
    match key_type {
        KeyType::K1 => {
            let secret_key = k256::SecretKey::from_bytes(&create_k1_field_bytes(my_secret)).expect("invalid private key");
            let their_public_key = k256::PublicKey::from_sec1_bytes(their_pub_key.as_slice()).unwrap();

            let shared_secret = k256::elliptic_curve::ecdh::diffie_hellman(
                secret_key.to_nonzero_scalar(),
                their_public_key.as_affine()
            );
            Ok(shared_secret.raw_secret_bytes().to_vec())
        }
        KeyType::R1 => {
            let secret_key = p256::SecretKey::from_bytes(&create_r1_field_bytes(my_secret)).expect("invalid private key");
            let their_public_key = p256::PublicKey::from_sec1_bytes(their_pub_key.as_slice()).unwrap();

            let shared_secret = p256::elliptic_curve::ecdh::diffie_hellman(
                secret_key.to_nonzero_scalar(),
                their_public_key.as_affine()
            );
            Ok(shared_secret.raw_secret_bytes().to_vec())
        }
    }
}
use k256;
use p256::{self, elliptic_curve::sec1::ToEncodedPoint};

use crate::chain::key_type::KeyType;

pub fn generate(curve_type: KeyType) -> Result<Vec<u8>, String> {
    // TODO: maybe these can use generic types to deduplicate code?
    match curve_type {
        KeyType::K1 => {
            let secret_key = k256::SecretKey::random(&mut rand::thread_rng());
            let scalar = k256::NonZeroScalar::from(secret_key);
            let public_key = k256::PublicKey::from_secret_scalar(&scalar);
            let encoded_point = public_key.to_encoded_point(true);
            Ok(encoded_point.as_bytes().to_vec())
        }
        KeyType::R1 => {
            let secret_key = p256::SecretKey::random(&mut rand::thread_rng());
            let scalar = p256::NonZeroScalar::from(secret_key);
            let public_key = p256::PublicKey::from_secret_scalar(&scalar);
            let encoded_point = public_key.to_encoded_point(true);
            Ok(encoded_point.as_bytes().to_vec())
        }
        KeyType::WA => Err("Unsupported key type".to_string()),
    }
}

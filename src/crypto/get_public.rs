use k256;
use p256;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use crate::chain::key_type::KeyType;
use crate::crypto::curves::{create_k1_field_bytes, create_r1_field_bytes};

pub fn get_public(priv_key: Vec<u8>, curve_type: KeyType) -> Result<Vec<u8>, String> {
    // TODO: maybe these can use generic types to deduplicate code?
    match curve_type {
        KeyType::K1 => {
            let secret_key = k256::SecretKey::from_bytes(&create_k1_field_bytes(&priv_key)).expect("invalid private key");
            let scalar = k256::NonZeroScalar::from(secret_key);
            let public_key = k256::PublicKey::from_secret_scalar(&scalar);
            let encoded_point = public_key.to_encoded_point(true);
            return Ok(encoded_point.as_bytes().to_vec());
        },
        KeyType::R1 => {
            let secret_key = p256::SecretKey::from_bytes(&create_r1_field_bytes(&priv_key)).expect("invalid private key");
            let scalar = p256::NonZeroScalar::from(secret_key);
            let public_key = p256::elliptic_curve::PublicKey::from_secret_scalar(&scalar);
            let encoded_point = public_key.to_encoded_point(true);
            return Ok(encoded_point.as_bytes().to_vec());
        },
    }
}

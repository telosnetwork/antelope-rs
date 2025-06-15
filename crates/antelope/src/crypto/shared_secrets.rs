use crate::{
    chain::key_type::KeyType,
    crypto::curves::{create_k1_field_bytes, create_r1_field_bytes},
};
use ecdsa::elliptic_curve::Error as ElipticCurveError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SharedSecretError {
    #[error(transparent)]
    ElipticCurve(#[from] ElipticCurveError),

    #[error("unsupported key type")]
    UnsupportedKeyType,
}

pub fn shared_secret(
    my_secret: &[u8],
    their_pub_key: &[u8],
    key_type: KeyType,
) -> Result<Vec<u8>, SharedSecretError> {
    match key_type {
        KeyType::K1 => {
            let secret_key =
                k256::SecretKey::from_bytes(&create_k1_field_bytes(my_secret))?;
            let their_public_key = k256::PublicKey::from_sec1_bytes(their_pub_key)?;
            let shared = k256::elliptic_curve::ecdh::diffie_hellman(
                secret_key.to_nonzero_scalar(),
                their_public_key.as_affine(),
            );
            Ok(shared.raw_secret_bytes().to_vec())
        }
        KeyType::R1 => {
            let secret_key =
                p256::SecretKey::from_bytes(&create_r1_field_bytes(my_secret))?;
            let their_public_key = p256::PublicKey::from_sec1_bytes(their_pub_key)?;
            let shared = p256::elliptic_curve::ecdh::diffie_hellman(
                secret_key.to_nonzero_scalar(),
                their_public_key.as_affine(),
            );
            Ok(shared.raw_secret_bytes().to_vec())
        }
        KeyType::WA => Err(SharedSecretError::UnsupportedKeyType),
    }
}

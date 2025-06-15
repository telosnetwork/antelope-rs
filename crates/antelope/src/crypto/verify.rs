use ecdsa::signature::Verifier;
use ecdsa::Error as ECSDAError;
use ecdsa::elliptic_curve::Error as ElipticCurveError;
use k256::elliptic_curve::sec1::ToEncodedPoint;

use crate::{
    chain::{key_type::KeyType, signature::Signature},
    crypto::curves::{create_k1_field_bytes, create_r1_field_bytes},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifyMessageError {
    #[error(transparent)]
    ECDSA(#[from] ECSDAError),

    #[error(transparent)]
    ElipticCurveError(#[from] ElipticCurveError),

    #[error("WA key type is unsupported")]
    UnsupportedKeyType
}

pub fn verify_message(signature: &Signature, message_bytes: &[u8], pub_key: &[u8]) -> Result<(), VerifyMessageError> {
    // TODO: This more generic
    let key_type = signature.key_type;
    match key_type {
        KeyType::K1 => {
            let public_key_point = k256::PublicKey::from_sec1_bytes(pub_key)?
                .to_encoded_point(false);
            let verifying_key = k256::ecdsa::VerifyingKey::from_encoded_point(&public_key_point)?;
            let r_scalar = create_k1_field_bytes(&signature.r());
            let s_scalar = create_k1_field_bytes(&signature.s());
            let sig_result = k256::ecdsa::Signature::from_scalars(r_scalar, s_scalar)?;

            verifying_key.verify(message_bytes, &sig_result)
                .map_err(VerifyMessageError::ECDSA)
        }
        KeyType::R1 => {
            let public_key_point = p256::PublicKey::from_sec1_bytes(pub_key)?
                .to_encoded_point(false);
            let verifying_key = p256::ecdsa::VerifyingKey::from_encoded_point(&public_key_point)?;
            let r_scalar = create_r1_field_bytes(&signature.r());
            let s_scalar = create_r1_field_bytes(&signature.s());
            let sig_result = p256::ecdsa::Signature::from_scalars(r_scalar, s_scalar)?;

            verifying_key.verify(message_bytes, &sig_result)
                .map_err(VerifyMessageError::ECDSA)
        }
        KeyType::WA => Err(VerifyMessageError::UnsupportedKeyType),
    }
}

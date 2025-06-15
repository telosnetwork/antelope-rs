use ecdsa::Error as EcdsaError;
use ecdsa::elliptic_curve::Error as EllipticCurveError;
use ecdsa::RecoveryId;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    chain::{key_type::KeyType, public_key::PublicKey, signature::Signature},
    crypto::curves::{create_k1_field_bytes, create_r1_field_bytes},
};

#[derive(Debug, Error)]
pub enum RecoverMessageError {
    #[error(transparent)]
    Ecdsa(#[from] EcdsaError),

    #[error(transparent)]
    EllipticCurve(#[from] EllipticCurveError),

    #[error("public key parse error: {0}")]
    PublicKey(String),

    #[error("invalid recovery_id byte")]
    InvalidRecoveryId,

    #[error("unsupported key type")]
    UnsupportedKeyType,
}

pub fn recover_message(
    signature: &Signature,
    message_bytes: &[u8],
) -> Result<PublicKey, RecoverMessageError> {
    match signature.key_type {
        KeyType::K1 => {
            let r = create_k1_field_bytes(&signature.r());
            let s = create_k1_field_bytes(&signature.s());
            let sig = k256::ecdsa::Signature::from_scalars(r, s)?;
            let digest = Sha256::new().chain_update(message_bytes);
            let rid = RecoveryId::from_byte(
                signature.recovery_id() - Signature::RECOVERY_ID_ADDITION,
            ).ok_or(RecoverMessageError::InvalidRecoveryId)?;
            let vk = k256::ecdsa::VerifyingKey::recover_from_digest(digest, &sig, rid)?;
            let bytes = vk.to_encoded_point(true).as_bytes().to_vec();
            Ok(PublicKey::from((bytes, KeyType::K1)))
        }
        KeyType::R1 => {
            let r = create_r1_field_bytes(&signature.r());
            let s = create_r1_field_bytes(&signature.s());
            let sig = p256::ecdsa::Signature::from_scalars(r, s)?;
            let digest = Sha256::new().chain_update(message_bytes);
            let rid = RecoveryId::from_byte(signature.recovery_id())
                .ok_or(RecoverMessageError::InvalidRecoveryId)?;
            let vk = p256::ecdsa::VerifyingKey::recover_from_digest(digest, &sig, rid)?;
            let bytes = vk.to_encoded_point(true).as_bytes().to_vec();
            Ok(PublicKey::from((bytes, KeyType::R1)))
        }
        KeyType::WA => Err(RecoverMessageError::UnsupportedKeyType),
    }
}

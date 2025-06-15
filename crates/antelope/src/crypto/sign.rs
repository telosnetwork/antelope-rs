use crate::{
    chain::{key_type::KeyType, signature::Signature},
    crypto::curves::{create_k1_field_bytes, create_r1_field_bytes},
};
use digest::{
    consts::U32,
    core_api::{CoreWrapper, CtVariableCoreWrapper},
    generic_array::ArrayLength,
};
use ecdsa::{
    elliptic_curve::{ops::Invert, subtle::CtOption, CurveArithmetic, Scalar},
    hazmat::{bits2field, DigestPrimitive, SignPrimitive},
    PrimeCurve, RecoveryId, SignatureSize,
};
use k256::ecdsa::signature::DigestSigner;
use sha2::{Digest, OidSha256, Sha256, Sha256VarCore};
use signature::Error as EcdsaError;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum SignError {
    #[error(transparent)]
    Ecdsa(#[from] EcdsaError),

    #[error("invalid private key")]
    InvalidKey,

    #[error("unsupported key type")]
    UnsupportedKeyType,

    #[error("reached max canonical signature checks: {0}")]
    MaxCanonicalAttempts(i8),

    #[error("signature construction failed: {0}")]
    SignatureBuild(String),
}

/// Sign `message` under `secret` for the given curve.
pub fn sign(
    secret: &[u8],
    message: &[u8],
    key_type: KeyType,
) -> Result<Signature, SignError> {
    match key_type {
        KeyType::K1 => {
            let mut attempt = 1i8;
            loop {
                let signing_key = k256::ecdsa::SigningKey::from_bytes(
                    &create_k1_field_bytes(secret),
                )
                .map_err(|_| SignError::InvalidKey)?;

                let pers = &attempt.to_be_bytes();
                let digest = Sha256::new().chain_update(message);

                let (sig, recid) = k1_sign_with_pers(signing_key, digest, pers)?;
                let r = sig.r().to_bytes().to_vec();
                let s = sig.s().to_bytes().to_vec();

                if Signature::is_canonical(&r, &s) {
                    return Signature::try_from((sig, recid))
                        .map_err(SignError::SignatureBuild);
                }

                if attempt % 10 == 0 {
                    info!("Failed {} times to find canonical signature", attempt);
                }

                if attempt > 100 {
                    return Err(SignError::MaxCanonicalAttempts(attempt));
                }

                attempt += 1;
            }
        }
        KeyType::R1 => {
            let signing_key = p256::ecdsa::SigningKey::from_bytes(
                &create_r1_field_bytes(secret),
            )
            .map_err(|_| SignError::InvalidKey)?;

            let digest = Sha256::new().chain_update(message);
            let (sig, recid) = signing_key.sign_digest(digest);

            Signature::try_from((sig, recid))
                .map_err(SignError::SignatureBuild)
        }
        KeyType::WA => Err(SignError::UnsupportedKeyType),
    }
}

/// Now accepts the `CoreWrapper` you get from `Sha256::new().chain_update(…)`.
fn k1_sign_with_pers<C>(
    signing_key: ecdsa::SigningKey<C>,
    digest: CoreWrapper<CtVariableCoreWrapper<Sha256VarCore, U32, OidSha256>>,
    pers: &[u8],
) -> signature::Result<(ecdsa::Signature<C>, RecoveryId)>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    let prehash = digest.finalize();
    let z = bits2field::<C>(prehash.as_slice())?;
    let (sig, recid) = signing_key
        .as_nonzero_scalar()
        .try_sign_prehashed_rfc6979::<C::Digest>(&z, pers)?;
    Ok((sig, recid.ok_or_else(EcdsaError::new)?))
}

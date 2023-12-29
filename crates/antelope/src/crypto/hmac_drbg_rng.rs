use digest::{Digest};
use hmac_drbg::HmacDRBG;
use hmac::Hmac;
use k256::{
    elliptic_curve::{rand_core::{RngCore, CryptoRng, Error as RandError}},
};
use p256::U32;
use sha2::Sha256;

pub struct HmacDRBGRng {
    drbg: HmacDRBG<Sha256>
}

impl HmacDRBGRng {
    pub(crate) fn create(entropy: &[u8], message: &Vec<u8>, pers: &[u8]) -> Self {
        let drbg = HmacDRBG::<Sha256>::new(entropy, message, pers);
        HmacDRBGRng { drbg }
    }
}

impl RngCore for HmacDRBGRng {
    fn next_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.fill_bytes(&mut buf);
        u32::from_ne_bytes(buf)
    }

    fn next_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill_bytes(&mut buf);
        u64::from_ne_bytes(buf)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        assert_eq!(dest.len(), 32, "fill_bytes only expecting a destination array of 32 bytes");
        self.drbg.generate::<U32>(Some(dest));
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RandError> {
        assert_eq!(dest.len(), 32, "try_fill_bytes only expecting a destination array of 32 bytes");
        self.drbg.generate::<U32>(Some(dest));
        Ok(())
    }
}

impl CryptoRng for HmacDRBGRng {}
use core::fmt;
use std::ffi::NulError;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;
use f128::f128;
use serde::de::{Visitor, Error as DeError};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::serializer::{Decoder, Encoder, Packer, PackerError};
use crate::check_unpack_len;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Float128 {
    pub f: f128,
}

impl Default for Float128 {
    fn default() -> Self {
        Self { f: f128::from(0u8) }
    }
}

impl FromStr for Float128 {
    type Err = NulError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        f128::parse(s).map(|f| f.into())
    }
}

impl From<f128> for Float128 {
    #[inline]
    fn from(f: f128) -> Self {
        Self { f }
    }
}

impl From<Float128> for f128 {
    #[inline]
    fn from(value: Float128) -> Self {
        value.f
    }
}

impl TryFrom<&[u8]> for Float128 {
    type Error = PackerError;
    fn try_from(raw: &[u8]) -> Result<Float128, PackerError> {
        let mut dec = Decoder::new(raw);
        let mut f = Float128::default();
        dec.unpack(&mut f)?;
        Ok(f)
    }
}

impl fmt::Display for Float128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.f)
    }
}

impl Packer for Float128 {
    #[inline]
    fn size(&self) -> usize {
        16
    }

    #[inline]
    fn pack(&self, enc: &mut Encoder) -> usize {
        enc.pack_raw(&self.f.into_inner());
        16
    }

    #[inline]
    fn unpack(&mut self, raw: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, raw, 16);

        let mut buf = [0u8; 16];
        buf.copy_from_slice(&raw[..16]);

        #[cfg(target_endian = "little")]
        {
            self.f = unsafe { std::mem::transmute::<[u8; 16], f128>(buf) };
        }

        #[cfg(target_endian = "big")]
        {
            self.f = f128::from(u128::from_le_bytes(buf));
        }

        Ok(16)
    }
}

impl Add for Float128 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Float128::from(self.f + rhs.f)
    }
}

impl Sub for Float128 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Float128::from(self.f - rhs.f)
    }
}

impl Mul for Float128 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Float128::from(self.f * rhs.f)
    }
}

impl Div for Float128 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Float128::from(self.f / rhs.f)
    }
}

impl Serialize for Float128 {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.serialize_str(&self.f.to_string())
    }
}

impl<'de> Deserialize<'de> for Float128 {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrVisitor;

        impl<'de> Visitor<'de> for StrVisitor {
            type Value = Float128;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a decimal string representing a 128-bit float")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                // f128 implements FromStr; propagate parse errors nicely
                Float128::from_str(v)
                    .map_err(|e| DeError::custom(e.to_string()))
            }
        }

        de.deserialize_str(StrVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializer::Encoder;
    use serde_json;

    #[test]
    fn default_is_zero() {
        assert_eq!(Float128::default(), Float128::from(f128::from(0u8)));
    }

    #[test]
    fn display_and_from_roundtrip() {
        let v = f128::from(42u8);
        let f = Float128::from(v);
        assert_eq!(f.to_string(), "42");
        let back: f128 = f.into();
        assert_eq!(back, v);
    }

    #[test]
    fn serde_string_roundtrip() {
        let v = Float128::from(f128::from(17u8));
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, format!("\"{}\"", v));
        let de: Float128 = serde_json::from_str(&json).unwrap();
        assert_eq!(de, v);
    }

    #[test]
    fn pack_unpack_roundtrip() {
        let orig = Float128::from(f128::from(99u8));
        let mut enc = Encoder::new(16);
        orig.pack(&mut enc);
        let buf = enc.get_bytes();
        let mut decoded = Float128::default();
        decoded.unpack(buf).unwrap();
        assert_eq!(decoded, orig);
    }

    #[test]
    fn arithmetic_ops() {
        let a = Float128::from(f128::from(2u8));
        let b = Float128::from(f128::from(3u8));

        assert_eq!(a + b, Float128::from(f128::from(5u8)));
        assert_eq!(b - a, Float128::from(f128::from(1u8)));
        assert_eq!(a * b, Float128::from(f128::from(6u8)));

        let div_expected = Float128::from(f128::from(3u8) / f128::from(2u8));
        assert_eq!(b / a, div_expected);
    }
}

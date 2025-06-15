//! varint.rs – variable-length 32-bit integers (unsigned / signed)

use std::{fmt::Display, ops::{Add, Div, Mul, Sub}, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{define_error, packer_error, serializer::{Decoder, Encoder, Packer, PackerError}};

/// Unsigned LEB128-encoded 32-bit integer.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct VarUint32 {
    pub n: u32,
    buf: [u8; 5],
    len: usize
}

impl From<u32> for VarUint32 {
    fn from(n: u32) -> Self {
        let mut val = n;
        let mut buf = [0; 5];
        let mut len = 0usize;
        if val == 0 {
            len = 1usize;
        } else {
            while val > 0 {
                let mut b = (val & 0x7f) as u8;
                val >>= 7;
                if val > 0 {
                    b |= 0x80;
                }
                buf[len] = b;
                len += 1;
            }
        }
        Self { n, buf, len }
    }
}

impl From<usize> for VarUint32 {
    fn from(value: usize) -> Self {
        VarUint32::from(value as u32)
    }
}

impl From<VarUint32> for u32 {
    fn from(value: VarUint32) -> Self {
        value.n
    }
}

impl From<VarUint32> for usize {
    fn from(value: VarUint32) -> Self {
        value.n as usize
    }
}

impl From<&VarUint32> for u32 {
    fn from(value: &VarUint32) -> Self {
        value.n
    }
}

impl From<&VarUint32> for usize {
    fn from(value: &VarUint32) -> Self {
        value.n as usize
    }
}

define_error!(VarUint32ParseError);

impl FromStr for VarUint32 {
    type Err = VarUint32ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(VarUint32::from(
            u32::from_str(s)
                .map_err(VarUint32ParseError::new)?
        ))
    }
}

impl TryFrom<&[u8]> for VarUint32 {
    type Error = PackerError;
    fn try_from(raw: &[u8]) -> Result<Self, PackerError> {
        let mut dec = Decoder::new(raw);
        let mut f = Self::default();
        dec.unpack(&mut f)?;
        Ok(f)
    }
}

impl Default for VarUint32 {
    fn default() -> Self {
        Self { n: 0, buf: [0; 5], len: 1}
    }
}

impl Packer for VarUint32 {
    fn size(&self) -> usize {
        self.len
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        enc.pack_raw(&self.buf[..self.len]);
        self.len
    }

    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        let mut shift = 0;
        let mut value = 0u32;
        let mut len = 0;
        for &b in data {
            value |= ((b & 0x7f) as u32) << shift;
            len += 1;
            if (b & 0x80) == 0 {
                break;
            }
            shift += 7;
            if shift >= 32 {
                return Err(packer_error!("malformed varuint32"));
            }
        }
        *self = value.into();
        Ok(len)
    }
}

impl Add for VarUint32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        VarUint32::from(self.n + rhs.n)
    }
}

impl Sub for VarUint32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        VarUint32::from(self.n - rhs.n)
    }
}

impl Mul for VarUint32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        VarUint32::from(self.n * rhs.n)
    }
}

impl Div for VarUint32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        VarUint32::from(self.n / rhs.n)
    }
}

impl Display for VarUint32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.n)
    }
}

/// Signed LEB128-encoded 32-bit integer (zig-zag mapped).
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct VarInt32 {
    pub n: i32,
    buf: [u8; 5],
    len: usize
}

impl VarInt32 {
    #[inline]
    fn zz_enc(v: i32) -> u32 {
        ((v << 1) ^ (v >> 31)) as u32
    }

    #[inline]
    fn zz_dec(v: u32) -> i32 {
        ((v >> 1) as i32) ^ -((v & 1) as i32)
    }
}

impl From<i32> for VarInt32 {
    fn from(n: i32) -> Self {
        let u: VarUint32 = Self::zz_enc(n).into();
        Self { n, buf: u.buf, len: u.len }
    }
}

impl From<isize> for VarInt32 {
    fn from(value: isize) -> Self {
        VarInt32::from(value as i32)
    }
}

impl From<VarInt32> for i32 {
    fn from(value: VarInt32) -> Self {
        value.n
    }
}

impl From<&VarInt32> for i32 {
    fn from(value: &VarInt32) -> Self {
        value.n
    }
}

define_error!(VarInt32ParseError);

impl FromStr for VarInt32 {
    type Err = VarInt32ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(VarInt32::from(
            i32::from_str(s)
                .map_err(VarInt32ParseError::new)?
        ))
    }
}

impl TryFrom<&[u8]> for VarInt32 {
    type Error = PackerError;
    fn try_from(raw: &[u8]) -> Result<Self, PackerError> {
        let mut dec = Decoder::new(raw);
        let mut f = Self::default();
        dec.unpack(&mut f)?;
        Ok(f)
    }
}

impl Default for VarInt32 {
    fn default() -> Self {
        Self { n: 0, buf: [0; 5], len: 1}
    }
}

impl Packer for VarInt32 {
    fn size(&self) -> usize {
        self.len
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        enc.pack_raw(&self.buf[..self.len]);
        self.len
    }

    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        let mut tmp = VarUint32::default();
        let len = tmp.unpack(data)?;
        *self = Self::zz_dec(tmp.n).into();
        Ok(len)
    }
}

impl Add for VarInt32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        VarInt32::from(self.n + rhs.n)
    }
}

impl Sub for VarInt32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        VarInt32::from(self.n - rhs.n)
    }
}

impl Mul for VarInt32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        VarInt32::from(self.n * rhs.n)
    }
}

impl Div for VarInt32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        VarInt32::from(self.n / rhs.n)
    }
}

impl Display for VarInt32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializer::Encoder;

    #[test]
    fn roundtrip_unsigned() {
        let vals = [
            0u32,
            1,
            127,
            128,
            255,
            256,
            16383,
            16384,
            2097151,
            2097152,
            268435455,
            268435456,
            u32::MAX,
        ];
        for &v in &vals {
            let vu: VarUint32 = v.into();
            let mut enc = Encoder::new(vu.size());
            vu.pack(&mut enc);
            let mut out = VarUint32::default();
            out.unpack(enc.get_bytes()).unwrap();
            assert_eq!(vu, out);
        }
    }

    #[test]
    fn roundtrip_signed() {
        let vals = [0, -1, 1, -123, 127, -128, 255, -255, i32::MIN, i32::MAX];
        for &v in &vals {
            let vi: VarInt32 = v.into();
            let mut enc = Encoder::new(vi.size());
            vi.pack(&mut enc);
            let mut out = VarInt32::default();
            out.unpack(enc.get_bytes()).unwrap();
            assert_eq!(vi, out);
        }
    }
}

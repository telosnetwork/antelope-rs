use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

use crate::chain::name::Name;
use crate::{check_unpack_len, packer_error};
use crate::serializer::{Decoder, Encoder, Packer, PackerError};

pub const ASSET_MAX_AMOUNT: i64 = (1 << 62) - 1;
pub const ASSET_MAX_PRECISION: u8 = 18;

#[inline]
pub fn is_valid_symbol_code(sym: u64) -> bool {
    let mut tmp = sym;
    if (sym >> 56) != 0 {
        return false;
    }
    for _ in 0..7 {
        let c = (tmp & 0xFF) as u8;
        if !c.is_ascii_uppercase() {
            return false;
        }
        tmp >>= 8;
        if (tmp & 0xFF) == 0 {
            break;
        }
    }
    tmp >>= 8;
    tmp == 0
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SymbolCode {
    value: u64,
}

impl SymbolCode {
    #[inline]
    pub fn value(&self) -> u64 {
        self.value
    }
}

impl From<SymbolCode> for u64 {
    #[inline]
    fn from(s: SymbolCode) -> u64 { s.value }
}


#[derive(Debug, Error)]
#[error("{0}")]
pub struct SymbolCodeError(String);

impl TryFrom<u64> for SymbolCode {
    type Error = SymbolCodeError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if !is_valid_symbol_code(value) {
            Err(SymbolCodeError("invalid symbol code".into()))
        } else {
            Ok(SymbolCode { value })
        }
    }
}

impl FromStr for SymbolCode {
    type Err = SymbolCodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = s.as_bytes();
        if raw.is_empty() {
            return Err(SymbolCodeError("empty symbol code".into()));
        }
        if raw.len() > 7 {
            return Err(SymbolCodeError("symbol code too long".into()));
        }
        let mut value = 0u64;
        for &c in raw.iter().rev() {
            if !c.is_ascii_uppercase() {
                return Err(SymbolCodeError(format!("invalid char '{}'", c as char)));
            }
            value = (value << 8) | c as u64;
        }
        SymbolCode::try_from(value)
    }
}

impl fmt::Display for SymbolCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tmp = self.value;
        let mut s = String::new();
        for _ in 0..7 {
            let c = (tmp & 0xFF) as u8;
            if c == 0 {
                break;
            }
            s.push(c as char);
            tmp >>= 8;
        }
        write!(f, "{s}")
    }
}

impl Packer for SymbolCode {
    fn size(&self) -> usize { 8 }
    fn pack(&self, enc: &mut Encoder) -> usize { self.value.pack(enc) }
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, data, 8);
        let mut val = 0u64;
        val.unpack(data)?;
        if !is_valid_symbol_code(val) {
            return Err(packer_error!("bad symbol code"));
        }
        self.value = val;
        Ok(8)
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    value: u64,
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct SymbolError(String);

pub fn str_to_symbol(input: &str) -> Result<u64, SymbolError> {
    let (prec, code) = input
        .split_once(',')
        .ok_or_else(|| SymbolError("expected 'precision,SYMBOL'".into()))?;
    let precision: u8 = prec
        .parse()
        .map_err(|_| SymbolError("invalid precision".into()))?;
    if !(1..=7).contains(&code.len())
        || !code.bytes().all(|b| b.is_ascii_uppercase())
    {
        return Err(SymbolError("invalid symbol code".into()));
    }
    let mut value = precision as u64;
    for (i, &b) in code.as_bytes().iter().enumerate() {
        value |= (b as u64) << (8 * (i + 1));
    }
    Ok(value)
}

impl From<u64> for Symbol {
    #[inline]
    fn from(value: u64) -> Self { Self { value } }
}

impl From<Symbol> for u64 {
    #[inline]
    fn from(s: Symbol) -> u64 { s.value }
}

impl FromStr for Symbol {
    type Err = SymbolError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = str_to_symbol(s)?;
        Ok(Symbol { value: v })
    }
}

impl TryFrom<(&str, u8)> for Symbol {
    type Error = SymbolError;
    fn try_from((name, prec): (&str, u8)) -> Result<Self, Self::Error> {
        if name.is_empty() || name.len() > 7 {
            return Err(SymbolError("invalid symbol name length".into()));
        }
        let mut v = 0u64;
        for &b in name.as_bytes().iter().rev() {
            if !b.is_ascii_uppercase() {
                return Err(SymbolError("invalid symbol name char".into()));
            }
            v = (v << 8) | b as u64;
        }
        v = (v << 8) | prec as u64;
        Ok(Symbol { value: v })
    }
}

impl Symbol {
    pub fn value(&self) -> u64 { self.value }
    pub fn code(&self) -> SymbolCode {
        SymbolCode { value: self.value >> 8 }
    }
    pub fn precision(&self) -> u8 { (self.value & 0xFF) as u8 }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.precision(), self.code())
    }
}

impl Packer for Symbol {
    fn size(&self) -> usize { 8 }
    fn pack(&self, enc: &mut Encoder) -> usize { self.value.pack(enc) }
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, data, 8);
        self.value.unpack(data)?;
        Ok(8)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum AssetStringParseStatus {
    Initial,
    FoundDot,
    FoundSpace,
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    amount: i64,
    symbol: Symbol,
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct AssetParseError(String);

#[derive(Debug, Error)]
#[error("{0}")]
pub struct AssetOpError(String);

impl TryFrom<(i64, Symbol)> for Asset {
    type Error = AssetParseError;
    fn try_from((amt, sym): (i64, Symbol)) -> Result<Self, Self::Error> {
        if !(-ASSET_MAX_AMOUNT..=ASSET_MAX_AMOUNT).contains(&amt) {
            return Err(AssetParseError("amount out of range".into()));
        }
        Ok(Asset { amount: amt, symbol: sym })
    }
}

impl Asset {
    pub fn amount(&self) -> i64 { self.amount }
    pub fn symbol(&self) -> Symbol { self.symbol }

    pub fn try_add(&self, other: Asset) -> Result<Asset, AssetOpError> {
        if self.symbol != other.symbol {
            return Err(AssetOpError("symbol mismatch".into()));
        }
        let sum = self.amount.checked_add(other.amount)
            .ok_or_else(|| AssetOpError("addition overflow".into()))?;
        Asset::try_from((sum, self.symbol))
            .map_err(|e| AssetOpError(e.to_string()))
    }

    pub fn try_sub(&self, other: Asset) -> Result<Asset, AssetOpError> {
        if self.symbol != other.symbol {
            return Err(AssetOpError("symbol mismatch".into()));
        }
        let diff = self.amount.checked_sub(other.amount)
            .ok_or_else(|| AssetOpError("subtraction overflow".into()))?;
        Asset::try_from((diff, self.symbol))
            .map_err(|e| AssetOpError(e.to_string()))
    }

    pub fn try_mul(&self, other: Asset) -> Result<Asset, AssetOpError> {
        if self.symbol != other.symbol {
            return Err(AssetOpError("symbol mismatch".into()));
        }
        let mul = self.amount.checked_mul(other.amount)
            .ok_or_else(|| AssetOpError("addition overflow".into()))?;
        Asset::try_from((mul, self.symbol))
            .map_err(|e| AssetOpError(e.to_string()))
    }

    pub fn try_div(&self, other: Asset) -> Result<Asset, AssetOpError> {
        if self.symbol != other.symbol {
            return Err(AssetOpError("symbol mismatch".into()));
        }
        let div = self.amount.checked_div(other.amount)
            .ok_or_else(|| AssetOpError("subtraction overflow".into()))?;
        Asset::try_from((div, self.symbol))
            .map_err(|e| AssetOpError(e.to_string()))
    }
}

impl FromStr for Asset {
    type Err = AssetParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(AssetParseError("empty string".into()));
        }
        let mut status = AssetStringParseStatus::Initial;
        let mut bytes = s.as_bytes();
        let mut negative = false;
        if bytes[0] == b'-' {
            negative = true;
            bytes = &bytes[1..];
        }
        let mut amt = 0i64;
        let mut prec = 0u8;
        let mut sym_bytes = Vec::new();
        for &c in bytes {
            // decimal separator only valid before space
            if status == AssetStringParseStatus::Initial && c == b'.' {
                status = AssetStringParseStatus::FoundDot;
                continue;
            }
            // always transition to symbol on the first space, whether before or after dot
            if (status == AssetStringParseStatus::Initial
                || status == AssetStringParseStatus::FoundDot)
                && c == b' '
            {
                status = AssetStringParseStatus::FoundSpace;
                continue;
            }

            match status {
                AssetStringParseStatus::Initial => {
                    if !c.is_ascii_digit() {
                        return Err(AssetParseError("invalid digit".into()));
                    }
                    amt = amt
                        .checked_mul(10)
                        .and_then(|a| a.checked_add((c - b'0') as i64))
                        .ok_or_else(|| AssetParseError("amount overflow".into()))?;
                }
                AssetStringParseStatus::FoundDot => {
                    if !c.is_ascii_digit() {
                        return Err(AssetParseError("invalid digit".into()));
                    }
                    prec = prec
                        .checked_add(1)
                        .ok_or_else(|| AssetParseError("precision overflow".into()))?;
                    amt = amt
                        .checked_mul(10)
                        .and_then(|a| a.checked_add((c - b'0') as i64))
                        .ok_or_else(|| AssetParseError("amount overflow".into()))?;
                    if prec > ASSET_MAX_PRECISION {
                        return Err(AssetParseError("precision too high".into()));
                    }
                }
                AssetStringParseStatus::FoundSpace => {
                    if !c.is_ascii_uppercase() {
                        return Err(AssetParseError("invalid symbol char".into()));
                    }
                    sym_bytes.push(c);
                }
            }
        }
        if sym_bytes.is_empty() {
            return Err(AssetParseError("empty symbol".into()));
        }
        if negative {
            amt = -amt;
        }
        let mut sym_val = 0u64;
        for &b in sym_bytes.iter().rev() {
            sym_val = (sym_val << 8) | b as u64;
        }
        sym_val = (sym_val << 8) | prec as u64;
        let sym = Symbol { value: sym_val };
        Asset::try_from((amt, sym))
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = self.symbol.precision() as usize;
        let abs = self.amount.unsigned_abs();
        let mut pow10 = 10u64.pow(p as u32);
        if pow10 == 0 {
            pow10 = 1;
        }
        let int = self.amount / pow10 as i64;
        let frac = abs % pow10;
        if p > 0 {
            write!(f, "{}.{:0width$} {}", int, frac, self.symbol.code(), width = p)
        } else {
            write!(f, "{} {}", int, self.symbol.code())
        }
    }
}

impl Packer for Asset {
    fn size(&self) -> usize { 16 }
    fn pack(&self, enc: &mut Encoder) -> usize {
        let start = enc.get_size();
        self.amount.pack(enc);
        self.symbol.pack(enc);
        enc.get_size() - start
    }
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, data, 16);
        let mut dec = Decoder::new(data);
        dec.unpack(&mut self.amount)?;
        dec.unpack(&mut self.symbol)?;
        Ok(dec.get_pos())
    }
}

pub(crate) fn deserialize_asset<'de, D>(deserializer: D) -> Result<Asset, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;
    impl de::Visitor<'_> for Visitor {
        type Value = Asset;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("asset string, e.g. \"1.23 EOS\"")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error {
            Asset::from_str(v).map_err(E::custom)
        }
    }
    deserializer.deserialize_str(Visitor)
}

pub(crate) fn deserialize_optional_asset<'de, D>(
    deserializer: D,
) -> Result<Option<Asset>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;
    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Option<Asset>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("optional asset string")
        }
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: de::Error {
            Ok(None)
        }
        fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
        where D: Deserializer<'de> {
            deserialize_asset(d).map(Some)
        }
    }
    deserializer.deserialize_option(Visitor)
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtendedAsset {
    pub quantity: Asset,
    pub contract: Name,
}

impl fmt::Display for ExtendedAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.quantity, self.contract)
    }
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ExtendedAssetError(String);

impl FromStr for ExtendedAsset {
    type Err = ExtendedAssetError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split('@').collect();
        if parts.len() != 2 {
            return Err(ExtendedAssetError("invalid format".into()));
        }
        let qty = Asset::from_str(parts[0])
            .map_err(|e| ExtendedAssetError(e.to_string()))?;
        let contract = Name::from_str(parts[1])
            .map_err(|e| ExtendedAssetError(e.to_string()))?;
        Ok(ExtendedAsset { quantity: qty, contract })
    }
}

impl Packer for ExtendedAsset {
    fn size(&self) -> usize { 16 + 8 }
    fn pack(&self, enc: &mut Encoder) -> usize {
        let start = enc.get_size();
        self.quantity.pack(enc);
        self.contract.pack(enc);
        enc.get_size() - start
    }
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, data, 24);
        let mut dec = Decoder::new(data);
        dec.unpack(&mut self.quantity)?;
        dec.unpack(&mut self.contract)?;
        Ok(dec.get_pos())
    }
}

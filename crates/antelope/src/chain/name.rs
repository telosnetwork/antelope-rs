use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};
use serde::{
    de::{self, SeqAccess, Visitor, Error as SerdeDeError},
    ser::Error as SerdeSerError,
    Deserialize, Deserializer, Serialize, Serializer,
};
use thiserror::Error;

use crate::{
    check_unpack_len,
    serializer::{Encoder, Packer, PackerError},
};

const INVALID_NAME_CHAR: u8 = 0xff;
const INVALID_NAME: u64 = 0xFFFF_FFFF_FFFF_FFFF;

pub const CHAR_MAP: [u8; 32] = [
    b'.', b'1', b'2', b'3', b'4', b'5', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i',
    b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x',
    b'y', b'z',
];

#[inline]
pub const fn char_to_index(c: u8) -> u8 {
    match c {
        b'a'..=b'z' => (c - b'a') + 6,
        b'1'..=b'5' => (c - b'1') + 1,
        b'.' => 0,
        _ => INVALID_NAME_CHAR,
    }
}

pub const fn str_to_name(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return 0;
    }
    if len > 13 {
        return INVALID_NAME;
    }
    let mut value = 0u64;
    let mut i = 0;
    let n = if len == 13 { 12 } else { len };
    while i < n {
        let idx = char_to_index(bytes[i]) as u64;
        if idx == INVALID_NAME_CHAR as u64 {
            return INVALID_NAME;
        }
        value = (value << 5) | idx;
        i += 1;
    }
    value <<= 4 + 5 * (12 - n);
    if len == 13 {
        let idx = char_to_index(bytes[12]) as u64;
        if idx == INVALID_NAME_CHAR as u64 || idx > 0x0f {
            return INVALID_NAME;
        }
        value |= idx;
    }
    value
}

#[derive(Debug, Error)]
pub enum NameError {
    #[error("invalid name")]
    InvalidName,
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub fn str_to_name_checked(s: &str) -> Result<u64, NameError> {
    let v = str_to_name(s);
    if v == INVALID_NAME {
        Err(NameError::InvalidName)
    } else {
        Ok(v)
    }
}


pub fn name_to_string(mut value: u64) -> Result<String, NameError> {
    // zero is the “empty” name
    if value == 0 {
        return Ok(String::new());
    }

    let mut buf = [b'.'; 13];
    for i in 0..13 {
        let index = if i == 0 {
            (value & 0x0f) as usize
        } else {
            (value & 0x1f) as usize
        };
        buf[12 - i] = CHAR_MAP[index];
        value >>= if i == 0 { 4 } else { 5 };
    }

    let last = buf.iter()
        .rposition(|&c| c != b'.')
        .ok_or(NameError::InvalidName)? + 1;

    String::from_utf8(buf[..last].to_vec()).map_err(NameError::Utf8)
}


#[repr(C, align(8))]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Name {
    n: u64,
}

impl Name {
    #[inline]
    pub fn value(&self) -> u64 {
        self.n
    }

    pub fn as_str(&self) -> Result<String, NameError> {
        name_to_string(self.n)
    }
}

impl FromStr for Name {
    type Err = NameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Name { n: str_to_name_checked(s)? })
    }
}

impl From<u64> for Name {
    #[inline]
    fn from(n: u64) -> Self {
        Name { n }
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{s}"),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl Packer for Name {
    fn size(&self) -> usize {
        8
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        self.n.pack(enc)
    }

    fn unpack(&mut self, raw: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, raw, 8);
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&raw[..8]);
        self.n = u64::from_le_bytes(buf);
        Ok(8)
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.n.cmp(&other.n))
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.n.cmp(&other.n)
    }
}

pub(crate) fn deserialize_name<'de, D>(deserializer: D) -> Result<Name, D::Error>
where
    D: Deserializer<'de>,
{
    struct VisitorImpl;

    impl Visitor<'_> for VisitorImpl {
        type Value = Name;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("EOSIO name string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Name, E>
        where
            E: de::Error,
        {
            Name::from_str(v).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(VisitorImpl)
}

pub(crate) fn deserialize_optional_name<'de, D>(deserializer: D) -> Result<Option<Name>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => Ok(Some(Name::from_str(&s).map_err(D::Error::custom)?)),
        None => Ok(None),
    }
}

pub(crate) fn deserialize_vec_name<'de, D>(deserializer: D) -> Result<Vec<Name>, D::Error>
where
    D: Deserializer<'de>,
{
    struct SeqVisitor;

    impl<'de> Visitor<'de> for SeqVisitor {
        type Value = Vec<Name>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("sequence of EOSIO name strings")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Vec<Name>, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut v = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                v.push(Name::from_str(&s).map_err(S::Error::custom)?);
            }
            Ok(v)
        }
    }

    deserializer.deserialize_seq(SeqVisitor)
}

pub(crate) fn serialize_name<S>(name: &Name, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&name.as_str().map_err(S::Error::custom)?)
}

#[allow(dead_code)]
pub(crate) fn serialize_optional_name<S>(
    opt: &Option<Name>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt {
        Some(name) => serializer.serialize_some(&name.as_str().map_err(S::Error::custom)?),
        None => serializer.serialize_none(),
    }
}

#[allow(dead_code)]
pub(crate) fn serialize_vec_name<S>(
    names: &[Name],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let strs: Vec<_> = names
        .iter()
        .map(|n| n.as_str().map_err(S::Error::custom))
        .collect::<Result<_, _>>()?;
    serializer.collect_seq(strs)
}

pub const SAME_PAYER: Name = Name { n: 0 };
pub const ACTIVE: Name = Name { n: str_to_name("active") };
pub const OWNER: Name = Name { n: str_to_name("owner") };
pub const CODE: Name = Name { n: str_to_name("eosio.code") };

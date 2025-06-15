use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, LowerHex},
    str::FromStr,
};
use serde::{Deserialize, Serialize};

use crate::{
    check_unpack_len,
    packer_error,
    serializer::{Encoder, Packer, PackerError},
};

#[derive(Clone, Copy, Debug, Default, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    #[default]
    K1,
    R1,
    WA,
}

impl FromStr for KeyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "K1" => Ok(KeyType::K1),
            "R1" => Ok(KeyType::R1),
            "WA" => Ok(KeyType::WA),
            _ => Err(format!("unknown key type '{s}'")),
        }
    }
}

impl TryFrom<u8> for KeyType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(KeyType::K1),
            1 => Ok(KeyType::R1),
            2 => Ok(KeyType::WA),
            i => Err(format!("unknown KeyType index {i}")),
        }
    }
}

impl KeyType {
    pub fn to_index(self) -> u8 {
        match self {
            KeyType::K1 => 0,
            KeyType::R1 => 1,
            KeyType::WA => 2,
        }
    }
}

impl Display for KeyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            KeyType::K1 => "K1",
            KeyType::R1 => "R1",
            KeyType::WA => "WA",
        })
    }
}

impl LowerHex for KeyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.to_index())
    }
}

impl Packer for KeyType {
    fn size(&self) -> usize {
        1
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        let buf = enc.alloc(self.size());
        buf[0] = self.to_index();
        self.size()
    }

    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, data, 1);
        *self = KeyType::try_from(data[0])
            .map_err(|e| packer_error!("KeyType index error: {e}"))?;
        Ok(1)
    }
}

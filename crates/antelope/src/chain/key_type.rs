use std::fmt::{Display, Formatter};
use crate::chain::{Encoder, Packer};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum KeyType {
    K1,
    R1,
    // ... other variants ...
}

pub trait KeyTypeTrait {
    fn from_string(s: &str) -> Result<KeyType, String>;
    fn from_index(i: u8) -> Result<KeyType, String>;
    fn to_index(&self) -> u8;
}

impl KeyTypeTrait for KeyType {
    fn from_string(s: &str) -> Result<KeyType, String> {
        if s == "K1" {
            return Ok(KeyType::K1);
        }

        if s == "R1" {
            return Ok(KeyType::R1);
        }

        return Err(format!("Unknown key type {s}"));
    }

    fn from_index(i: u8) -> Result<KeyType, String> {
        if i == 0 {
            return Ok(KeyType::K1);
        }

        if i == 1 {
            return Ok(KeyType::R1);
        }
        return Err(String::from(format!("Unknown KeyType index {i}")));
    }

    fn to_index(&self) -> u8 {
        match self {
            KeyType::K1 => { 0 }
            KeyType::R1 => { 1 }
        }
    }
}

impl Display for KeyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::K1 => { write!(f, "K1") }
            KeyType::R1 => { write!(f, "R1") }
        }
    }
}


impl Default for KeyType {
    fn default() -> Self { KeyType::K1 }
}

impl Packer for KeyType {
    fn size(&self) -> usize {
        1usize
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        let data = enc.alloc(self.size());
        match self {
            KeyType::K1 => { data[0] = 0u8 }
            KeyType::R1 => { data[0] = 1u8 }
        }
        self.size()
    }

    fn unpack(&mut self, data: &[u8]) -> usize {
        assert!(data.len() >= self.size(), "KeyType::unpack: buffer overflow");
        *self = KeyType::from_index(data[0]).unwrap();
        self.size()
    }
}


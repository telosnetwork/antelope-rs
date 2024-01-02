use ripemd::{Digest as Ripemd160Digest, Ripemd160};
use sha2::{Sha256, Sha512};
use crate::chain::{Encoder, Packer};
use crate::util::{bytes_to_hex, hex_to_bytes, slice_copy};

#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub struct Checksum160 {
    pub data: [u8; 20],
}

impl Checksum160 {
    pub fn from_hex(s: &str) -> Result<Self, String> {
        if s.len() != 40 { return Err(String::from("Checksum160: bad hex string length")) }
        let data = hex_to_bytes(s);
        Self::from_bytes(data.as_slice())
    }

    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if b.len() != 20 { return Err(String::from("Checksum160: bad byte array length")) }
        let mut ret = Self::default();
        slice_copy(&mut ret.data, b);
        Ok(ret)
    }

    pub fn hash(bytes: Vec<u8>) -> Self {
        let mut hasher = Ripemd160::new();
        hasher.update(bytes);
        let ripe_hash = hasher.finalize();
        Checksum160::from_bytes(ripe_hash.as_slice()).unwrap()
    }

    pub fn to_string(&self) -> String {
        bytes_to_hex(&self.data.to_vec())
    }
}

impl Packer for Checksum160 {
    fn size(&self) -> usize {
        return 20;
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        let data = enc.alloc(self.size());
        slice_copy(data, &self.data);
        self.size()
    }

    fn unpack(&mut self, raw: &[u8]) -> usize {
        let size = self.size();
        assert!(raw.len() >= size, "Checksum160.unpack: buffer overflow!");
        slice_copy(&mut self.data, &raw[..size]);
        return size;
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub struct Checksum256 {
    pub data: [u8; 32],
}

impl Checksum256 {
    pub fn from_hex(s: &str) -> Result<Self, String> {
        if s.len() != 64 { return Err(String::from("Checksum256: bad hex string length")) }
        let data = hex_to_bytes(s);
        Self::from_bytes(data.as_slice())
    }

    pub fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if b.len() != 32 { return Err(String::from("Checksum256: bad byte array length")) }
        let mut ret = Self::default();
        slice_copy(&mut ret.data, b);
        Ok(ret)
    }

    pub fn hash(bytes: Vec<u8>) -> Self {
        return Checksum256::from_bytes(Sha256::digest(bytes).as_slice()).unwrap();
    }

    pub fn to_string(&self) -> String {
        bytes_to_hex(&self.data.to_vec())
    }
}

impl Packer for Checksum256 {
    fn size(&self) -> usize {
        return 32;
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        let data = enc.alloc(self.size());
        slice_copy(data, &self.data);
        self.size()
    }

    fn unpack(&mut self, raw: &[u8]) -> usize {
        let size = self.size();
        assert!(raw.len() >= size, "Checksum256.unpack: buffer overflow!");
        slice_copy(&mut self.data, &raw[..size]);
        return self.size();
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Checksum512 {
    pub data: [u8; 64],
}

impl Checksum512 {
    pub fn from_hex(s: &str) -> Result<Self, String> {
        if s.len() != 128 { return Err(String::from("Checksum512: bad hex string length")) }
        let data = hex_to_bytes(s);
        Ok(Self::from_bytes(data.as_slice()))
    }

    pub fn from_bytes(b: &[u8]) -> Self {
        assert!(b.len() == 64, "Checksum512: bad byte array length");
        let mut ret = Self::default();
        slice_copy(&mut ret.data, b);
        ret
    }

    pub fn hash(bytes: Vec<u8>) -> Self {
        return Checksum512::from_bytes(Sha512::digest(bytes).as_slice());
    }

    pub fn to_string(&self) -> String {
        bytes_to_hex(&self.data.to_vec())
    }
}

impl Default for Checksum512 {
    fn default() -> Self {
        Checksum512 {data: [0; 64]}
    }
}

impl Packer for Checksum512 {
    fn size(&self) -> usize {
        return 64;
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        let data = enc.alloc(self.size());
        slice_copy(data, &self.data);
        self.size()
    }

    fn unpack(&mut self, raw: &[u8]) -> usize {
        let size = self.size();
        assert!(raw.len() >= size, "Checksum512.unpack: buffer overflow!");
        slice_copy(&mut self.data, &raw[..size]);
        return size;
    }
}
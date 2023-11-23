use std::collections::HashMap;
use crate::serializer::Serializer;
use crate::chain::ABISerializableObject;

pub struct EncodeArgsSerializable {
    pub object: Box<dyn ABISerializableObject>
}

pub enum EncodeArgs {
    EncodeArgsSerializable(EncodeArgsSerializable)
}

impl Serializer {
    pub fn encode(args: EncodeArgs) -> Vec<u8> {
        let mut encoder = ABIEncoder::new();
        match args {
            EncodeArgs::EncodeArgsSerializable(serializable) => {
                serializable.object.to_abi(&mut encoder);
            }
        }
        return encoder.get_bytes();
    }
}

pub struct ABIEncoder {
    pos: usize,
    data: Vec<u8>,
    page_size: usize,
    metadata: HashMap<String, String>,
}

impl ABIEncoder {
    pub fn new() -> Self {
        let page_size = 1024;

        ABIEncoder {
            pos: 0,
            data: vec![0; page_size],
            page_size,
            metadata: HashMap::new(),
        }
    }

    fn ensure(&mut self, bytes: usize) {
        let required = self.pos + bytes;
        if self.data.len() < required {
            let pages = (bytes + self.page_size - 1) / self.page_size;
            self.data.resize(self.data.len() + pages * self.page_size, 0);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.ensure(1);
        self.data[self.pos] = byte;
        self.pos += 1;
    }

    pub fn write_array(&mut self, bytes: Vec<u8>) {
        let size = bytes.len();
        self.ensure(size);
        self.data[self.pos..self.pos + size].copy_from_slice(bytes.as_slice());
        self.pos += size;
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut trimmed = self.data.clone();
        trimmed.truncate(self.pos);
        return trimmed;
    }

}
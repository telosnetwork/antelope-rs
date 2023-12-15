use crate::chain::{Encoder, Packer};

#[derive(Copy, Clone, Default, PartialEq)]
pub struct TimePoint {
    /// elapsed in microseconds
    pub elapsed: u64,
}

impl Packer for TimePoint {
    fn size(&self) -> usize {
        return 8;
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        self.elapsed.pack(enc)
    }

    fn unpack(&mut self, raw: &[u8]) -> usize {
        assert!(raw.len() >= self.size(), "TimePoint.unpack: buffer overflow!");
        return self.elapsed.unpack(raw);
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct TimePointSec {
    ///
    pub seconds: u32,
}

impl TimePointSec {
    pub fn new(seconds: u32) -> Self{
        Self{ seconds }
    }

    pub fn seconds(&self) -> u32 {
        return self.seconds;
    }
}

impl Packer for TimePointSec {
    fn size(&self) -> usize {
        return 4;
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        self.seconds.pack(enc)
    }

    fn unpack(&mut self, raw: &[u8]) -> usize {
        assert!(raw.len() >= self.size(), "TimePointSec.unpack: buffer overflow!");
        return self.seconds.unpack(raw);
    }
}
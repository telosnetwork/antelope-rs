use crate::chain::{Encoder, Packer};
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TimePoint {
    /// elapsed in microseconds
    pub elapsed: u64,
}

impl TimePoint {
    pub fn from_timestamp(t: &str) -> Result<Self, String> {
        //2023-12-16T16:17:47.500
        let naive_date_time = NaiveDateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S%.f");

        if naive_date_time.is_err() {
            return Err(String::from("Failed to parse datetime ")
                + naive_date_time.err().unwrap().to_string().as_str());
        }
        let date_time = Utc.from_utc_datetime(&naive_date_time.unwrap());

        Ok(Self {
            elapsed: (date_time.timestamp_millis() * 1000) as u64,
        })
    }
}

impl Packer for TimePoint {
    fn size(&self) -> usize {
        8
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        self.elapsed.pack(enc)
    }

    fn unpack(&mut self, raw: &[u8]) -> usize {
        assert!(
            raw.len() >= self.size(),
            "TimePoint.unpack: buffer overflow!"
        );
        self.elapsed.unpack(raw)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct TimePointSec {
    ///
    pub seconds: u32,
}

impl TimePointSec {
    pub fn new(seconds: u32) -> Self {
        Self { seconds }
    }

    pub fn seconds(&self) -> u32 {
        self.seconds
    }
}

impl Packer for TimePointSec {
    fn size(&self) -> usize {
        4
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        self.seconds.pack(enc)
    }

    fn unpack(&mut self, raw: &[u8]) -> usize {
        assert!(
            raw.len() >= self.size(),
            "TimePointSec.unpack: buffer overflow!"
        );
        self.seconds.unpack(raw)
    }
}

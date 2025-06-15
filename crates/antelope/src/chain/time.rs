use chrono::{NaiveDateTime, TimeZone, Utc, LocalResult};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

use crate::check_unpack_len;
use crate::serializer::{Encoder, Packer, PackerError};

const BLOCK_INTERVAL_MS: u64 = 500;
const BLOCK_TIMESTAMP_EPOCH_MS: u64 = 946_684_800_000; // 2000-01-01T00:00:00Z

#[derive(Debug, Error)]
pub enum TimeError {
    #[error("failed to parse datetime: {0}")]
    Parse(#[from] chrono::ParseError),
    #[error("datetime out of valid range")]
    OutOfRange,
}

fn utc_from_naive(ndt: NaiveDateTime) -> i64 {
    Utc.from_utc_datetime(&ndt).timestamp_micros()
}

#[derive(Copy, Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct TimePoint {
    /// microseconds since Unix epoch
    pub elapsed: u64,
}

impl From<u64> for TimePoint {
    #[inline]
    fn from(elapsed: u64) -> Self {
        TimePoint { elapsed }
    }
}

impl FromStr for TimePoint {
    type Err = TimeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_timestamp(s)
    }
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let millis = (self.elapsed / 1_000) as i64;
        let sub_ms = self.elapsed % 1_000;
        match Utc.timestamp_millis_opt(millis) {
            LocalResult::Single(dt) => {
                write!(f, "{}.{:03}", dt.format("%Y-%m-%dT%H:%M:%S"), sub_ms)
            }
            _ => Err(fmt::Error),
        }
    }
}

impl TimePoint {
    pub fn from_timestamp(t: &str) -> Result<Self, TimeError> {
        let ndt = NaiveDateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S%.f")?;
        Ok(Self { elapsed: utc_from_naive(ndt) as u64 })
    }
}

impl Packer for TimePoint {
    fn size(&self) -> usize { 8 }
    fn pack(&self, enc: &mut Encoder) -> usize { self.elapsed.pack(enc) }
    fn unpack(&mut self, raw: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, raw, 8);
        self.elapsed.unpack(raw)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Debug)]
pub struct TimePointSec {
    pub seconds: u32,
}

impl From<u32> for TimePointSec {
    #[inline]
    fn from(seconds: u32) -> Self {
        TimePointSec { seconds }
    }
}

impl FromStr for TimePointSec {
    type Err = TimeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::from_timestamp(s) }
}

impl fmt::Display for TimePointSec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match Utc.timestamp_opt(self.seconds as i64, 0) {
            LocalResult::Single(dt) => write!(f, "{}", dt.format("%Y-%m-%dT%H:%M:%S")),
            _ => Err(fmt::Error),
        }
    }
}

impl TimePointSec {
    pub fn new(seconds: u32) -> Self { Self { seconds } }

    pub fn from_timestamp(t: &str) -> Result<Self, TimeError> {
        let ndt = NaiveDateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S")?;
        let secs = ndt.and_utc().timestamp();
        if secs < 0 { return Err(TimeError::OutOfRange); }
        Ok(Self { seconds: secs as u32 })
    }
}

impl Packer for TimePointSec {
    fn size(&self) -> usize { 4 }
    fn pack(&self, enc: &mut Encoder) -> usize { self.seconds.pack(enc) }
    fn unpack(&mut self, raw: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, raw, 4);
        self.seconds.unpack(raw)
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BlockTimestamp {
    pub slot: u32,
}

impl From<u32> for BlockTimestamp {
    #[inline]
    fn from(slot: u32) -> Self {
        BlockTimestamp { slot }
    }
}

impl FromStr for BlockTimestamp {
    type Err = TimeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_time_point_sec(TimePointSec::from_str(s)?))
    }
}

impl fmt::Display for BlockTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_time_point_sec())
    }
}

impl BlockTimestamp {
    pub fn new(slot: u32) -> Self { Self { slot } }
    pub fn maximum() -> Self { Self { slot: 0xFFFF } }
    pub fn min() -> Self { Self { slot: 0 } }
    pub fn next(&self) -> Self { Self { slot: self.slot.saturating_add(1) } }

    pub fn from_time_point(tp: TimePoint) -> Self {
        let ms = tp.elapsed / 1_000;
        let slot = ((ms - BLOCK_TIMESTAMP_EPOCH_MS) / BLOCK_INTERVAL_MS) as u32;
        Self { slot }
    }

    pub fn from_time_point_sec(tp: TimePointSec) -> Self {
        let ms = tp.seconds as u64 * 1_000;
        let slot = ((ms - BLOCK_TIMESTAMP_EPOCH_MS) / BLOCK_INTERVAL_MS) as u32;
        Self { slot }
    }

    pub fn to_time_point(&self) -> TimePoint {
        let ms = self.slot as u64 * BLOCK_INTERVAL_MS + BLOCK_TIMESTAMP_EPOCH_MS;
        TimePoint { elapsed: ms * 1_000 }
    }

    pub fn to_time_point_sec(&self) -> TimePointSec {
        let seconds = (self.slot as u64 * BLOCK_INTERVAL_MS + BLOCK_TIMESTAMP_EPOCH_MS) / 1_000;
        TimePointSec { seconds: seconds as u32 }
    }
}

impl Packer for BlockTimestamp {
    fn size(&self) -> usize { 4 }
    fn pack(&self, enc: &mut Encoder) -> usize { self.slot.pack(enc) }
    fn unpack(&mut self, raw: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, raw, 4);
        self.slot.unpack(raw)
    }
}

pub(crate) fn deserialize_timepoint<'de, D>(deserializer: D) -> Result<TimePoint, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;
    impl de::Visitor<'_> for Visitor {
        type Value = TimePoint;
        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("ISO-8601 datetime string")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            TimePoint::from_str(v).map_err(de::Error::custom)
        }
    }
    deserializer.deserialize_str(Visitor)
}

pub(crate) fn deserialize_optional_timepoint<'de, D>(
    deserializer: D,
) -> Result<Option<TimePoint>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;
    impl de::Visitor<'_> for Visitor {
        type Value = Option<TimePoint>;
        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("null or ISO-8601 datetime string")
        }
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        { Ok(None) }
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        { Ok(None) }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        { TimePoint::from_str(v).map(Some).map_err(de::Error::custom) }
    }
    deserializer.deserialize_any(Visitor)
}

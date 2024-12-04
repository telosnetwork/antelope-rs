use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::chain::{Encoder, Packer};

#[derive(Copy, Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct TimePoint {
    /// elapsed in microseconds
    pub elapsed: u64,
}

impl FromStr for TimePoint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TimePoint::from_timestamp(s)
    }
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

pub(crate) fn deserialize_timepoint<'de, D>(deserializer: D) -> Result<TimePoint, D::Error>
where
    D: Deserializer<'de>,
{
    struct TimePointVisitor;

    impl de::Visitor<'_> for TimePointVisitor {
        type Value = TimePoint;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string representing a datetime")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            TimePoint::from_timestamp(value).map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_str(TimePointVisitor)
}

pub(crate) fn deserialize_optional_timepoint<'de, D>(
    deserializer: D,
) -> Result<Option<TimePoint>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionalTimePointVisitor;

    impl de::Visitor<'_> for OptionalTimePointVisitor {
        type Value = Option<TimePoint>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an optional string representing a datetime or null")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            TimePoint::from_timestamp(value)
                .map(Some)
                .map_err(de::Error::custom)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    // Updated to handle null values directly
    deserializer.deserialize_any(OptionalTimePointVisitor)
}

use crate::types::{AntelopeType, AntelopeValue};
use crate::types::uint64::UInt64;

pub struct Name {
    pub uint64: UInt64
}

impl Name {
    pub fn from_str(name: &str) -> Self {
        let value = UInt64 {
            value: Name::string_to_u64(name).unwrap()
        };

        return Name {
            uint64: value
        }
    }

    fn string_to_u64(s: &str) -> Result<u64, String> {
        if s.len() > 13 {
            return Err("invalid string length".to_string());
        }

        let mut name: u64 = 0;
        for (i, ch) in s.chars().enumerate().take(12) {
            name |= (Name::char_to_symbol(ch as u32) & 0x1f) << (64 - 5 * (i + 1));
        }

        if s.len() == 13 {
            let ch = s.chars().nth(12).unwrap() as u32;
            name |= Name::char_to_symbol(ch) & 0x0f;
        }

        Ok(name)
    }

    fn char_to_symbol(c: u32) -> u64 {
        match c {
            c if (('a' as u32)..=('z' as u32)).contains(&c) => (c - 'a' as u32) as u64 + 6,
            c if (('1' as u32)..=('5' as u32)).contains(&c) => (c - '1' as u32) as u64 + 1,
            _ => 0,
        }
    }

    fn u64_to_string(mut n: u64, strip_dots: bool) -> String {
        let charmap = ".12345abcdefghijklmnopqrstuvwxyz".as_bytes();
        let mut s = vec![b'.'; 13];

        for i in 0..13 {
            let mask = if i == 0 { 0x0f } else { 0x1f };
            let c = charmap[(n & mask) as usize];
            s[12 - i] = c;
            n >>= if i == 0 { 4 } else { 5 };
        }

        let mut result = String::from_utf8(s).expect("Invalid UTF-8");
        if strip_dots {
            result = result.trim_matches('.').to_string();
        }
        return result;
    }
}

impl AntelopeType for Name {
    fn deserialize(&self) -> AntelopeValue {
        return AntelopeValue::String(Name::u64_to_string(self.uint64.value, true));
    }

    fn serialize(&self) -> Vec<u8> {
        return self.uint64.serialize();
    }
}
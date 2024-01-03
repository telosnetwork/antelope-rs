use std::fmt::{Display, Formatter};
use crate::serializer::{
    Packer,
    Encoder,
};

const INVALID_NAME_CHAR: u8 = 0xffu8;

/// a helper function that converts a single ASCII character to
/// a symbol used by the eosio::name object.
/// ".12345abcdefghijklmnopqrstuvwxyz"
pub const fn char_to_index(c: u8) -> u8 {
    match c as char {
        'a'..='z' => {
            (c - b'a') + 6
        }
        '1'..='5' => {
            (c - b'1') + 1
        }
        '.' => {
            0
        }
        _ => {
            INVALID_NAME_CHAR
        }
    }
}

const INVALID_NAME: u64 = 0xFFFF_FFFF_FFFF_FFFFu64;


// converts a static string to an `name` object.
pub const fn static_str_to_name(s: &'static str) -> u64 {
    let mut value: u64 = 0;
    let _s = s.as_bytes();

    if _s.len() > 13 {
        return INVALID_NAME;
    }

    if _s.is_empty() {
        return 0;
    }

    let mut n = _s.len();
    if n == 13 {
        n = 12;
    }

    let mut i = 0usize;

    loop {
        if i >= n {
            break;
        }
        let tmp = char_to_index(_s[i]) as u64;
        if tmp == INVALID_NAME_CHAR as u64 {
            return INVALID_NAME;
        }
        value <<= 5;
        value |= tmp;

        i += 1;
    }
    value <<=  4 + 5*(12 - n);

    if _s.len() == 13 {
        let tmp = char_to_index(_s[12]) as u64;
        if tmp == INVALID_NAME_CHAR as u64 {
            return INVALID_NAME;
        }
        if tmp > 0x0f {
            return INVALID_NAME;
        }
        value |= tmp;
    }

    value
}


/// similar to static_str_to_name,
/// but also checks the validity of the resulting `name` object.
pub fn static_str_to_name_checked(s: &'static str) -> u64 {
    let n = static_str_to_name(s);
    assert_ne!(n, INVALID_NAME, "bad name");
    n
}


// a shorthand for static_str_to_name_checked.
pub fn s2n(s: &'static str) -> u64 {
    static_str_to_name_checked(s)
}

// ".12345abcdefghijklmnopqrstuvwxyz"
pub const CHAR_MAP: [u8; 32] = [46,49,50,51,52,53,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122];

/// converts an `name` object to a string.
pub fn n2s(value: u64) -> String {
    // 13 dots
    let mut s: [u8; 13] = [46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46, 46]; //'.'
    let mut tmp = value;
    for i in 0..13 {
        let c: u8 = if i == 0 {
            CHAR_MAP[(tmp&0x0f) as usize]
        } else {
            CHAR_MAP[(tmp&0x1f) as usize]
        };
        s[12-i] = c;
        if i == 0 {
            tmp >>= 4
        } else {
            tmp >>= 5
        }
    }

    let mut i = s.len() - 1;
    while i != 0 {
        if s[i] != b'.' {
            break
        }
        i -= 1;
    }
    String::from_utf8(s[0..i+1].to_vec()).unwrap()
}


///
fn str_to_name(s: &str) -> u64 {
    let mut value: u64 = 0;
    let _s = s.as_bytes();

    if _s.len() > 13 {
        return INVALID_NAME;
    }

    if _s.is_empty() {
        return 0;
    }

    let mut n = _s.len();
    if n == 13 {
        n = 12;
    }

    let mut i = 0usize;

    loop {
        if i >= n {
            break;
        }
        let tmp = char_to_index(_s[i]) as u64;
        if tmp == 0xff {
            return INVALID_NAME;
        }
        value <<= 5;
        value |= tmp;

        i += 1;
    }
    value <<=  4 + 5*(12 - n);

    if _s.len() == 13 {
        let tmp = char_to_index(_s[12]) as u64;
        if tmp == 0xff {
            return INVALID_NAME;
        }
        if tmp > 0x0f {
            return INVALID_NAME;
        }
        value |= tmp;
    }

    value
}

fn str_to_name_checked(s: &str) -> u64 {
    let n = str_to_name(s);
    assert_ne!(n, INVALID_NAME, "bad name string");
    n
}

/// a wrapper around a 64-bit unsigned integer that represents a name in the Antelope blockchain
#[repr(C, align(8))]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Name {
    pub n: u64,
}

impl Name {
    pub fn new(s: &'static str) -> Self {
        Name { n: s2n(s) }
    }

    pub fn value(&self) -> u64 {
        self.n
    }

    pub fn from_u64(n: u64) -> Self {
        assert_ne!(n, INVALID_NAME, "bad name value");
        Name { n }
    }

    pub fn new_from_str(s: &str) -> Self {
        Name{ n: str_to_name_checked(s) }
    }

    pub fn as_string(&self) -> String {
        n2s(self.n)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl Packer for Name {
    fn size(&self) -> usize {
        8
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        self.n.pack(enc)
    }

    fn unpack(&mut self, raw: &[u8]) -> usize {
        assert!(raw.len() >= 8, "Name.unpack: buffer overflow!");
        self.n = u64::from_ne_bytes(raw[0..8].try_into().unwrap());
        8
    }
}

pub const SAME_PAYER: Name = Name{n: 0};
pub const ACTIVE: Name = Name{n: static_str_to_name("active")};
pub const OWNER: Name = Name{n: static_str_to_name("owner")};
pub const CODE: Name = Name{n: static_str_to_name("eosio.code")};
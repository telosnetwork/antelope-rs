pub mod abi;
pub mod action;
pub mod asset;
pub mod authority;
pub mod binary_extension;
pub mod blob;
pub mod checksum;
pub mod key_type;
pub mod name;
pub mod private_key;
pub mod public_key;
pub mod signature;
pub mod time;
pub mod transaction;
pub mod varint;
pub mod float;


#[macro_export]
macro_rules! name {
    ($str:expr) => {{
        <Name as ::std::str::FromStr>::from_str($str)
            .expect(&format!("Invalid name: \"{}\"", $str))
    }};
}


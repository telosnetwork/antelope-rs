pub use crate::serializer::{ Decoder, Encoder, Packer };

pub mod action;
pub mod asset;
pub mod blob;
pub mod block_id;
pub mod checksum;
pub mod key_type;
pub mod name;
pub mod private_key;
pub mod public_key;
pub mod signature;
pub mod transaction;
pub mod varint;
pub mod time;

#[macro_export]
macro_rules! name {
    ($str:expr) => {
        Name::new_from_str($str)
    };
}
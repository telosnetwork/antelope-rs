use hex::{decode, encode};
use crate::chain::ABISerializableObject;
use crate::serializer::encoder::{EncodeArgs, EncodeArgsSerializable};

pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    return decode(hex).unwrap();
}

pub fn bytes_to_hex(bytes: &Vec<u8>) -> String {
    return encode(bytes);
}

pub fn serializable_to_encode_args(object: Box<dyn ABISerializableObject>) -> EncodeArgs {
    return EncodeArgs::EncodeArgsSerializable(EncodeArgsSerializable { object });
}
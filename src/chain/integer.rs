use crate::chain::{ABISerializableObject, JSONValue};
use crate::serializer::encoder::ABIEncoder;

pub struct IntType<T> {
    pub value: T,
}

pub type Int8 = IntType<i8>;
pub type Int16 = IntType<i16>;
pub type Int32 = IntType<i32>;
pub type Int64 = IntType<i64>;
pub type Int128 = IntType<i128>;
pub type UInt8 = IntType<u8>;
pub type UInt16 = IntType<u16>;
pub type UInt32 = IntType<u32>;
pub type UInt64 = IntType<u64>;
pub type UInt128 = IntType<u128>;

impl<T> IntType<T> {
    pub fn new(value: T) -> Self {
        IntType { value }
    }
}

impl<T> From<T> for IntType<T> {
    fn from(value: T) -> Self {
        IntType::new(value)
    }
}

macro_rules! impl_int_type {
    ($type:ty, $abi_name:expr) => {
        impl ABISerializableObject for IntType<$type> {
            fn get_abi_name(&self) -> String {
                $abi_name.to_string()
            }

            fn to_abi(&self, encoder: &mut ABIEncoder) {
                encoder.write_array(&self.value.to_le_bytes().to_vec());
            }

            fn to_json(&self) -> JSONValue {
                JSONValue::String(self.value.to_string())
            }
        }
    };
}

impl_int_type!(i8, "int8");
impl_int_type!(i16, "int16");
impl_int_type!(i32, "int32");
impl_int_type!(i64, "int64");
impl_int_type!(i128, "int128");
impl_int_type!(u8, "uint8");
impl_int_type!(u16, "uint16");
impl_int_type!(u32, "uint32");
impl_int_type!(u64, "uint64");
impl_int_type!(u128, "uint128");
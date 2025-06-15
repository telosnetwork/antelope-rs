use core::mem::size_of;
use crate::{chain::varint::VarUint32, define_error, util::slice_copy};

define_error!(PackerError);

#[macro_export]
macro_rules! packer_error {
    ($($arg:tt)*) => {
        $crate::serializer::packer::PackerError::fmt(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! check_unpack_len {
    // With literal expected size
    ($self:ident, $data:expr, $size:expr) => {{
        let my_size = $size;
        let delta = $data.len() as isize - my_size as isize;
        if delta < 0 {
            return Err($crate::serializer::packer::PackerError::fmt(format_args!(
                "buffer overflow by {} bytes while unpacking {}",
                delta,
                std::any::type_name::<Self>()
            )));
        }
        my_size
    }};
    // Default: use self.data.size()
    ($self:ident, $data:expr) => {{
        let my_size = $self.data.size();
        let delta = $data.len() as isize - my_size as isize;
        if delta < 0 {
            return Err($crate::serializer::packer::PackerError::fmt(format_args!(
                "buffer overflow by {} bytes while unpacking {}",
                delta,
                std::any::type_name::<Self>()
            )));
        }
        my_size
    }};
}

///
/// The `Packer` trait provides methods for packing and unpacking values to and
/// from byte arrays.
///
/// # Examples
///
/// ```
/// use crate::antelope::serializer::{Encoder, Decoder, Packer};
///
/// let mut encoder = Encoder::new(4);
/// let value = 123u32;
/// value.pack(&mut encoder);
///
/// let mut decoder = Decoder::new(&encoder.get_bytes());
/// let mut unpacked_value = 0u32;
/// decoder.unpack(&mut unpacked_value).unwrap();
///
/// assert_eq!(value, unpacked_value);
/// ```
pub trait Packer {
    /// Returns the size of the packed representation of this value in bytes.
    fn size(&self) -> usize;

    /// Packs this value into the given `Encoder`.
    ///
    /// # Arguments
    ///
    /// * `enc` - The encoder to pack this value into.
    ///
    /// # Returns
    ///
    /// The number of bytes written to the encoder.
    fn pack(&self, enc: &mut Encoder) -> usize;

    /// Unpacks this value from the given byte array.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte array to unpack this value from.
    ///
    /// # Returns
    ///
    /// The number of bytes read from the byte array.
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError>;
}

/// The `Encoder` struct provides methods for packing values that implement the
/// `Packer` trait.
///
/// # Examples
///
/// ```
/// use antelope::serializer::{Encoder, Packer};
///
/// let mut encoder = Encoder::new(4);
/// let value = 123u32;
///
/// let bytes_written = value.pack(&mut encoder);
/// assert_eq!(bytes_written, 4);
///
/// let packed_bytes = encoder.get_bytes();
/// assert_eq!(packed_bytes, [123, 0, 0, 0]);
/// ```
pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    /// Constructs a new `Encoder` with the given initial capacity.
    ///
    /// # Arguments
    ///
    /// * `size` - The initial capacity of the encoder in bytes.
    ///
    /// # Returns
    ///
    /// A new `Encoder` instance with the given initial capacity.
    pub fn new(size: usize) -> Self {
        Self {
            buf: Vec::with_capacity(size),
        }
    }

    /// Returns the packed bytes of this encoder as a byte array.
    ///
    /// # Returns
    ///
    /// A reference to the packed bytes of this encoder as a byte array.
    pub fn get_bytes(&self) -> &[u8] {
        &self.buf
    }

    /// Returns the number of packed bytes in this encoder.
    ///
    /// # Returns
    ///
    /// The number of packed bytes in this encoder.
    pub fn get_size(&self) -> usize {
        self.buf.len()
    }

    /// Allocates space in this encoder for packing a value of the given size.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of bytes to allocate in this encoder.
    ///
    /// # Returns
    ///
    /// A mutable reference to the allocated
    pub fn alloc(&mut self, size: usize) -> &mut [u8] {
        let old_size = self.buf.len();
        self.buf.resize(old_size + size, 0u8);
        &mut self.buf[old_size..]
    }

    /// Pack a raw slice directly
    ///
    /// # Arguments
    ///
    /// * `bytes` - The slice to append at the end of the current encoder payload.
    pub fn pack_raw(&mut self, bytes: &[u8]) {
        let target = self.alloc(bytes.len());
        target.copy_from_slice(bytes);
    }

    /// Packs the given value using the encoder
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be packed
    ///
    /// # Examples
    ///
    /// ```
    /// use antelope::serializer::{Encoder, Packer};
    ///
    /// let data = Encoder::pack(&1234u32);
    /// assert_eq!(data, vec![210, 4, 0, 0]);
    /// ```
    pub fn pack<T: Packer>(value: &T) -> Vec<u8> {
        // Create a new Encoder with the size of the value being packed
        let mut enc = Self::new(value.size());
        // Pack the value using the encoder
        value.pack(&mut enc);
        // Return the packed data as a vector of bytes
        enc.get_bytes().to_vec()
    }
}

/// A struct for unpacking packed data
///
/// # Examples
///
/// ```
/// use crate::antelope::serializer::{Decoder, Packer};
///
/// let data = &vec![210, 4, 0, 0];
/// let mut decoder = Decoder::new(&data);
/// let mut value = 0u32;
/// decoder.unpack(&mut value);
/// assert_eq!(value, 1234);
/// ```
pub struct Decoder<'a> {
    buf: &'a [u8],
    pos: usize,
}

/// A struct for unpacking packed data
impl<'a> Decoder<'a> {
    /// Creates a new `Decoder` instance from the given byte array.
    pub fn new(data: &'a [u8]) -> Self {
        Self { buf: data, pos: 0 }
    }

    /// Unpacks the given value from the decoder
    pub fn unpack<T>(&mut self, packer: &mut T) -> Result<usize, PackerError>
    where
        T: Packer,
    {
        let size = packer.unpack(&self.buf[self.pos..])?;
        self.pos += size;
        Ok(size)
    }

    /// Returns the current position of the decoder
    pub fn get_pos(&self) -> usize {
        self.pos
    }

    /// Returns the total size of the buffer to decode
    pub fn size(&self) -> usize {
        self.buf.len()
    }

    /// Return the amount of remaining bytes to decode
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }
}

/// A trait for packing and unpacking values
macro_rules! impl_packed {
    ( $ty:ident ) => {
        impl Packer for $ty {
            /// Returns the size of this value in bytes.
            fn size(&self) -> usize {
                size_of::<$ty>()
            }

            /// Packs this value into the given encoder.
            fn pack(&self, enc: &mut Encoder) -> usize {
                let data = enc.alloc(size_of::<$ty>());
                let src = self.to_le_bytes();
                slice_copy(data, &src);
                self.size()
            }

            /// Unpacks this value from the given data.
            fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
                let size = size_of::<$ty>();
                check_unpack_len!(self, data, size);
                *self = $ty::from_le_bytes(
                    data[..size]
                        .try_into()
                        .map_err(|_e| packer_error!(
                            "overflow while unpacking {}",
                            std::any::type_name::<Self>(),
                        ))?
                );
                Ok(size)
            }
        }
    };
}

/// Implement`Packer` for bool type.
impl Packer for bool {
    fn size(&self) -> usize {
        1usize
    }

    /// Packs this value into the given encoder.
    fn pack(&self, enc: &mut Encoder) -> usize {
        let data = enc.alloc(1);
        if *self {
            data[0] = 1u8;
        } else {
            data[0] = 0u8;
        }
        1
    }

    /// Unpacks this value from the given data.
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, data, 1);
        if data[0] == 1 {
            *self = true;
        } else if data[0] == 0 {
            *self = false;
        } else {
            return Err(packer_error!("bool unpack: invalid raw bool value {}", data[0]));
        }
        Ok(1)
    }
}

/// Implement `Packer` for i8 type.
impl Packer for i8 {
    /// Returns the size of this value in bytes.
    fn size(&self) -> usize {
        1
    }

    /// Packs this value into the given encoder.
    fn pack(&self, enc: &mut Encoder) -> usize {
        let data = enc.alloc(1);
        data[0] = *self as u8;
        1
    }

    /// Unpacks this value from the given data.
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, data, 1);
        *self = data[0] as i8;
        Ok(1)
    }
}

/// Implement `Packer` for u8 type.
impl Packer for u8 {
    /// Returns the size of this value in bytes.
    fn size(&self) -> usize {
        1
    }

    /// Packs this value into the given encoder.
    fn pack(&self, enc: &mut Encoder) -> usize {
        let data = enc.alloc(1);
        data[0] = *self;
        1
    }

    /// Unpacks this value from the given data.
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        check_unpack_len!(self, data, 1);
        *self = data[0];
        Ok(1)
    }
}

impl_packed!(i16);
impl_packed!(u16);
impl_packed!(i32);
impl_packed!(u32);
impl_packed!(i64);
impl_packed!(u64);
impl_packed!(i128);
impl_packed!(u128);
impl_packed!(f32);
impl_packed!(f64);

/// Implement `Packer` for `String` type.
impl Packer for String {
    /// Returns the size of this value in bytes.
    fn size(&self) -> usize {
        VarUint32::from(self.len()).size() + self.len()
    }

    /// Packs this value into the given encoder.
    fn pack(&self, enc: &mut Encoder) -> usize {
        let pos = enc.get_size();

        let raw = self.as_bytes();

        let n = VarUint32::from(raw.len());
        n.pack(enc);

        let data = enc.alloc(raw.len());
        slice_copy(data, raw);

        enc.get_size() - pos
    }

    /// Unpacks this value from the given data.
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        // First 1 to 4 bytes is gonna be LEB128 encoded u32
        // with length of string
        let mut length = VarUint32::default();
        let size: usize = length.unpack(data)?;
        // TODO: Non utf-8 strings will return error, but leap supports them?
        *self = String::from_utf8(
            data[size..size + usize::from(&length)].to_vec()
        ).map_err(|e| packer_error!("{} while unpacking string", e.to_string()))?;
        Ok(size + usize::from(length))
    }
}

/// Implement `Packer` for `Vec<T>` type.
impl<T> Packer for Vec<T>
where
    T: Packer + Default,
{
    /// Returns the size of this value in bytes.
    fn size(&self) -> usize {
        if self.is_empty() {
            return 1;
        }

        let mut size: usize = 0;
        for i in self {
            size += i.size();
        }
        VarUint32::from(size as u32).size() + size
    }

    /// Packs this value into the given encoder.
    fn pack(&self, enc: &mut Encoder) -> usize {
        let pos = enc.get_size();
        let len = VarUint32::from(self.len());
        len.pack(enc);
        for v in self {
            v.pack(enc);
        }
        enc.get_size() - pos
    }

    /// Unpacks this value from the given data.
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        // First 1 to 4 bytes is gonna be LEB128 encoded u32
        // with item length of vector
        let mut dec = Decoder::new(data);
        let mut size = VarUint32::default();
        dec.unpack(&mut size)?;
        self.reserve((&size).into());
        // Each item will be packed next to each other
        for _ in 0..usize::from(size) {
            let mut v: T = Default::default();
            dec.unpack(&mut v)?;
            self.push(v);
        }
        Ok(dec.get_pos())
    }
}

/// Implement `Packer` for `Option<T>` type.
impl<T> Packer for Option<T>
where
    T: Packer + Default,
{
    /// Returns the size of this value in bytes.
    fn size(&self) -> usize {
        match self {
            Some(x) => 1 + x.size(),
            None => 1,
        }
    }

    /// Packs this value into the given encoder.
    fn pack(&self, enc: &mut Encoder) -> usize {
        let pos = enc.get_size();
        match self {
            Some(x) => {
                1u8.pack(enc);
                x.pack(enc);
            }
            None => {
                0u8.pack(enc);
            }
        }
        enc.get_size() - pos
    }

    /// Unpacks this value from the given data.
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        // TODO: no more bytes to read is ok?
        // is this for extension support?
        if data.is_empty() {
            *self = None;
            return Ok(0);
        }
        // Decode actual flag
        let mut dec = Decoder::new(data);
        let mut ty: u8 = 0;
        let mut value: T = Default::default();
        dec.unpack(&mut ty)?;
        // Flag indicates no value present
        if ty == 0 {
            *self = None;
            return Ok(1);
        }
        // Only other allowed value is 1, assert
        if ty != 1 {
            return Err(packer_error!("bad option type!: {}", ty))
        }
        // Finally unpack into underlying value
        dec.unpack(&mut value)?;
        *self = Some(value);
        Ok(dec.get_pos())
    }
}

/// Implement `Packer` for `Box<T>` type.
impl<T> Packer for Box<T>
where
    T: Packer + Default,
{
    /// Returns the size of this value in bytes.
    fn size(&self) -> usize {
        (**self).size()
    }

    /// Packs this value into the given encoder.
    fn pack(&self, enc: &mut Encoder) -> usize {
        (**self).pack(enc)
    }

    /// Unpacks this value from the given data.
    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        (**self).unpack(data)
    }
}

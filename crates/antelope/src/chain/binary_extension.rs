use crate::serializer::{Encoder, Packer, PackerError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct BinaryExtension<T: Packer + Default> {
    pub value: Option<T>,
}

impl<T> BinaryExtension<T>
where
    T: Packer + Default,
{
    pub fn new(value: Option<T>) -> Self {
        Self { value }
    }

    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

impl<T> Packer for BinaryExtension<T>
where
    T: Packer + Default,
{
    fn size(&self) -> usize {
        if let Some(x) = &self.value {
            x.size()
        } else {
            0
        }
    }

    fn pack(&self, enc: &mut Encoder) -> usize {
        let pos = enc.get_size();
        if let Some(x) = &self.value {
            x.pack(enc);
        }
        enc.get_size() - pos
    }

    fn unpack(&mut self, data: &[u8]) -> Result<usize, PackerError> {
        if !data.is_empty() {
            let mut value = T::default();
            let size = value.unpack(data)?;
            self.value = Some(value);
            Ok(size)
        } else {
            self.value = None;
            Ok(0)
        }
    }
}

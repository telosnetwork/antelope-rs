use std::fmt::{Display, Formatter};

#[derive(Clone, Copy)]
pub enum KeyType {
    K1,
    R1,
    // ... other variants ...
}

impl Display for KeyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::K1 => { write!(f, "K1") }
            KeyType::R1 => { write!(f, "R1") }
        }
    }
}
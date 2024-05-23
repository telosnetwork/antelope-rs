use antelope_client_macros::StructPacker;
use crate::chain::action::PermissionLevel;
use crate::chain::public_key::{deserialize_public_key, PublicKey};
use crate::serializer::{Decoder, Encoder, Packer};
use serde::{Deserialize, Serialize};

// Assuming basic types like PublicKey and PermissionLevel are defined elsewhere

/// KeyWeight associates a PublicKey with a Weight.
#[derive(Serialize, Deserialize, Debug, Clone, Default, StructPacker)]
pub struct KeyWeight {
    #[serde(deserialize_with = "deserialize_public_key")]
    pub key: PublicKey,
    pub weight: u16,
}

/// PermissionLevelWeight associates a PermissionLevel with a Weight.
#[derive(Serialize, Deserialize, Debug, Clone, Default, StructPacker)]
pub struct PermissionLevelWeight {
    pub permission: PermissionLevel,
    pub weight: u16,
}

/// WaitWeight associates a wait time (in seconds) with a Weight.
#[derive(Serialize, Deserialize, Debug, Clone, Default, StructPacker)]
pub struct WaitWeight {
    pub wait_sec: u32,
    pub weight: u16,
}

/// Authority defines a set of keys and/or accounts that can authorize an action.
#[derive(Serialize, Deserialize, Debug, Clone, Default, StructPacker)]
pub struct Authority {
    pub threshold: u32,
    pub keys: Vec<KeyWeight>,
    pub accounts: Vec<PermissionLevelWeight>,
    pub waits: Vec<WaitWeight>,
}

impl Authority {
    pub fn new(threshold: u32) -> Self {
        Authority {
            threshold,
            keys: Vec::new(),
            accounts: Vec::new(),
            waits: Vec::new(),
        }
    }

    pub fn new_single_key(public_key: PublicKey) -> Self {
        let mut authority = Authority::new(1);
        authority.keys.push(KeyWeight {
            key: public_key,
            weight: 1,
        });
        authority
    }

    pub fn wait_threshold(&self) -> u16 {
        self.waits.iter().map(|w| w.weight).sum()
    }

    pub fn key_threshold(&self) -> u32 {
        self.threshold - self.wait_threshold() as u32
    }

    pub fn key_weight(&self, public_key: &PublicKey) -> u16 {
        self.keys
            .iter()
            .find(|&key_weight| &key_weight.key == public_key)
            .map_or(0, |key_weight| key_weight.weight)
    }

    pub fn has_permission(&self, public_key: &PublicKey, include_partial: bool) -> bool {
        let threshold = if include_partial {
            1
        } else {
            self.key_threshold()
        };
        self.key_weight(public_key) >= threshold as u16
    }

    pub fn sort(&mut self) {
        self.keys.sort_unstable_by_key(|k| k.key.clone());
        self.accounts.sort_unstable_by_key(|a| a.permission);
        self.waits.sort_unstable_by_key(|w| w.wait_sec);
    }
}

// impl Packer for Weight {
//     fn size(&self) -> usize {
//         2 // Weight is 2 bytes (since it's a u16)
//     }
//
//     fn pack(&self, enc: &mut Encoder) -> usize {
//         // Convert the u16 Weight value to a byte array
//         let data = self.0.to_le_bytes(); // Assuming little-endian encoding
//                                          // Use a similar approach to pack_checksum to copy the bytes into the encoder
//         let allocated = enc.alloc(self.size());
//         slice_copy(allocated, &data);
//         self.size()
//     }
//
//     fn unpack(&mut self, raw: &[u8]) -> usize {
//         let size = self.size();
//         assert!(raw.len() >= size, "Weight.unpack: buffer overflow!");
//         // Assuming the data is little-endian encoded, directly reconstruct the u16 value
//         self.0 = u16::from_le_bytes([raw[0], raw[1]]);
//         size
//     }
// }
//
// impl Packer for KeyWeight {
//     fn size(&self) -> usize {
//         self.key.size() + self.weight.size()
//     }
//
//     fn pack(&self, enc: &mut Encoder) -> usize {
//         let pos = enc.get_size();
//         self.key.pack(enc);
//         self.weight.pack(enc);
//         enc.get_size() - pos
//     }
//
//     fn unpack(&mut self, data: &[u8]) -> usize {
//         let mut dec = Decoder::new(data);
//         dec.unpack(&mut self.key);
//         dec.unpack(&mut self.weight);
//         dec.get_pos()
//     }
// }
//
// impl Packer for WaitWeight {
//     fn size(&self) -> usize {
//         4 + self.weight.size() // wait_sec is a u32 (4 bytes) + size of weight
//     }
//
//     fn pack(&self, enc: &mut Encoder) -> usize {
//         let pos = enc.get_size();
//         // Convert the u32 wait_sec value to a byte array and pack it
//         let wait_sec_bytes = self.wait_sec.to_le_bytes(); // Assuming little-endian encoding
//         let allocated_wait_sec = enc.alloc(4); // Allocate 4 bytes for wait_sec
//         slice_copy(allocated_wait_sec, &wait_sec_bytes); // Copy wait_sec_bytes into the encoder
//
//         // Pack weight
//         self.weight.pack(enc);
//
//         enc.get_size() - pos
//     }
//
//     fn unpack(&mut self, data: &[u8]) -> usize {
//         assert!(
//             data.len() >= self.size(),
//             "WaitWeight.unpack: buffer overflow!"
//         );
//         // Assuming the data is little-endian encoded, directly reconstruct the u32 wait_sec value
//         self.wait_sec = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
//         let mut dec = Decoder::new(&data[4..]); // Create a new Decoder starting after the first 4 bytes
//         dec.unpack(&mut self.weight);
//         4 + dec.get_pos() // Return total bytes read (4 bytes for wait_sec + bytes read for weight)
//     }
// }
//
// impl Packer for Authority {
//     fn size(&self) -> usize {
//         // Calculate the size based on the dynamic content of the Authority struct
//         4 + // for threshold (u32)
//             self.keys.iter().map(Packer::size).sum::<usize>() +
//             self.accounts.iter().map(Packer::size).sum::<usize>() +
//             self.waits.iter().map(Packer::size).sum::<usize>()
//     }
//
//     fn pack(&self, enc: &mut Encoder) -> usize {
//         let pos = enc.get_size();
//         // Convert the u32 threshold value to a byte array and pack it
//         let threshold_bytes = self.threshold.to_le_bytes(); // Assuming little-endian encoding
//         let allocated_threshold = enc.alloc(4); // Allocate 4 bytes for threshold
//         slice_copy(allocated_threshold, &threshold_bytes); // Copy threshold_bytes into the encoder
//
//         // Iterate over keys, accounts, and waits to pack them
//         for key_weight in &self.keys {
//             key_weight.pack(enc);
//         }
//         for account in &self.accounts {
//             account.pack(enc);
//         }
//         for wait_weight in &self.waits {
//             wait_weight.pack(enc);
//         }
//
//         enc.get_size() - pos
//     }
//
//     fn unpack(&mut self, data: &[u8]) -> usize {
//         assert!(
//             data.len() >= 4,
//             "Authority.unpack: buffer underflow for threshold!"
//         );
//         // Assuming the data is little-endian encoded, directly reconstruct the u32 threshold value
//         self.threshold = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
//
//         let mut total_bytes_read = 4;
//         let mut dec = Decoder::new(&data[total_bytes_read..]);
//
//         // Unpack keys
//         for key_weight in &mut self.keys {
//             total_bytes_read += dec.unpack(key_weight);
//         }
//         // Unpack accounts
//         for account in &mut self.accounts {
//             total_bytes_read += dec.unpack(account);
//         }
//         // Unpack waits
//         for wait_weight in &mut self.waits {
//             total_bytes_read += dec.unpack(wait_weight);
//         }
//
//         total_bytes_read
//     }
// }
//
// impl Packer for PermissionLevelWeight {
//     fn size(&self) -> usize {
//         self.permission.size() + self.weight.size()
//     }
//
//     fn pack(&self, enc: &mut Encoder) -> usize {
//         let pos = enc.get_size();
//         // Pack `permission` and `weight` into the encoder
//         self.permission.pack(enc);
//         self.weight.pack(enc);
//         enc.get_size() - pos
//     }
//
//     fn unpack(&mut self, data: &[u8]) -> usize {
//         let mut dec = Decoder::new(data);
//         // Unpack `permission` and `weight` from the data
//         dec.unpack(&mut self.permission);
//         dec.unpack(&mut self.weight);
//         dec.get_pos()
//     }
// }

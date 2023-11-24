use ripemd::{Digest as RipeDigest, Ripemd160};
use sha2::Sha256;
use crate::chain::private_key::KeyType;

pub fn decode_ripemd160_check(encoded: &str, size: Option<usize>, key_type: KeyType) -> Result<Vec<u8>, String> {
    // Decode the Base58 string
    let decoded = bs58::decode(encoded).into_vec().map_err(|e| e.to_string())?;

    // Check if the decoded data is at least 5 bytes (4 for the checksum)
    if decoded.len() < 5 {
        return Err("Data is too short".to_string());
    }

    // Split the data and checksum
    let (data, checksum) = decoded.split_at(decoded.len() - 4);

    // Calculate RIPEMD-160 hash of the data
    let hash = Ripemd160::digest(data);

    // Verify the checksum
    if checksum != &hash[..4] {
        return Err("Checksum mismatch".to_string());
    }

    return Ok(data.to_vec());
}

pub fn decode_check(encoded: &str) -> Result<Vec<u8>, String> {
    let decoded = bs58::decode(encoded).into_vec().map_err(|e| e.to_string())?;

    if decoded.len() < 4 {
        return Err("Data too short for checksum".to_string());
    }

    let (data, checksum) = decoded.split_at(decoded.len() - 4);
    let expected_checksum = double_sha_checksum(data.to_vec());

    if checksum != expected_checksum {
        return Err("Checksum mismatch".to_string());
    }

    return Ok(data.to_vec());
}

pub fn decode_key(value: &str) -> Result<(KeyType, Vec<u8>), String> {
    if value.starts_with("PVT_") {
        let parts: Vec<&str> = value.split('_').collect();
        if parts.len() != 3 {
            return Err("Invalid PVT format".to_string());
        }
        let key_type = match parts[1] {
            "K1" => KeyType::K1,
            "R1" => KeyType::R1,
            // ... handle other key types ...
            _ => return Err("Invalid key type".to_string()),
        };
        let size = match key_type {
            KeyType::K1 | KeyType::R1 => Some(32),
            // ... other cases ...
        };
        let data = decode_ripemd160_check(parts[2], size, key_type).unwrap();
        Ok((key_type, data))
    } else {
        // WIF format
        let key_type = KeyType::K1;
        let mut data = decode_check(value).unwrap();
        if data[0] != 0x80 {
            return Err("Invalid WIF".to_string());
        }
        data.remove(0); // droppingFirst equivalent
        Ok((key_type, data))
    }
}

pub fn encode_check(data: Vec<u8>) -> String {
    let double_hash = double_sha_checksum(data.to_vec());
    let mut with_checksum = data.to_vec();
    with_checksum.append(&mut double_hash.to_vec());
    return bs58::encode(with_checksum).into_string();
}

pub fn encode_ripemd160_check(data: Vec<u8>, suffix: Option<&str>) -> String {
    let mut hasher = Ripemd160::new();
    hasher.update(data.to_vec());
    if let Some(s) = suffix {
        hasher.update(s);
    }
    let ripe_hash = hasher.finalize();


    let mut with_ripe_checksum = data.to_vec();
    with_ripe_checksum.append(&mut ripe_hash.as_slice().to_vec());
    return bs58::encode(with_ripe_checksum).into_string();
}

fn double_sha_checksum(data: Vec<u8>) -> Vec<u8> {
    let data_hash = Sha256::digest(Sha256::digest(data));
    let checksum = &data_hash[..4];
    return checksum.to_vec();
}
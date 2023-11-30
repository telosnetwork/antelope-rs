use ripemd::{Digest as RipeDigest, Ripemd160};
use sha2::Sha256;
use crate::base58;
use crate::chain::key_type::KeyType;

pub fn encode(data: Vec<u8>) -> String {
    return bs58::encode(data).into_string();
}

pub fn decode(encoded: &str, size: Option<usize>) -> Result<Vec<u8>, String> {
    let decode_result = bs58::decode(encoded).into_vec();
    if decode_result.is_err() {
        return Err(format!("Failed to decode str {encoded}"));
    }
    let decoded = decode_result.unwrap();

    if size.is_some() && decoded.len() != size.unwrap() {
        return Err(String::from("Size did not match"));
    }

    return Ok(decoded);
}

pub fn decode_ripemd160_check(encoded: &str, size: Option<usize>, key_type: Option<KeyType>) -> Result<Vec<u8>, String> {
    let decoded = bs58::decode(encoded).into_vec().map_err(|e| e.to_string())?;

    if decoded.len() < 5 {
        return Err("Data is too short".to_string());
    }

    let (data, checksum) = decoded.split_at(decoded.len() - 4);
    let suffix = key_type.as_ref().map(KeyType::to_string);
    let hash = ripemd160_checksum(data.to_vec(), suffix.as_deref());

    // Verify the checksum
    if checksum != &hash[..4] {
        return Err("Checksum mismatch".to_string());
    }

    if size.is_some() {
        let size_value = size.unwrap() + 4;
        if data.len() > size_value {
            return Ok(data[0..size_value].to_vec());
        }
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

pub fn decode_public_key(value: &str) -> Result<(KeyType, Vec<u8>), String> {
    if value.starts_with("PUB_") {
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
        let data = decode_ripemd160_check(parts[2], size, Option::from(key_type)).unwrap();
        return Ok((key_type, data));
    } else if value.len() > 50 {
        let without_prefix = value.chars().skip(value.len() - 50).collect::<String>();
        let data = base58::decode_ripemd160_check(without_prefix.as_str(), None, None);
        return Ok((KeyType::K1, data.unwrap().to_vec()));
    } else {
        return Err(String::from("Public key format invalid"));
    }
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
        let data = decode_ripemd160_check(parts[2], size, Some(key_type)).unwrap();
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
    let mut ripe_checksum = ripemd160_checksum(data.to_vec(), suffix);

    let mut with_ripe_checksum = data.to_vec();
    with_ripe_checksum.append(&mut ripe_checksum.to_vec());
    return bs58::encode(with_ripe_checksum).into_string();
}

fn ripemd160_checksum(data: Vec<u8>, suffix: Option<&str>) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(data.to_vec());
    if let Some(s) = suffix {
        hasher.update(s);
    }
    let ripe_hash = hasher.finalize();
    return ripe_hash.as_slice()[0..4].to_vec();
}

fn double_sha_checksum(data: Vec<u8>) -> Vec<u8> {
    let data_hash = Sha256::digest(Sha256::digest(data));
    let checksum = &data_hash[..4];
    return checksum.to_vec();
}
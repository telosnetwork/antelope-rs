use bs58;
use ripemd::{Digest as RipeDigest, Ripemd160};
use sha2::Sha256;

use crate::{base58, chain::key_type::KeyType};

pub fn encode(data: Vec<u8>) -> String {
    bs58::encode(data).into_string()
}

pub fn decode(encoded: &str, size: Option<usize>) -> Result<Vec<u8>, String> {
    let decoded = bs58::decode(encoded)
        .into_vec()
        .map_err(|_| format!("Failed to decode str {encoded}"))?;

    if let Some(expected) = size {
        if decoded.len() != expected {
            return Err("Size did not match".to_string());
        }
    }

    Ok(decoded)
}

pub fn decode_ripemd160_check(
    encoded: &str,
    size: Option<usize>,
    key_type: Option<KeyType>,
    ignore_checksum: bool,
) -> Result<Vec<u8>, String> {
    let decoded = bs58::decode(encoded)
        .into_vec()
        .map_err(|e| e.to_string())?;

    if decoded.len() < 5 {
        return Err("Data is too short".to_string());
    }

    let (data, checksum) = decoded.split_at(decoded.len() - 4);
    let suffix = key_type.as_ref().map(KeyType::to_string);
    let expected = ripemd160_checksum(data, suffix.as_deref());

    if !ignore_checksum && checksum != expected.as_slice() {
        return Err("Checksum mismatch".to_string());
    }

    if let Some(base) = size {
        let limit = base + 4;
        if data.len() > limit {
            return Ok(data[..limit].to_vec());
        }
    }

    Ok(data.to_vec())
}

pub fn decode_check(encoded: &str, ignore_checksum: bool) -> Result<Vec<u8>, String> {
    let decoded = bs58::decode(encoded)
        .into_vec()
        .map_err(|e| e.to_string())?;

    if decoded.len() < 4 {
        return Err("Data too short for checksum".to_string());
    }

    let (data, checksum) = decoded.split_at(decoded.len() - 4);
    let expected = double_sha_checksum(data);

    if !ignore_checksum && checksum != expected.as_slice() {
        return Err("Checksum mismatch".to_string());
    }

    Ok(data.to_vec())
}

pub fn decode_public_key(value: &str) -> Result<(KeyType, Vec<u8>), String> {
    if let Some(rest) = value.strip_prefix("PUB_") {
        let mut parts = rest.split('_');
        let ty = parts.next().ok_or("Invalid key type")?;
        let body = parts.next().ok_or("Invalid format")?;
        if parts.next().is_some() {
            return Err("Invalid PVT format".to_string());
        }

        let key_type = match ty {
            "K1" => KeyType::K1,
            "R1" => KeyType::R1,
            "WA" => KeyType::WA,
            _ => return Err("Invalid key type".to_string()),
        };

        let size = match key_type {
            KeyType::K1 | KeyType::R1 => Some(32),
            KeyType::WA => None,
        };

        let data = decode_ripemd160_check(body, size, Some(key_type), false)?;
        return Ok((key_type, data));
    }

    if value.len() > 50 {
        let without_prefix: String = value.chars().skip(value.len() - 50).collect();
        let data = base58::decode_ripemd160_check(&without_prefix, Some(32), None, false)?;
        return Ok((KeyType::K1, data));
    }

    Err("Public key format invalid".to_string())
}

pub fn decode_key(value: &str, ignore_checksum: bool) -> Result<(KeyType, Vec<u8>), String> {
    if let Some(rest) = value.strip_prefix("PVT_") {
        let mut parts = rest.split('_');
        let ty = parts.next().ok_or("Invalid key type")?;
        let body = parts.next().ok_or("Invalid format")?;
        if parts.next().is_some() {
            return Err("Invalid PVT format".to_string());
        }

        let key_type = match ty {
            "K1" => KeyType::K1,
            "R1" => KeyType::R1,
            _ => return Err("Invalid key type".to_string()),
        };

        let size = match key_type {
            KeyType::K1 | KeyType::R1 => Some(32),
            KeyType::WA => None,
        };

        let data = decode_ripemd160_check(body, size, Some(key_type), ignore_checksum)?;
        return Ok((key_type, data));
    }

    // WIF
    let key_type = KeyType::K1;
    let mut data = decode_check(value, ignore_checksum)?;

    if data.first() != Some(&0x80) {
        return Err("Invalid WIF".to_string());
    }

    data.remove(0);
    Ok((key_type, data))
}

pub fn encode_check(mut data: Vec<u8>) -> String {
    let checksum = double_sha_checksum(&data);
    data.extend_from_slice(&checksum);
    bs58::encode(data).into_string()
}

pub fn encode_ripemd160_check(mut data: Vec<u8>, suffix: Option<&str>) -> String {
    let checksum = ripemd160_checksum(&data, suffix);
    data.extend_from_slice(&checksum);
    bs58::encode(data).into_string()
}

fn ripemd160_checksum(data: &[u8], suffix: Option<&str>) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    if let Some(s) = suffix {
        hasher.update(s.as_bytes());
    }
    hasher.finalize()[..4].to_vec()
}

fn double_sha_checksum(data: &[u8]) -> Vec<u8> {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    second[..4].to_vec()
}

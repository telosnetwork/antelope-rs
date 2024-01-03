use std::io::Write;
use hex::{decode, encode};
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    decode(hex).unwrap()
}

pub fn bytes_to_hex(bytes: &Vec<u8>) -> String {
    encode(bytes)
}

pub fn array_equals<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x == y)
}

pub fn array_to_hex(bytes: &[u8]) -> String {
    //bytes.iter().map(|b| format!("{:02x}", b)).collect()
    encode(bytes)
}

pub fn slice_copy(dst: &mut [u8], src: &[u8]) {
    dst.copy_from_slice(src);
    // assert!(dst.len() == src.len(), "copy_slice: length not the same!");
    // unsafe { memcpy(dst.as_mut_ptr(), src.as_ptr(), dst.len()); }
}

pub fn zlib_compress(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    if e.write_all(bytes).is_err() {
        return Err("Error during compression".into());
    }
    let compressed_bytes = e.finish();
    if compressed_bytes.is_err() {
        return Err("Error during compression".into());
    }
    Ok(compressed_bytes.unwrap())
}
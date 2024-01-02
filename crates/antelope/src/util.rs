use std::io::Write;
use hex::{decode, encode};
use std::slice;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    return decode(hex).unwrap();
}

pub fn bytes_to_hex(bytes: &Vec<u8>) -> String {
    return encode(bytes);
}

pub fn array_equals<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x == y)
}

pub fn array_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn memcpy( dst: *mut u8, src: *const u8, length: usize) -> *mut u8 {
    let mut _dst = unsafe {
        slice::from_raw_parts_mut(dst, length)
    };

    let _src = unsafe {
        slice::from_raw_parts(src, length)
    };
    _dst.copy_from_slice(_src);
    dst
}

pub fn slice_copy(dst: &mut [u8], src: &[u8]) {
    assert!(dst.len() == src.len(), "copy_slice: length not the same!");
    memcpy(dst.as_mut_ptr(), src.as_ptr(), dst.len());
}

pub fn zlib_compress(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(bytes);
    let compressed_bytes = e.finish();
    if compressed_bytes.is_err() {
        return Err(String::from("Error during compression"));
    }
    Ok(compressed_bytes.unwrap())
}
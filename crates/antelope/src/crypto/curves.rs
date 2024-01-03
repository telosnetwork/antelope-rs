pub fn create_k1_field_bytes(bytes: &[u8]) -> k256::elliptic_curve::FieldBytes<k256::Secp256k1> {
    return *k256::elliptic_curve::FieldBytes::<k256::Secp256k1>::from_slice(bytes);
}

pub fn create_r1_field_bytes(bytes: &[u8]) -> p256::elliptic_curve::FieldBytes<p256::NistP256> {
    return *p256::elliptic_curve::FieldBytes::<p256::NistP256>::from_slice(bytes);
}

use crate::Result;
use num::{BigInt, Num};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

/// Append str to Vec<u8>.
pub fn append_str(buffer: &mut Vec<u8>, data: &str) {
    for value in data.bytes() {
        buffer.push(value);
    }
}

/// Convert hex string(hash) to BigInt.
pub fn hex_to_big_int(hex: &str) -> BigInt {
    BigInt::from_str_radix(hex, 16).unwrap()
}

/// Convert [u8] to hash.
pub fn hash_utf8(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);

    format!("{:x}", hasher.finalize())
}

/// Convert str to hash.
pub fn hash_str(data: impl Into<String>) -> String {
    let data = data.into();
    let mut hasher = Sha256::new();
    hasher.update(&data);

    format!("{:x}", hasher.finalize())
}

/// Encode str to base58.
pub fn base58_encode(data: &[u8]) -> String {
    bs58::encode(data)
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_string()
}

pub fn base58_decode(data: &str) -> Result<Vec<u8>> {
    let v = bs58::decode(data)
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_vec()?;
    Ok(v)
}

/// Calculate SHA256 hash.
pub fn sha256_digest(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);

    let res = hasher.finalize();
    res[..].into()
}

/// Calculate ripemd160 hash.
pub fn ripemd160_digest(data: &[u8]) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    let res = hasher.finalize();
    res[..].into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::hex_to_big_int;
    use std::ops::ShlAssign;

    #[test]
    fn test_hex_to_big_int() {
        let hex_str = "00000041662c5fc2883535dc19ba8a33ac993b535da9899e593ff98e1eda56a1".to_owned();
        let b = hex_to_big_int(&hex_str);
        println!("b {}", b);

        let mut target = BigInt::from(1);
        target.shl_assign(256 - 8);
        println!("t {}", target);

        assert!(b.lt(&target));
    }

    #[test]
    fn test_hash_utf8() {
        let data = "This is a data for tests".as_bytes();
        let hash = hash_utf8(data);
        println!("{}", hash);
    }
}

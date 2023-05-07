use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// One single part of the blockchain
/// Basically contains a list of transactions
#[derive(Clone, Debug)]
pub struct Block {
    /// The current timestamp when the block is created
    timestamp: u64,

    /// The actual valuable information in the block
    data: String,

    /// The hash of the previous block
    pub pre_hash: String,

    /// The hash of this block, also as block headers
    pub hash: String,
}

impl Block {
    /// New a genesis block
    pub fn new_genesis() -> Self {
        Self::new("Genesis Block".to_owned(), String::new())
    }

    pub fn new(data: String, pre_hash: String) -> Self {
        let mut block = Block {
            data,
            pre_hash,
            hash: String::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        block.set_hash();
        block
    }

    pub fn set_hash(&mut self) {
        let mut hasher = Sha256::new();
        let mut headers: Vec<u8> = vec![];

        // join timestamp, pre_hash and data to headers
        append_u64(&mut headers, &self.timestamp);
        append_str(&mut headers, self.pre_hash.as_str());
        append_str(&mut headers, self.data.as_str());

        hasher.update(headers);
        self.hash = format!("{:X}", hasher.finalize());
    }
}

fn append_str(buffer: &mut Vec<u8>, data: &str) {
    for value in data.bytes() {
        buffer.push(value);
    }
}
fn append_u64(buffer: &mut Vec<u8>, data: &u64) {
    let values = data.to_be_bytes();
    for value in values {
        buffer.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Block;
    use sha2::Sha256;

    #[test]
    fn test_set_hash() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut block = Block {
            timestamp,
            pre_hash: "pre_hash".to_owned(),
            data: "data".to_owned(),
            hash: String::new(),
        };
        block.set_hash();

        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_be_bytes());
        hasher.update(b"pre_hashdata");
        let hash = format!("{:X}", hasher.finalize());

        println!("hash - {}", block.hash);
        println!("data - {}", block.data);
        assert_eq!(hash, block.hash);
    }

    #[test]
    fn test_new_block() {
        let block = Block::new(
            "data1".to_owned(),
            "16C90CF81A56919922EDFC29BFE5D5E39D098B4F05A50A68568566E524B130E4".to_owned(),
        );
        println!("block {:?}", block);
    }
}

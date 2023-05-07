use crate::proof_of_work::ProofOfWork;
use log::debug;
use std::time::{SystemTime, UNIX_EPOCH};

/// One single part of the blockchain.
/// Basically contains a list of transactions.
#[derive(Clone, Debug)]
pub struct Block {
    /// The current timestamp when the block is created.
    pub timestamp: u64,

    /// The actual valuable information in the block.
    pub data: String,

    /// The hash of the previous block.
    pub pre_hash: String,

    /// The hash of this block, also as block headers.
    pub hash: String,

    /// The nonce from Proof-of-Work mining.
    pub nonce: u64,
}

impl Block {
    /// New a genesis block.
    pub fn new_genesis() -> Self {
        debug!("new a genesis block");
        Self::new("Genesis Block".to_owned(), String::new())
    }

    /// New a block with some data and the previous hash.
    pub fn new(data: String, pre_hash: String) -> Self {
        let mut block = Block {
            data,
            pre_hash,
            hash: String::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce: 0,
        };
        let pow = ProofOfWork::new(block.clone());
        let (nonce, hash) = pow.run();
        debug!("The block get nonce {}", nonce);
        block.hash = hash;
        block.nonce = nonce;
        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Block;
    use sha2::Sha256;

    #[test]
    fn test_new_block() {
        let block = Block::new(
            "data1".to_owned(),
            "16C90CF81A56919922EDFC29BFE5D5E39D098B4F05A50A68568566E524B130E4".to_owned(),
        );
        println!("block {:?}", block);
    }
}

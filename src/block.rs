use crate::proof_of_work::ProofOfWork;
use crate::Result;
use log::info;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// One single part of the blockchain.
/// Basically contains a list of transactions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
        info!("The block get nonce {}", nonce);
        block.hash = hash;
        block.nonce = nonce;
        block
    }

    /// Serialize a block to String.
    pub fn serialize(&self) -> Result<String> {
        let serialization = ron::to_string(&self)?;
        Ok(serialization)
    }

    /// deserialize str to a block.
    pub fn deserialize(value: &str) -> Result<Self> {
        let block: Block = ron::from_str(value).map_err(|e| e.code)?;
        Ok(block)
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

    #[test]
    fn test_serialize() {
        let block = Block::new(
            "this is tests block".to_owned(),
            "16C90CF81A56919922EDFC29BFE5D5E39D098B4F05A50A68568566E524B130E4".to_owned(),
        );
        let str = block.serialize().expect("serialize error");
        println!("got: {}", str);
        let new_block = Block::deserialize(&str).expect("deserialize error");
        println!("new: {:?}", new_block);
        assert_eq!(block, new_block);
    }
}

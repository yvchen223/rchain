use crate::proof_of_work::ProofOfWork;
use crate::transaction::Transaction;
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

    /// The hash of the previous block.
    pub pre_hash: String,

    /// The hash of this block, also as block headers.
    pub hash: String,

    /// The nonce from Proof-of-Work mining.
    pub nonce: u64,

    /// Stores transactions.
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// New a genesis block.
    pub fn new_genesis(coinbase: Transaction) -> Self {
        Self::new(vec![coinbase], String::new())
    }

    /// New a block with some data and the previous hash.
    pub fn new(transactions: Vec<Transaction>, pre_hash: String) -> Self {
        let mut block = Block {
            transactions,
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

    /// deserialize transactions
    pub fn serialize_transactions(&self) -> Result<String> {
        let mut str = String::new();
        for tx in &self.transactions {
            str.push_str(ron::to_string(tx)?.as_str());
        }
        Ok(str)
    }
}

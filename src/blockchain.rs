use std::collections::HashMap;
use crate::block::Block;
use crate::engine::{SledEngine, LAST_HASH_OF_CHAIN};
use crate::{error, Result};
use log::info;
use std::path::PathBuf;
use crate::transaction::{Transaction, TXOutput};

const GENESIS_COINBASE_DATA: &str = "The Times 03/Jan/2009 Chancellor on brink of second bailout for bank";

/// The actual Blockchain container.
pub struct Blockchain {
    /// Hash of the last block
    pub tip: String,

    engine: SledEngine,
}

impl Blockchain {
    /// New a genesis Blockchain.
    pub fn new(path: impl Into<PathBuf>, address: String) -> Result<Self> {
        let engine = SledEngine::new(path)?;
        let tip = engine.get(LAST_HASH_OF_CHAIN)?;
        let tip = match tip {
            Some(v) => v,
            None => {
                info!("Creating a genesis block...");
                let cbtx = Transaction::new_coinbase_tx(address, GENESIS_COINBASE_DATA.to_owned());
                let genesis = Block::new_genesis(cbtx);
                let _ = engine.set(LAST_HASH_OF_CHAIN, &genesis.hash)?;
                engine.set(&genesis.hash, genesis.serialize()?).unwrap();
                genesis.hash
            }
        };
        Ok(Blockchain { tip, engine })
    }

    /// Will add a block to the Blockchain.
    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        // Get the last block hash from db
        let pre_hash = self.get_last_hash()?;

        // Mine a new block
        let block = Block::new(transactions, pre_hash);

        // Store the new block to db
        self.update_engine(&block)?;

        Ok(())
    }

    fn get_last_hash(&self) -> Result<String> {
        let last_hash = self.engine.get(LAST_HASH_OF_CHAIN)?;
        match last_hash {
            Some(v) => Ok(v),
            None => Err(error::Error::StringError(
                "There is not last hash in database".to_owned(),
            )),
        }
    }

    fn update_engine(&mut self, block: &Block) -> Result<()> {
        self.tip = block.hash.clone();
        self.engine.set(LAST_HASH_OF_CHAIN, &block.hash)?;
        self.engine.set(&block.hash, block.serialize()?)?;

        Ok(())
    }

    /// Return an iterator over the Blockchain
    pub fn iter(&self) -> BlockChainIterator {
        BlockChainIterator {
            cur_hash: self.tip.clone(),
            engine: self.engine.clone(),
        }
    }

    /// Find unspent transaction outputs.
    pub fn find_utxo(&self, address: String) -> HashMap<String, Vec<TXOutput>> {
        let mut iter = self.iter();
        let mut spent_txos: HashMap<String, Vec<usize>> = HashMap::new();

        let mut utxo = HashMap::new();

        loop {
            let block = iter.next();

            let Some(block) = block else { break };
            for tx in block.transactions {
                for (out_idx, output) in tx.vout.iter().enumerate() {
                    // Check if an output was already referenced in an input.
                    // Skip those that were referenced in inputs(their values were moved to other outputs,
                    // thus we cannot count them).
                    if let Some(mut idxs) = spent_txos.get(&tx.id) {
                        if idxs.contains(&out_idx) {
                            continue;
                        }
                    }

                    if output.can_be_unlocked_with(address.clone()) {
                        let outs_idx = utxo.entry(tx.id.clone()).or_insert(vec![]);
                        outs_idx.push(output.clone());
                    }
                }

                // Coinbase transaction don't unlock outputs.
                if !tx.is_coinbase() {
                    // We gather all inputs that could unlock outputs locked with the provided address.
                    for input in tx.vin {
                        if input.can_unlock_output_with(address.clone()) {
                            let idxs = spent_txos.entry(input.tx_id.clone()).or_insert(vec![]);
                            idxs.push(input.idx_vout);
                        }
                    }
                }
            }
        }

        utxo
    }
}

impl IntoIterator for Blockchain {
    type Item = Block;
    type IntoIter = BlockChainIterator;

    fn into_iter(self) -> Self::IntoIter {
        BlockChainIterator {
            cur_hash: self.tip,
            engine: self.engine,
        }
    }
}

pub struct BlockChainIterator {
    cur_hash: String,
    engine: SledEngine,
}

impl Iterator for BlockChainIterator {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        let block = self.engine.get(&self.cur_hash).unwrap();
        match block {
            Some(val) => {
                let block = Block::deserialize(&val).unwrap();
                self.cur_hash = block.pre_hash.clone();
                Some(block)
            }
            None => None,
        }
    }

}



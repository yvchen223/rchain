use crate::block::Block;
use crate::engine::{SledEngine, LAST_HASH_OF_CHAIN};
use crate::transaction::{TXOutput, Transaction};
use crate::{error, Result};
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;

const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for bank";

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

    /// Will mine a block to the Blockchain.
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<()> {
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
    pub fn find_utxo(&self, address: &str) -> HashMap<String, Vec<(usize, TXOutput)>> {
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
                    if let Some(idxs) = spent_txos.get(&tx.id) {
                        if idxs.contains(&out_idx) {
                            continue;
                        }
                    }

                    if output.is_locked_with_key(address) {
                        let outs_idx = utxo.entry(tx.id.clone()).or_insert(vec![]);
                        outs_idx.push((out_idx, output.clone()));
                    }
                }

                // Coinbase transaction don't unlock outputs.
                if !tx.is_coinbase() {
                    // We gather all inputs that could unlock outputs locked with the provided address.
                    for input in tx.vin {
                        if input.use_key(address) {
                            let idxs = spent_txos.entry(input.tx_id.clone()).or_insert(vec![]);
                            idxs.push(input.idx_vout);
                        }
                    }
                }
            }
        }
        utxo
    }

    /// Iterate over all unspent transactions and accumulate their values.
    /// When the accumulated value is more or equal to the amount we want to transfer, it returns.
    /// Return the accumulated value and map(K -> tx_id, V -> the vector of output index in the transaction).
    pub fn find_spendable_outputs(
        &self,
        address: &str,
        amount: i64,
    ) -> (i64, HashMap<String, Vec<usize>>) {
        let mut outputs_idx = HashMap::new();
        let mut acc = 0;

        let utxo = self.find_utxo(address);

        'find_acc: for (tx_id, outputs) in utxo {
            for (output_idx, output) in outputs {
                if output.is_locked_with_key(address) && acc < amount {
                    acc += output.value;
                    let entry = outputs_idx.entry(tx_id.clone()).or_insert(vec![]);
                    entry.push(output_idx);

                    if acc >= amount {
                        break 'find_acc;
                    }
                }
            }
        }
        (acc, outputs_idx)
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

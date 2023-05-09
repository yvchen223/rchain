use crate::block::Block;
use crate::engine::{SledEngine, LAST_HASH_OF_CHAIN};
use crate::{error, Result};
use std::env;

/// The actual Blockchain container.
pub struct Blockchain {
    /// Stores all the blocks which are accepted already within the blockchain.
    pub blocks: Vec<Block>,

    pub tip: String,
    engine: SledEngine,
}

impl Blockchain {
    /// New a genesis Blockchain.
    pub fn new() -> Self {
        // TODO rm unwrap
        let engine = SledEngine::new(env::current_dir().unwrap()).unwrap();
        let tip = engine.get(LAST_HASH_OF_CHAIN).unwrap();
        let tip = match tip {
            Some(v) => v,
            None => {
                let genesis = Block::new_genesis();
                let _ = engine.set(LAST_HASH_OF_CHAIN, &genesis.hash).unwrap();
                engine
                    .set(&genesis.hash, genesis.serialize().unwrap())
                    .unwrap();
                genesis.hash
            }
        };
        Blockchain {
            blocks: vec![],
            tip,
            engine,
        }
    }

    /// Will add a block to the Blockchain.
    pub fn add_block(&mut self, data: String) -> Result<()> {
        // Get the last block hash from db
        let pre_hash = self.get_last_hash()?;

        // Mine a new block
        let block = Block::new(data, pre_hash);

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_new_blockchain() {
        env_logger::init();
        let mut chain = Blockchain::new();

        thread::sleep(Duration::from_secs(1));
        chain
            .add_block("Send 1 BTC to user_a".to_owned())
            .expect("add error");

        thread::sleep(Duration::from_secs(1));
        chain
            .add_block("Send 2 BTC to user_b".to_owned())
            .expect("add error");

        let mut iter = chain.into_iter();
        while let Some(block) = iter.next() {
            println!("block: {:?}", block);
        }
    }

    #[test]
    fn test_tip_in_new_blockchain() {
        let tip_1;
        let tip_2;
        {
            let mut chain = Blockchain::new();
            println!("tip 1 = {}", chain.tip);
            tip_1 = chain.tip.clone();
        }
        {
            let mut chain = Blockchain::new();
            println!("tip 2 = {}", chain.tip);
            tip_2 = chain.tip.clone();
        }
        assert!(tip_1.len() > 0);
        assert!(tip_2.len() > 0);
        assert_eq!(tip_1, tip_2);
    }
}

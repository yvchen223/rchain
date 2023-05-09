use std::env;
use serde::de::Unexpected::Str;
use crate::block::Block;
use crate::engine::{LAST_HASH_OF_CHAIN, SledEngine};
use crate::{error, Result};

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
        let tip = engine.get(LAST_HASH_OF_CHAIN.to_owned()).unwrap();
        let tip = match tip {
            Some(v) => v,
            None => {
                let genesis = Block::new_genesis();
                let _ = engine.set(LAST_HASH_OF_CHAIN.to_owned(), genesis.hash.clone()).unwrap();
                engine.set(genesis.hash.clone(), genesis.serialize().unwrap()).unwrap();
                genesis.hash.clone()
            }
        };
        Blockchain {
            blocks: vec![],
            tip,
            engine
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
        let last_hash = self.engine.get(LAST_HASH_OF_CHAIN.to_owned())?;
        match last_hash {
            Some(v) => Ok(v),
            None => Err(error::Error::StringError("There is not last hash in database".to_owned())),
        }
    }

    fn update_engine(&mut self, block: &Block) -> Result<()> {

        self.tip = block.hash.clone();
        self.engine.set(LAST_HASH_OF_CHAIN.to_owned(), block.hash.clone())?;
        self.engine.set(block.hash.clone(), block.serialize()?)?;

        Ok(())
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
        chain.add_block("Send 1 BTC to user_a".to_owned()).expect("add error");

        thread::sleep(Duration::from_secs(1));
        chain.add_block("Send 2 BTC to user_b".to_owned()).expect("add error");

        for (i, block) in chain.blocks.iter().enumerate() {
            println!("block-{}: {:?}", i, block);
            if i == 0 {
                assert_eq!(block.pre_hash, "");
            } else {
                assert_eq!(block.pre_hash, chain.blocks[i - 1].hash);
            }
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

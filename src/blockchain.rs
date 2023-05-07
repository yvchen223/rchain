use crate::block::Block;

/// The actual Blockchain container
pub struct Blockchain {
    /// Stores all the blocks which are accepted already within the blockchain
    pub blocks: Vec<Block>,
}

impl Blockchain {
    /// New a genesis Blockchain
    pub fn new() -> Self {
        Blockchain {
            blocks: vec![Block::new_genesis()],
        }
    }

    /// Will add a block to the Blockchain
    pub fn add_block(&mut self, data: String) {
        let pre_hash = self.get_last_hash();
        let block = Block::new(data, pre_hash);
        self.blocks.push(block);
    }

    fn get_last_hash(&self) -> String {
        self.blocks[self.blocks.len() - 1].hash.clone()
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
        chain.add_block("Send 1 BTC to user_a".to_owned());

        thread::sleep(Duration::from_secs(1));
        chain.add_block("Send 2 BTC to user_b".to_owned());

        for (i, block) in chain.blocks.iter().enumerate() {
            println!("block-{}: {:?}", i, block);
            if i == 0 {
                assert_eq!(block.pre_hash, "");
            } else {
                assert_eq!(block.pre_hash, chain.blocks[i - 1].hash);
            }
        }
    }
}

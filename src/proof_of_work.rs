use crate::block::Block;
use crate::common::{append_str, hash_utf8, hex_to_big_int};
use log::debug;
use num::{BigInt, Num};
use sha2::{Digest, Sha256};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::ops::ShlAssign;

/// This is an arbitrary number that takes less than 256 bits in memory.
/// Here we won't implement a target adjusting algorithm.
/// For now, so we can just define the difficulty as a global constant.
const TARGET_BITS: i32 = 8;

const MAX_NONCE: u64 = u64::MAX;

/// Proof of work
pub struct ProofOfWork {
    block: Block,

    /// Use a big integer because of the way we'll compare a hash to the target:
    /// we'll convert a hash to a big integer and check if it's less than the target.
    target: BigInt,
}

impl ProofOfWork {
    pub fn new(block: Block) -> Self {
        let mut target = BigInt::from(1);
        target.shl_assign(256 - TARGET_BITS);

        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", target).as_bytes());

        ProofOfWork { block, target }
    }

    /// Merge block fields with target and nonce.
    pub fn prepare_data(&self, nonce: u64) -> Vec<u8> {
        let mut data = vec![];

        append_str(&mut data, self.block.pre_hash.as_str());
        append_str(&mut data, self.block.data.as_str());
        append_str(&mut data, format!("{:x}", self.block.timestamp).as_str());
        append_str(&mut data, format!("{:x}", TARGET_BITS).as_str());
        append_str(&mut data, format!("{:x}", nonce).as_str());

        data
    }

    /// Get the nonce which is for the requirement and hash.
    pub fn run(&self) -> (u64, String) {
        let mut nonce: u64 = 0;
        let mut hash_res = String::new();
        debug!("Mining the block containing {}", self.block.data);

        // limited by MAX_NONCE due to avoid a possible overflow of nonce.
        while nonce < MAX_NONCE {
            let prepare_data = self.prepare_data(nonce);

            // hash the prepare data with SHA-256.
            let hash = hash_utf8(prepare_data.as_slice());

            // convert the hash(hex string) to big int.
            let hash_int = hex_to_big_int(&hash);

            // compare the integer with the target.
            // the requirement sounds like "first few bits of a hash must be zeros",
            // and the number of zero bits depends on TARGET_BITS which is also the difficulty of mining.
            if hash_int.lt(self.target.borrow()) {
                debug!("hash {:?}", hash);
                hash_res = hash;
                break;
            }
            nonce += 1;
        }

        (nonce, hash_res)
    }

    /// Validate proof of works.
    pub fn validate(&self) -> bool {
        let data = self.prepare_data(self.block.nonce);
        let hash = hash_utf8(data.as_slice());
        let hash_int = hex_to_big_int(&hash);

        hash_int.cmp(&self.target) == Ordering::Less
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::Blockchain;
    use data_encoding::HEXLOWER;
    use num::Num;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_target() {
        let mut target = BigInt::from(1);
        target.shl_assign(256 - TARGET_BITS);
        println!("target int {}", target);

        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", target).as_bytes());
        println!("target hash {}", format!("{:x}", hasher.finalize()));

        let (_, vec) = target.to_bytes_be();
        let target_hex = HEXLOWER.encode(&*vec);
        println!("target hex {}", target_hex);
    }

    #[test]
    fn test_validate() {
        env_logger::init();
        let mut chain = Blockchain::new();

        thread::sleep(Duration::from_secs(1));
        chain.add_block("Send 1 BTC to user_a".to_owned());

        thread::sleep(Duration::from_secs(1));
        chain.add_block("Send 2 BTC to user_b".to_owned());

        for block in chain.blocks {
            println!("block {:?}", block);
            let pow = ProofOfWork::new(block);
            assert!(pow.validate());
        }
    }
}

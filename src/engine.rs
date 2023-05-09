use crate::Result;
use std::borrow::ToOwned;
use std::path::PathBuf;
use std::str::from_utf8;
use std::string::ToString;

/// The isolated keyspace that stores block data.
pub const BLOCK_TREE: &str = "block_tree";

/// The key that stores the last block hash of the chain.
pub const LAST_HASH_OF_CHAIN: &str = "l";

/// The database that stores persistent blockchain
#[derive(Debug, Clone)]
pub struct SledEngine {
    sled: sled::Db,
    block_tree: sled::Tree,
}

impl SledEngine {
    /// Input a path of directory and return database
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let db = sled::open(path)?;
        let block_tree = db.open_tree(BLOCK_TREE)?;
        Ok(SledEngine {
            sled: db,
            block_tree,
        })
    }

    /// Get the string value of the given key.
    ///
    /// Return `None' if the key does not exist.
    pub fn get(&self, key: impl Into<String>) -> Result<Option<String>> {
        let key = key.into();
        let val = self.block_tree.get(key)?;
        match val {
            Some(v) => {
                let str = from_utf8(&v)?;
                Ok(Some(str.to_string()))
            }
            None => Ok(None),
        }
    }

    /// Set a pair of key-value.
    ///
    /// Return the last value if it was set.
    pub fn set(&self, key: impl Into<String>, val: impl Into<String>) -> Result<Option<String>> {
        let key = key.into();
        let val = val.into();
        let last_value = self.block_tree.insert(key, val.into_bytes())?;
        match last_value {
            Some(v) => {
                let val_str = std::str::from_utf8(&v)?;
                Ok(Some(val_str.to_owned()))
            }
            None => Ok(None),
        }
    }
}

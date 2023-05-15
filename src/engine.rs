use crate::Result;
use std::borrow::ToOwned;
use std::env::current_dir;
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
    tree: sled::Tree,
}

impl SledEngine {
    /// New a default SledEngine in current directory.
    pub fn new(tree_name: &str) -> Result<Self> {
        let db = sled::open(current_dir().unwrap())?;
        let tree = db.open_tree(tree_name)?;
        Ok(SledEngine { tree })
    }

    /// New a SledEngine with sled db.
    pub fn with_db(tree_name: &str, db: &sled::Db) -> Result<Self> {
        let tree = db.open_tree(tree_name)?;
        Ok(SledEngine { tree })
    }

    /// New a SledEngine with a path.
    pub fn with_path(path: impl Into<PathBuf>, tree_name: &str) -> Result<Self> {
        let path = path.into();
        let db = sled::open(path)?;
        let tree = db.open_tree(tree_name)?;
        Ok(SledEngine { tree })
    }

    /// Get the string value of the given key.
    ///
    /// Return `None' if the key does not exist.
    pub fn get(&self, key: impl Into<String>) -> Result<Option<String>> {
        let key = key.into();
        let val = self.tree.get(key)?;
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
        let last_value = self.tree.insert(key, val.into_bytes())?;
        match last_value {
            Some(v) => {
                let val_str = std::str::from_utf8(&v)?;
                Ok(Some(val_str.to_owned()))
            }
            None => Ok(None),
        }
    }

    /// List all wallets that have benn stored.
    pub fn list(&self) -> Vec<(String, String)> {
        let mut vec = vec![];
        let iter = self.tree.iter();
        for v in iter {
            match v {
                Ok((k, v)) => {
                    let key = from_utf8(&k).unwrap();
                    let val = from_utf8(&v).unwrap();
                    vec.push((key.to_owned(), val.to_owned()));
                }
                Err(_) => continue,
            }
        }

        vec
    }
}

//! Wallet.

use crate::common::{base58_encode, ripemd160_digest, sha256_digest};
use crate::engine::{SledEngine, WALLETS_TREE};
use crate::Result;
use p256::pkcs8::EncodePrivateKey;
use p256::SecretKey;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Version for generate an address.
const VERSION: u8 = 0x00;
/// The len of checksum.
const ADDRESS_CHECKSUM_LEN: usize = 4;

/// Basic wallet.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Wallet {
    private_key: String,
    public_key: String,
}

impl Wallet {
    /// New a wallet.
    pub fn new() -> Self {
        let (private_key, public_key) = Self::new_key_pair();
        Wallet {
            private_key,
            public_key,
        }
    }

    /// New an empty wallet.
    pub fn empty() -> Self {
        Wallet {
            private_key: String::from("empty"),
            public_key: String::from("empty"),
        }
    }

    /// New a public key and a private key.
    fn new_key_pair() -> (String, String) {
        let private_key = SecretKey::random(&mut OsRng);
        let public_key = private_key.public_key();
        let private_key = private_key
            .to_pkcs8_pem(Default::default())
            .unwrap()
            .to_string();
        let public_key = public_key.to_string();

        (private_key, public_key)
    }

    /// Return private key.
    pub fn private_key(&self) -> String {
        self.private_key.clone()
    }

    /// Return public key.
    pub fn public_key(&self) -> String {
        self.public_key.clone()
    }

    /// Calculate an address that is a real Bitcoin address.
    /// We can even check its balance on https://blockchain.info/.
    pub fn address(&self) -> String {
        let pub_key_hash = Self::hash_pub_key(self.public_key.as_bytes());

        // Check sum.
        let mut versioned_payload = vec![VERSION];
        versioned_payload.extend(pub_key_hash.as_slice());
        let checksum = Self::checksum(&versioned_payload);

        // `Version + pub_key_hash + checksum` and encode it with Base58
        let mut full_payload = vec![VERSION];
        full_payload.extend(pub_key_hash.as_slice());
        full_payload.extend(checksum.as_slice());
        base58_encode(&full_payload)
    }

    /// Get the public key hash.
    pub fn pub_key_hash(&self) -> Vec<u8> {
        let pub_key_sha256 = sha256_digest(self.public_key().as_bytes());
        ripemd160_digest(&pub_key_sha256)
    }

    /// Take the public key and hash it twice with `RIPEMD160(SHA256(public_key))`.
    pub(crate) fn hash_pub_key(public_key: &[u8]) -> Vec<u8> {
        let pub_key_sha256 = sha256_digest(public_key);
        ripemd160_digest(&pub_key_sha256)
    }

    /// Calculate the checksum by hashing the payload with `SHA256(SHA256(version + pub_key_hash))`.
    fn checksum(payload: &[u8]) -> Vec<u8> {
        let first_sha = sha256_digest(payload);
        let second_sha = sha256_digest(first_sha.as_slice());
        second_sha[0..ADDRESS_CHECKSUM_LEN].to_vec()
    }

    /// Serialize a block to String.
    pub fn serialize(&self) -> Result<String> {
        let serialization = ron::to_string(&self)?;
        Ok(serialization)
    }

    /// deserialize str to a block.
    pub fn deserialize(value: &str) -> Result<Self> {
        let wallet: Wallet = ron::from_str(value).map_err(|e| e.code)?;
        Ok(wallet)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

/// Wallets.
pub struct Wallets {
    wallets: SledEngine,
}

impl Wallets {
    /// New wallets with sled db.
    pub fn new(db: &sled::Db) -> Self {
        let wallets = SledEngine::new(WALLETS_TREE, db).unwrap();
        Wallets { wallets }
    }

    /// New wallets with path.
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let db = sled::open(path).unwrap();
        let wallets = SledEngine::new(WALLETS_TREE, &db).unwrap();
        Wallets { wallets }
    }

    /// Set wallet.
    pub fn set(&self, wallet: &Wallet) -> Result<()> {
        let address = wallet.address();
        let serialized = wallet.serialize()?;
        self.wallets.set(address, serialized)?;
        Ok(())
    }

    /// Set wallet with an address.
    pub fn set_with_address(&self, address: &str, wallet: &Wallet) -> Result<()> {
        let serialized = wallet.serialize()?;
        self.wallets.set(address, serialized)?;
        Ok(())
    }

    /// Get wallet.
    pub fn get(&self, address: &str) -> Result<Option<Wallet>> {
        let wallet = match self.wallets.get(address)? {
            Some(v) => Wallet::deserialize(&v)?,
            None => return Ok(None),
        };
        Ok(Some(wallet))
    }

    /// List wallets.
    pub fn list(&self) -> Vec<(String, Wallet)> {
        let mut v = vec![];
        let vec = self.wallets.list();
        for (key, val) in vec {
            let wallet = Wallet::deserialize(&val).unwrap();
            v.push((key, wallet));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use crate::wallet::Wallet;
    use p256::ecdsa::signature::{Signer, Verifier};
    use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
    use p256::pkcs8::EncodePrivateKey;
    use p256::PublicKey;
    use p256::SecretKey;
    use rand_core::OsRng;

    #[test]
    fn test_wallet() {
        let wallet = Wallet::new();
        println!("{:?}", wallet);

        let address = wallet.address();
        println!("address: {}", address);
    }

    #[test]
    fn test_key() {
        let secret_key = SecretKey::random(&mut OsRng);
        let serialized = secret_key
            .to_pkcs8_pem(Default::default())
            .unwrap()
            .to_string();
        println!("key: \n{}", serialized);

        let public_key = secret_key.public_key();
        println!("public: {}", public_key.to_string());

        let secret_key = serialized.parse::<SecretKey>().unwrap();
        let signing_key: SigningKey = secret_key.into();

        let msg = b"message";
        let signature: Signature = signing_key.sign(msg);

        let verifying_key: VerifyingKey = public_key.into();
        assert!(verifying_key.verify(msg, &signature).is_ok())
    }
}

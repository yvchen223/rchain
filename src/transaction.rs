use crate::common::{base58_decode, hash_str};
use crate::error::Error;
use crate::error::Error::{InvalidTransaction, NoEnoughBalance, StringError};
use crate::wallet::Wallet;
use crate::Blockchain;
use crate::Result;
use p256::ecdsa::signature::{Signer, Verifier};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::{from_utf8, FromStr};

/// The subsidy of mining a block.
const SUBSIDY: i64 = 10;

/// The output in a transaction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TXOutput {
    /// Stores the number of coins.
    pub value: i64,

    /// The public key hash in output.
    pub_key_hash: Vec<u8>,
}

impl TXOutput {
    pub fn new(value: i64, address: &str) -> Result<Self> {
        let pub_key_hash = Self::get_pub_key_hash(address)?;
        let tx_out = TXOutput {
            value,
            pub_key_hash,
        };
        Ok(tx_out)
    }

    fn get_pub_key_hash(address: &str) -> Result<Vec<u8>> {
        let full_hash = base58_decode(address)?;
        if full_hash.len() < 4 {
            return Err(Error::StringError(
                "lock with an invalid address.".to_owned(),
            ));
        }
        // Take out the version and the checksum.
        let pub_key_hash = full_hash[1..full_hash.len() - 4].to_vec();
        Ok(pub_key_hash)
    }

    pub fn lock(&mut self, address: &str) -> Result<()> {
        let pub_key_hash = Self::get_pub_key_hash(address)?;
        self.pub_key_hash = pub_key_hash;
        Ok(())
    }

    /// Check if provided public key hash was used to lock the output.
    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash.eq(&pub_key_hash)
    }
}

/// The input in a transaction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TXInput {
    /// The id of a previous transaction that input references.
    pub tx_id: String,

    /// The index of output in the transaction referenced by `tx_id`.
    pub idx_vout: usize,

    /// A script which provides data to be used in an output's `script_pub_key`.
    ///
    /// If the data is correct,the output can be unlocked and it's value can be used to
    /// generate new outputs;if it's not correct,the output cannot be referenced in the input.
    ///
    /// This is the mechanism that guarantees that users cannot spend coins belonging to other people.
    ///
    /// Now it is just an arbitrary string.
    // pub script_sig: String,
    signature: Option<Signature>,
    /// Raw public key.
    pub public_key: Vec<u8>,
}

impl TXInput {
    /// Check if someone with the public key hash can use the input.
    pub fn use_key(&self, pub_key_hash: &[u8]) -> bool {
        let locking_hash = Wallet::hash_pub_key(&self.public_key);
        locking_hash.eq(pub_key_hash)
    }
}

/// Transaction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    /// Transaction id.
    pub id: String,

    /// Inputs of a new transaction reference outputs of previous transaction.
    pub vin: Vec<TXInput>,

    /// Outputs are where coins are actually stored.
    pub vout: Vec<TXOutput>,
}

impl Transaction {
    /// New a coinbase transaction.
    pub fn new_coinbase_tx(to: String, data: String) -> Self {
        let data = if data.is_empty() {
            format!("Reward to {}", to)
        } else {
            data
        };
        // A coinbase transaction has only one input.In this implementation its `tx_id` is empty
        // and `idx_vout` equals to -1.Also, it doesn't store a script in `script_sig`.
        // Instead, arbitrary data is stored there.
        let tx_in = TXInput {
            tx_id: String::new(),
            idx_vout: 0,
            signature: None,
            public_key: data.into_bytes(),
        };
        let tx_out = TXOutput::new(SUBSIDY, &to).unwrap();
        println!("out: {:?}", tx_out);

        let mut tx = Transaction {
            id: String::new(),
            vin: vec![tx_in],
            vout: vec![tx_out],
        };
        tx.set_id();

        tx
    }

    /// New a normal transaction.
    pub fn new(from: &str, to: &str, amount: i64, blockchain: &Blockchain) -> Result<Self> {
        let from_wallet = match blockchain.get_wallet(from)? {
            Some(v) => v,
            None => return Err(StringError(format!("no such address: {}", from))),
        };

        let _to_wallet = match blockchain.get_wallet(to)? {
            Some(v) => v,
            None => return Err(StringError(format!("no such address: {}", from))),
        };
        // Find all unspent outputs and ensure that they store enough value.
        let (acc, outputs_idx) = blockchain.find_spendable_outputs(from, amount);
        if acc < amount {
            return Err(NoEnoughBalance);
        }

        let mut inputs = vec![];
        let mut outputs = vec![];

        // Build a list of inputs
        for (tx_id, output_idx) in outputs_idx {
            for idx in output_idx {
                let input = TXInput {
                    tx_id: tx_id.clone(),
                    idx_vout: idx,
                    public_key: Vec::from(from_wallet.public_key()),
                    signature: None,
                };
                // println!("input: {:?}", input);
                inputs.push(input);
            }
        }

        // Build a list of outputs
        // Locked with the receiver address.This is the actual transferring of coins to other address.
        let output = TXOutput::new(amount, to).unwrap();
        outputs.push(output);
        if acc > amount {
            // Locked with the sender address.This is a change.
            let output = TXOutput::new(acc - amount, from).unwrap();
            outputs.push(output);
        }

        let mut tx = Transaction {
            id: String::new(),
            vin: inputs,
            vout: outputs,
        };

        tx.set_id();

        // Here, we sign the transaction to guarantee
        // that one cannot spend coins belonging to someone else.
        blockchain.sign_transaction(&mut tx, &from_wallet.private_key());

        Ok(tx)
    }

    fn set_id(&mut self) {
        self.id = self.hash();
    }

    fn hash(&self) -> String {
        let str = self.serialize().unwrap();
        hash_str(str)
    }

    /// Serialize a transaction to String.
    fn serialize(&self) -> Result<String> {
        let serialization = ron::to_string(&self)?;
        Ok(serialization)
    }

    /// Is a coinbase transaction.
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].tx_id.is_empty()
    }

    /// Choose data to identify the tx in an unique way.
    ///
    /// Public key hashes stored in unlocked outputs. This identifies "sender" of a tx.
    ///
    /// Public key hashes stored in new, locked outputs. This identifies "recipient" of a tx.
    ///
    /// We store the signature of the referenced output to the `signature` field in vin.
    pub fn sign(&mut self, private_key: &str, prev_txs: HashMap<String, Transaction>) {
        // Nothing to do if a coinbase tx.
        if self.is_coinbase() {
            return;
        }

        // A trimmed clone will be signed, not a full transaction.
        let mut tx = self.trimmed_clone();

        for i in 0..tx.vin.len() {
            let vin = tx.vin.get_mut(i).unwrap();
            let prev_tx = match prev_txs.get(&vin.tx_id) {
                Some(v) => v,
                None => continue,
            };
            vin.signature = None;
            // All the inputs but the current one are empty.
            vin.public_key = prev_tx.vout[vin.idx_vout].pub_key_hash.clone();
            let hash = tx.hash();
            // Set empty for the next input.
            tx.vin[i].public_key = vec![];

            let secret_key = private_key.parse::<SecretKey>().unwrap();
            let signing_key: SigningKey = secret_key.into();
            let signature = signing_key.sign(hash.as_bytes());
            self.vin[i].signature = Some(signature);
        }
    }

    /// Verify the tx.
    ///
    /// We get the hash of previous transactions. The operation is just like `sign`.
    ///
    /// And compare it to the `signature` field in vin.
    pub fn verify(&self, prev_txs: HashMap<String, Transaction>) -> Result<bool> {
        if self.is_coinbase() {
            return Ok(true);
        }
        let mut tx = self.trimmed_clone();
        for i in 0..self.vin.len() {
            let prev_tx = match prev_txs.get(&self.vin[i].tx_id) {
                Some(v) => v,
                None => continue,
            };
            tx.vin[i].signature = None;
            tx.vin[i].public_key = prev_tx.vout[self.vin[i].idx_vout].pub_key_hash.clone();

            let hash = tx.hash();
            tx.vin[i].public_key = vec![];

            let signature = match self.vin[i].signature.as_ref() {
                Some(v) => v,
                None => {
                    return Err(InvalidTransaction(format!(
                        "tx_id-{} has no signature",
                        self.id
                    )))
                }
            };
            let public_key = self.vin[i].public_key.clone();
            let str = from_utf8(&public_key)?;
            let public_key = PublicKey::from_str(str).unwrap();
            let verifying_key: VerifyingKey = public_key.into();
            if verifying_key.verify(hash.as_bytes(), signature).is_err() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Include all the inputs and outputs, but `TXInput.signature` and `TXInput.public_key`.
    fn trimmed_clone(&self) -> Self {
        let mut inputs = vec![];
        let mut outputs = vec![];
        for vin in &self.vin {
            inputs.push(TXInput {
                tx_id: vin.tx_id.clone(),
                idx_vout: vin.idx_vout,
                signature: None,
                public_key: vec![],
            });
        }
        for vout in &self.vout {
            outputs.push(TXOutput {
                value: vout.value,
                pub_key_hash: vout.pub_key_hash.clone(),
            })
        }

        Transaction {
            id: String::new(),
            vin: inputs,
            vout: outputs,
        }
    }

    /// .
    pub fn pub_key_hash_from_address(address: &str) -> Vec<u8> {
        let full_hash = match base58_decode(address) {
            Ok(v) => v,
            Err(_) => return vec![],
        };
        if full_hash.len() < 4 {
            return vec![];
        }
        // Take out the version and the checksum.
        full_hash[1..full_hash.len() - 4].to_vec()
    }
}

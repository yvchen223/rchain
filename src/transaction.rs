use crate::error::Error::NoEnoughBalance;
use crate::Blockchain;
use crate::Result;
use serde::{Deserialize, Serialize};
use crate::common::{base58_decode, hash_str};
use crate::error::Error;
use crate::wallet::Wallet;

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
            return Err(Error::StringError("lock with an invalid address.".to_owned()));
        }
        /// Take out the version and the checksum.
        let pub_key_hash = full_hash[1..full_hash.len() - 4].to_vec();
        Ok(pub_key_hash)
    }

    pub fn lock(&mut self, address: &str) -> Result<()> {
        let pub_key_hash = Self::get_pub_key_hash(address)?;
        self.pub_key_hash = pub_key_hash;
        Ok(())
    }

    /// Check if provided public key hash was used to lock the output.
    pub fn is_locked_with_key(&self, address: &str) -> bool {
        let full_hash = match base58_decode(address) {
            Ok(v) => v,
            Err(_) => return false,
        };
        if full_hash.len() < 4 {
            return false;
        }
        /// Take out the version and the checksum.
        let pub_key_hash = full_hash[1..full_hash.len() - 4].to_vec();
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

    signature: String,
    /// Raw public key.
    pub public_key: String,
}

impl TXInput {

    /// Check if someone with the public key hash can use the input.
    pub fn use_key(&self, pub_key_hash: &str) -> bool {
        //let locking_hash = Wallet::hash_pub_key(&self.public_key);
        self.public_key.eq(pub_key_hash)
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
            signature: String::new(),
            public_key: data.clone(),
        };
        let tx_out = TXOutput::new(SUBSIDY, &to).unwrap();

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
                    public_key: from.to_owned(),
                    signature: String::new(),
                };
                inputs.push(input);
            }
        }

        // Build a list of outputs
        // Locked with the receiver address.This is the actual transferring of coins to other address.
        let mut output = TXOutput {
            value: amount,
            pub_key_hash: vec![],
        };
        output.lock(to)?;
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
}

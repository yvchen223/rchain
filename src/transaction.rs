use crate::error::Error::NoEnoughBalance;
use crate::Blockchain;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// The subsidy of mining a block.
const SUBSIDY: i64 = 10;

/// The output in a transaction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TXOutput {
    /// Stores the number of coins.
    pub value: i64,

    /// Stores a puzzle that locks the value.
    ///
    /// Now it is an arbitrary string.
    pub script_pub_key: String,
}

impl TXOutput {
    /// Check if the output can be unlocked with the data and spend it.
    pub fn can_be_unlocked_with(&self, unlocking_data: String) -> bool {
        self.script_pub_key == unlocking_data
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
    pub script_sig: String,
}

impl TXInput {
    /// Check if it can unlocked the output with the data.
    pub fn can_unlock_output_with(&self, unlocking_data: String) -> bool {
        self.script_sig == unlocking_data
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
            script_sig: data,
        };
        let tx_out = TXOutput {
            value: SUBSIDY,
            script_pub_key: to,
        };

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
                    script_sig: from.to_owned(),
                };
                inputs.push(input);
            }
        }

        // Build a list of outputs
        // Locked with the receiver address.This is the actual transferring of coins to other address.
        outputs.push(TXOutput {
            value: amount,
            script_pub_key: to.to_owned(),
        });
        if acc > amount {
            // Locked with the sender address.This is a change.
            outputs.push(TXOutput {
                value: acc - amount,
                script_pub_key: from.to_owned(),
            });
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
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        self.id = timestamp.to_string();
    }

    /// Is a coinbase transaction.
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].tx_id.is_empty()
    }
}

use serde::{Deserialize, Serialize};
use crate::Block;

const SUBSIDY: i64 = 10;

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
    pub fn can_be_unlocked_with(&self, unlocking_data: String) -> bool {
        self.script_pub_key == unlocking_data
    }
}

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
    pub fn can_unlock_output_with(&self, unlocking_data: String) -> bool {
        self.script_sig == unlocking_data
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    /// id
    pub id: String,

    /// Inputs of a new transaction reference outputs of previous transaction.
    pub vin: Vec<TXInput>,

    /// Outputs are where coins are actually stored.
    pub vout: Vec<TXOutput>,
}

impl Transaction {
    pub fn new_coinbase_tx(to: String, data: String) -> Self {
        let data = if data.len() == 0{
            format!("Reward to {}", to)
        } else {
            data
        };
        // A coinbase transaction has only one input.In this implementation its `tx_id` is empty
        // and `idx_vout` equals to -1.Also, it doesn't store a script in `script_sig`.
        // Instead, arbitrary data is stored there.
        let tx_in = TXInput { tx_id: String::new(), idx_vout: 0, script_sig: data };
        let tx_out = TXOutput { value: SUBSIDY, script_pub_key: to };

        let mut tx = Transaction {
            id: String::new(),
            vin: vec![tx_in],
            vout: vec![tx_out],
        };
        tx.set_id();

        tx
    }

     fn set_id(&mut self) {

     }

    pub fn is_coinbase(&self) -> bool {
        self.id.len() == 0
    }
}
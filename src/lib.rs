#![deny(missing_docs)]

//! A blockchain building in Rust

pub use block::Block;
pub use blockchain::Blockchain;
pub use error::Result;
pub use proof_of_work::ProofOfWork;
pub use transaction::Transaction;

mod block;
mod blockchain;
mod common;
mod engine;
mod error;
mod proof_of_work;
mod transaction;
pub mod wallet;

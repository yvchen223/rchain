#![deny(missing_docs)]

//! A blockchain building in Rust

pub use error::Result;

mod block;
mod blockchain;
mod common;
mod engine;
mod error;
mod proof_of_work;

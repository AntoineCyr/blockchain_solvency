pub type Result<T> = std::result::Result<T, failure::Error>;
use merkle_sum_tree::{Leaf, MerkleSumTree};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::proofs::ProofOfLiabilities;

//work on hash and nonce
//Start from root sum 0, every change -> compile a proof
// For each change, create input object =>
// Max number of user is 8, can change that if we compile the circuit for a bigger tree
#[derive(Debug, Clone)]
pub struct Block {
    block_number: i32,
    transactions: Vec<Transaction>,
    prev_block_hash: i32,
    hash: i32,
    nonce: i32,
    leaf_index: HashMap<String, usize>,
    merkle_sum_tree: MerkleSumTree,
    liabilities_verified: bool,
    liabilities_proof: Option<ProofOfLiabilities>,
    timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    from: String,
    to: String,
    amount: i32,
}

impl Block {
    pub fn get_hash(&self) -> i32 {
        self.hash.clone()
    }

    pub fn get_previous_hash(&self) -> i32 {
        self.prev_block_hash.clone()
    }

    pub fn get_merkle_sum_tree(&self) -> MerkleSumTree {
        self.merkle_sum_tree.clone()
    }

    pub fn get_timestamp(&self) -> SystemTime {
        self.timestamp.clone()
    }

    pub fn get_block_number(&self) -> i32 {
        self.block_number
    }

    pub fn new(
        block_number: i32,
        transactions: Vec<Transaction>,
        prev_block_hash: i32,
        leaf_index: HashMap<String, usize>,
        merkle_sum_tree: MerkleSumTree,
        liabilities_verified: bool,
        liabilities_proof: Option<ProofOfLiabilities>,
    ) -> Result<Block> {
        Ok(Block {
            block_number: block_number,
            transactions,
            prev_block_hash,
            nonce: 0,
            hash: prev_block_hash + 1,
            leaf_index,
            merkle_sum_tree,
            liabilities_verified,
            liabilities_proof,
            timestamp: SystemTime::now(),
        })
    }
}

impl Transaction {
    pub fn new(from: String, to: String, amount: i32) -> Transaction {
        let transaction = Transaction { from, to, amount };
        transaction
    }

    pub fn get_to(&self) -> String {
        self.to.clone()
    }

    pub fn get_from(&self) -> String {
        self.from.clone()
    }

    pub fn get_amount(&self) -> i32 {
        self.amount.clone()
    }
}

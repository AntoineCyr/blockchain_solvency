pub type Result<T> = std::result::Result<T, failure::Error>;
use merkle_sum_tree::{Leaf, MerkleSumTree};
use std::collections::HashMap;

//work on hash and nonce
//Start from root sum 0, every change -> compile a proof
// For each change, create input object =>
// Max number of user is 8, can change that if we compile the circuit for a bigger tree

#[derive(Debug, Clone)]
pub struct Block {
    transactions: Vec<Transaction>,
    prev_block_hash: i32,
    hash: i32,
    nonce: i32,
    leaf_index: HashMap<String, usize>,
    merkle_sum_tree: MerkleSumTree,
    merkle_sum_tree_verified: bool,
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

    pub fn get_merkle_sum_tree(&self) -> MerkleSumTree {
        self.merkle_sum_tree.clone()
    }

    pub fn new(
        transactions: Vec<Transaction>,
        prev_block_hash: i32,
        leaf_index: HashMap<String, usize>,
        merkle_sum_tree: MerkleSumTree,
        merkle_sum_tree_verified: bool,
    ) -> Result<Block> {
        Ok(Block {
            transactions,
            prev_block_hash,
            nonce: 0,
            hash: prev_block_hash + 1,
            leaf_index,
            merkle_sum_tree,
            merkle_sum_tree_verified,
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

pub type Result<T> = std::result::Result<T, failure::Error>;
use chrono;
use merkle_sum_tree::MerkleSumTree;
use std::collections::HashMap;
use std::sync::Arc;

//TODO
//Have a real hash for the block, random nonce

#[derive(Debug)]
pub struct Block {
    block_number: i32,
    transactions: Vec<Transaction>,
    prev_block_hash: i32,
    hash: i32,
    nonce: i32,
    leaf_index: HashMap<String, usize>,
    merkle_sum_tree: Arc<MerkleSumTree>,
    timestamp: String,
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Block {
            block_number: self.block_number,
            transactions: self.transactions.clone(),
            prev_block_hash: self.prev_block_hash,
            hash: self.hash,
            nonce: self.nonce,
            leaf_index: self.leaf_index.clone(),
            merkle_sum_tree: Arc::clone(&self.merkle_sum_tree),
            timestamp: self.timestamp.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    from: String,
    to: String,
    amount: i32,
}

impl Block {
    pub fn get_hash(&self) -> i32 {
        self.hash
    }

    pub fn get_previous_hash(&self) -> i32 {
        self.prev_block_hash
    }

    pub fn get_merkle_sum_tree(&self) -> &MerkleSumTree {
        &self.merkle_sum_tree
    }

    pub fn get_merkle_sum_tree_arc(&self) -> Arc<MerkleSumTree> {
        Arc::clone(&self.merkle_sum_tree)
    }

    pub fn get_block_number(&self) -> i32 {
        self.block_number
    }

    pub fn get_timestamp(&self) -> &str {
        &self.timestamp
    }

    pub fn new(
        block_number: i32,
        transactions: Vec<Transaction>,
        prev_block_hash: i32,
        leaf_index: HashMap<String, usize>,
        merkle_sum_tree: Arc<MerkleSumTree>,
    ) -> Result<Block> {
        let _ = chrono::offset::Utc::now();
        Ok(Block {
            block_number: block_number,
            transactions,
            prev_block_hash,
            nonce: 0,
            hash: prev_block_hash + 1,
            leaf_index,
            merkle_sum_tree,
            timestamp: format!("{:?}", chrono::offset::Utc::now()),
        })
    }
}

impl Transaction {
    pub fn new(from: String, to: String, amount: i32) -> Transaction {
        let transaction = Transaction { from, to, amount };
        transaction
    }

    pub fn get_to(&self) -> &str {
        &self.to
    }

    pub fn get_from(&self) -> &str {
        &self.from
    }

    pub fn get_amount(&self) -> i32 {
        self.amount
    }
}

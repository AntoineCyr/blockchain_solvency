pub type Result<T> = std::result::Result<T, failure::Error>;
use chrono;
use merkle_sum_tree::MerkleSumTree;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct Block {
    block_number: i32,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    leaf_index: HashMap<String, usize>,
    merkle_sum_tree: Arc<MerkleSumTree>,
    timestamp: String,
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Block {
            block_number: self.block_number,
            transactions: self.transactions.clone(),
            prev_block_hash: self.prev_block_hash.clone(),
            hash: self.hash.clone(),
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
    pub fn get_hash(&self) -> &str {
        &self.hash
    }

    pub fn get_previous_hash(&self) -> &str {
        &self.prev_block_hash
    }

    pub fn get_merkle_sum_tree(&self) -> &MerkleSumTree {
        &self.merkle_sum_tree
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
        prev_block_hash: &str,
        leaf_index: HashMap<String, usize>,
        merkle_sum_tree: Arc<MerkleSumTree>,
    ) -> Result<Block> {
        let prev_hash_string = prev_block_hash.to_string();

        let timestamp = format!("{:?}", chrono::offset::Utc::now());

        // Calculate proper block hash
        let hash = Self::calculate_hash(
            block_number,
            &transactions,
            &prev_hash_string,
            &timestamp,
            &merkle_sum_tree,
        );

        Ok(Block {
            block_number,
            transactions,
            prev_block_hash: prev_hash_string,
            hash,
            leaf_index,
            merkle_sum_tree,
            timestamp,
        })
    }

    fn calculate_hash(
        block_number: i32,
        transactions: &[Transaction],
        prev_block_hash: &str,
        timestamp: &str,
        merkle_sum_tree: &Arc<MerkleSumTree>,
    ) -> String {
        let mut hasher = Sha256::new();

        hasher.update(block_number.to_be_bytes());
        hasher.update(prev_block_hash.as_bytes());
        hasher.update(timestamp.as_bytes());

        for tx in transactions {
            hasher.update(tx.from.as_bytes());
            hasher.update(tx.to.as_bytes());
            hasher.update(tx.amount.to_be_bytes());
        }

        if let Ok(root_hash) = merkle_sum_tree.get_root_hash() {
            hasher.update(root_hash.to_string().as_bytes());
        }

        if let Ok(root_sum) = merkle_sum_tree.get_root_sum() {
            hasher.update(root_sum.to_be_bytes());
        }

        format!("{:x}", hasher.finalize())
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

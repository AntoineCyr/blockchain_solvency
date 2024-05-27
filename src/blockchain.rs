#![allow(clippy::expect_used, clippy::unwrap_used)]

use ff::Field;

use pasta_curves::vesta::Base as Fr;
use std::env::current_dir;

use crate::block::Block;
use crate::block::Transaction;
use merkle_sum_tree::{Leaf, MerkleSumTree};
pub type Result<T> = std::result::Result<T, failure::Error>;
use std::collections::HashMap;

//handle creating 2 times the same id
//remove blockchainData
//Have data for each block? State, merkle tree, ...

//server should print error message

#[derive(Debug, Clone)]
pub struct Blockchain {
    current_hash: i32,
    mempool: Vec<Transaction>,
    chain: HashMap<i32, Block>,
    state: HashMap<String, i32>,
    merkle_sum_tree: MerkleSumTree,
    leaf_index: HashMap<String, usize>,
}

impl Blockchain {
    pub fn get_balance(&self, address: String) -> i32 {
        match self.state.get(&address) {
            Some(&number) => number,
            _ => -1,
        }
    }

    pub fn get_merkle_sum_tree(&self) -> MerkleSumTree {
        self.merkle_sum_tree.clone()
    }

    pub fn create_blockchain() -> Result<Blockchain> {
        let mut chain = HashMap::new();
        let mempool = Vec::new();
        let state = HashMap::new();
        let leaf_index = HashMap::new();
        let merkle_sum_tree = MerkleSumTree::new(vec![]).unwrap();
        let block = Block::new_block(
            mempool.clone(),
            0,
            leaf_index.clone(),
            merkle_sum_tree.clone(),
        )?;
        let block_hash = block.get_hash();
        chain.insert(block_hash.clone(), block);
        let bc: Blockchain = Blockchain {
            current_hash: block_hash,
            mempool,
            chain,
            state,
            merkle_sum_tree,
            leaf_index,
        };

        Ok(bc)
    }

    pub fn add_block(&mut self) -> Result<()> {
        self.update_blockchain_data(self.mempool.clone());

        let block = Block::new_block(
            self.mempool.clone(),
            self.current_hash,
            self.leaf_index.clone(),
            self.get_merkle_sum_tree(),
        )?;
        println!(
            "new block, number of transactions confirmed: {}",
            self.mempool.len()
        );
        self.mempool.clear();
        self.current_hash = block.get_hash();
        self.chain.insert(block.get_hash(), block);

        Ok(())
    }

    fn update_blockchain_data(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        for transaction in transactions {
            let from: String = transaction.get_from();
            let to: String = transaction.get_to();
            let amount: i32 = transaction.get_amount();

            let number_from = match self.state.get(&from) {
                Some(&number) => number,
                _ => 0,
            };
            let number_to = match self.state.get(&to) {
                Some(&number) => number,
                _ => 0,
            };
            if from != "" {
                self.update_state(from, number_from - amount)?;
            }
            self.update_state(to, number_to + amount)?;
        }
        Ok(())
    }

    fn update_state(&mut self, address: String, amount: i32) -> Result<()> {
        self.state.insert(address.clone(), amount.clone());
        let index = self.leaf_index.get(&address);

        let leaf = Leaf::new(address.clone(), amount);
        if index.is_some() {
            _ = self.merkle_sum_tree.set_leaf(leaf, *index.unwrap());
        } else {
            let index = self.merkle_sum_tree.push(leaf).unwrap();
            self.leaf_index.insert(address, index);
        }
        Ok(())
    }

    pub fn add_transaction(&mut self, from: String, to: String, amount: i32) -> Result<()> {
        let transaction = Transaction::new(from, to, amount);
        self.mempool.push(transaction);

        Ok(())
    }
}

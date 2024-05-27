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
}

impl Blockchain {
    pub fn get_balance(&self, address: String) -> i32 {
        let state = self.chain.get(&self.current_hash).unwrap().get_state();
        return match state.get(&address) {
            Some(&number) => number,
            _ => -1,
        };
    }

    pub fn get_merkle_sum_tree(&self) -> MerkleSumTree {
        self.chain
            .get(&self.current_hash)
            .unwrap()
            .get_merkle_sum_tree()
    }

    pub fn create_blockchain() -> Result<Blockchain> {
        let mut chain = HashMap::new();
        let mempool = Vec::new();
        let block = Block::new_block(mempool.clone(), None)?;
        let block_hash = block.get_hash();
        chain.insert(block_hash.clone(), block);
        let bc: Blockchain = Blockchain {
            current_hash: block_hash,
            mempool,
            chain,
        };

        Ok(bc)
    }

    pub fn add_block(&mut self) -> Result<()> {
        let prev_block = self.chain.get(&self.current_hash).unwrap().clone();
        let block = Block::new_block(self.mempool.clone(), Some(prev_block))?;

        println!(
            "new block, number of transactions confirmed: {}",
            self.mempool.len()
        );
        self.mempool.clear();
        self.current_hash = block.get_hash();
        self.chain.insert(block.get_hash(), block);

        Ok(())
    }

    pub fn add_transaction(&mut self, from: String, to: String, amount: i32) -> Result<()> {
        let transaction = Transaction::new(from, to, amount);
        self.mempool.push(transaction);

        Ok(())
    }
}

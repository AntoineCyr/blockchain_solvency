#![allow(clippy::expect_used, clippy::unwrap_used)]

use ff::Field;

use core::time;
use merkle_sum_tree::InclusionProof;
use pasta_curves::vesta::Base as Fr;
use std::env::current_dir;

use crate::block::Block;
use crate::block::Transaction;
use crate::proofs::InclusionInput;
use crate::proofs::{
    CircuitSetup, LiabilitiesInput, MerkleSumTreeChange, ProofOfInclusion, ProofOfLiabilities,
};
use merkle_sum_tree::{Leaf, MerkleSumTree};
pub type Result<T> = std::result::Result<T, failure::Error>;
use std::collections::HashMap;
use std::time::SystemTime;

// verify initial tree

//handle creating 2 times the same id
//remove blockchainData
//Have data for each block? State, merkle tree, ...
// implement a check for max number of users

//server should print error message

const MAX_USERS: usize = 8;

//#[derive(Debug, Clone)]
pub struct Blockchain {
    current_hash: i32,
    current_block_number: i32,
    mempool: Vec<Transaction>,
    chain: HashMap<i32, Block>,
    state: HashMap<String, i32>,
    changes: Vec<MerkleSumTreeChange>,
    merkle_sum_tree: MerkleSumTree,
    liabilities_verified: bool,
    liabilities_proof: Option<ProofOfLiabilities>,
    liabilities_circuit_setup: CircuitSetup,
    inclusion_circuit_setup: CircuitSetup,
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

    pub fn get_liabilities_verified(&self) -> bool {
        self.liabilities_verified
    }

    pub fn get_liabilities_proof(&self) -> Option<ProofOfLiabilities> {
        self.liabilities_proof.clone()
    }

    pub fn get_changes(&self) -> Vec<MerkleSumTreeChange> {
        self.changes.clone()
    }

    pub fn create_blockchain() -> Result<Blockchain> {
        let mut chain = HashMap::new();
        let mempool = Vec::new();
        let state = HashMap::new();
        let leaf_index = HashMap::new();
        let leaf_0 = Leaf::new("0".to_string(), 0);
        let mut leafs = vec![];
        let changes = vec![];
        for _ in 0..MAX_USERS {
            leafs.push(leaf_0.clone());
        }
        let merkle_sum_tree = MerkleSumTree::new(leafs).unwrap();
        let current_block_number = 1;
        let block = Block::new(
            current_block_number,
            mempool.clone(),
            0,
            leaf_index.clone(),
            merkle_sum_tree.clone(),
            true,
            None,
        )?;
        let block_hash = block.get_hash();
        let liabilities_proof = None;
        chain.insert(block_hash.clone(), block);
        let liabilities_circuit_setup = CircuitSetup::new("liabilities_changes_folding");
        let inclusion_circuit_setup = CircuitSetup::new("inclusion");
        let bc: Blockchain = Blockchain {
            current_block_number,
            current_hash: block_hash,
            mempool,
            chain,
            state,
            changes,
            merkle_sum_tree,
            liabilities_proof,
            leaf_index,
            liabilities_circuit_setup,
            inclusion_circuit_setup,
            liabilities_verified: true,
        };

        Ok(bc)
    }

    pub fn add_block(&mut self) -> Result<()> {
        let _ = self.update_blockchain_data(self.mempool.clone());
        self.current_block_number += 1;
        let block = Block::new(
            self.current_block_number,
            self.mempool.clone(),
            self.current_hash,
            self.leaf_index.clone(),
            self.get_merkle_sum_tree(),
            self.get_liabilities_verified(),
            self.get_liabilities_proof(),
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
        if (transactions.len() == 0) {
            return Ok(());
        }
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
        let _ = self.proove_merkle_tree();
        Ok(())
    }

    fn update_state(&mut self, address: String, amount: i32) -> Result<()> {
        self.state.insert(address.clone(), amount.clone());
        let index_option = self.leaf_index.get(&address);
        let index: usize;
        let leaf = Leaf::new(address.clone(), amount);
        let old_merkle_tree = self.get_merkle_sum_tree();

        if index_option.is_some() {
            _ = self
                .merkle_sum_tree
                .set_leaf(leaf.clone(), *index_option.unwrap());
            index = *index_option.unwrap();
        } else {
            index = self.merkle_sum_tree.push(leaf.clone()).unwrap();
            self.leaf_index.insert(address, index);
        }
        let new_merkle_tree = self.get_merkle_sum_tree();
        let change = MerkleSumTreeChange::new(index, old_merkle_tree, new_merkle_tree.clone());
        self.merkle_sum_tree = new_merkle_tree;
        self.liabilities_verified = false;
        self.changes.push(change);
        Ok(())
    }

    fn proove_merkle_tree(&mut self) -> Result<()> {
        let changes = self.get_changes();
        let mut liabilities_inputs = vec![];
        for change in changes {
            liabilities_inputs.push(LiabilitiesInput::new(vec![change]).unwrap())
        }
        self.changes = vec![];
        let circuit_setup = &self.liabilities_circuit_setup;
        let liabilities_proof = ProofOfLiabilities::new(liabilities_inputs, circuit_setup);
        self.liabilities_proof = Some(liabilities_proof.unwrap());
        self.liabilities_verified = true;
        Ok(())
    }

    pub fn add_transaction(&mut self, from: String, to: String, amount: i32) -> Result<()> {
        let transaction = Transaction::new(from, to, amount);
        self.mempool.push(transaction);

        Ok(())
    }

    pub fn get_inclusion_proof(
        &self,
        address: String,
    ) -> (Option<ProofOfInclusion>, Option<Vec<Block>>) {
        let index_option = self.leaf_index.get(&address);
        let index: usize;
        let mut blocks = vec![];
        if index_option.is_some() {
            index = *index_option.unwrap();
        } else {
            return (None, None);
        }
        let mut current_block = self.chain.get(&self.current_hash).unwrap();
        let mut inclusion_inputs = vec![];
        let mut last_hash = "".to_string();
        loop {
            let inclusion_input =
                InclusionInput::new(current_block.get_merkle_sum_tree(), index).unwrap();
            if inclusion_input.get_root_hash() != last_hash {
                blocks.push(current_block.clone());
                inclusion_inputs.push(inclusion_input.clone());
            }
            last_hash = inclusion_input.get_root_hash();
            current_block = self.chain.get(&current_block.get_previous_hash()).unwrap();
            if current_block
                .get_merkle_sum_tree()
                .get_leaf(index)
                .unwrap()
                .get_id()
                != address
            {
                break;
            }
        }
        let proof_of_inclusion =
            ProofOfInclusion::new(inclusion_inputs, &self.inclusion_circuit_setup);

        return (Some(proof_of_inclusion.unwrap()), Some(blocks));
    }
}

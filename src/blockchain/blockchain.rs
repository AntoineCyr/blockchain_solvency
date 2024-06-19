#![allow(clippy::expect_used, clippy::unwrap_used)]

use crate::blockchain::block::Block;
use crate::blockchain::block::Transaction;
use crate::proofs::inclusion::{InclusionInput, ProofOfInclusion};
use crate::proofs::liabilities::{LiabilitiesInput, MerkleSumTreeChange, ProofOfLiabilities};
use crate::proofs::setup::{CircuitSetup, PP};
use merkle_sum_tree::{Leaf, MerkleSumTree};
pub type Result<T> = std::result::Result<T, failure::Error>;
use std::collections::HashMap;

//TODO
//verify initial tree
//implement a check for max number of users
//Number of users is bounded to the size of the merkle tree in the circuits, need to compile a bigger tree
//if we want more users

const MAX_USERS: usize = 8;

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
            _ => 0,
        }
    }

    pub fn get_merkle_sum_tree(&self) -> MerkleSumTree {
        self.merkle_sum_tree.clone()
    }

    pub fn get_changes(&self) -> Vec<MerkleSumTreeChange> {
        self.changes.clone()
    }

    pub fn get_liabilities_circuit_setup(&self) -> &CircuitSetup {
        &self.liabilities_circuit_setup
    }

    pub fn get_inclusion_circuit_setup(&self) -> &CircuitSetup {
        &self.inclusion_circuit_setup
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
        )?;
        let block_hash = block.get_hash();
        let liabilities_proof = None;
        chain.insert(block_hash, block);
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
        )?;
        println!(
            "block num: {}, root_sum: {}, root_hash: {:?},  num of tx processed: {}",
            self.current_block_number,
            self.get_merkle_sum_tree().get_root_sum().unwrap(),
            self.get_merkle_sum_tree()
                .get_root_hash()
                .unwrap()
                .to_string(),
            self.mempool.len()
        );
        self.mempool.clear();
        self.current_hash = block.get_hash();
        self.chain.insert(block.get_hash(), block);

        Ok(())
    }

    fn update_blockchain_data(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        if transactions.len() == 0 {
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
                if number_from - amount < 0 {
                    println!("Insufficient balance");
                    break;
                }
                self.update_state(from, number_from - amount)?;
            }
            self.update_state(to, number_to + amount)?;
        }
        if self.get_changes().len() == 0 {
            return Ok(());
        }
        let _ = self.proove_merkle_tree();
        Ok(())
    }

    fn update_state(&mut self, address: String, amount: i32) -> Result<()> {
        self.state.insert(address.clone(), amount);
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
    ) -> (Option<ProofOfInclusion>, Option<Vec<Block>>, Option<PP>) {
        let index_option = self.leaf_index.get(&address);
        let index: usize;
        let mut blocks = vec![];
        if index_option.is_some() {
            index = *index_option.unwrap();
        } else {
            return (None, None, None);
        }
        let mut current_block = self.chain.get(&self.current_hash).unwrap();
        let mut inclusion_inputs = vec![];
        let mut last_hash = "".to_string();
        loop {
            let inclusion_input =
                InclusionInput::new(current_block.get_merkle_sum_tree(), index).unwrap();
            if inclusion_input.get_root_hash() == last_hash {
                blocks.pop();
                inclusion_inputs.pop();
            }
            blocks.push(current_block.clone());
            inclusion_inputs.push(inclusion_input.clone());

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
        let r1cs = self.get_inclusion_circuit_setup().get_r1cs();
        let pp = PP::new(r1cs);

        return (Some(proof_of_inclusion.unwrap()), Some(blocks), Some(pp));
    }

    pub fn get_liabilities_proof(&self) -> (Option<ProofOfLiabilities>, PP) {
        let r1cs = self.get_liabilities_circuit_setup().get_r1cs();
        let pp = PP::new(r1cs);
        (self.liabilities_proof.clone(), pp)
    }
}

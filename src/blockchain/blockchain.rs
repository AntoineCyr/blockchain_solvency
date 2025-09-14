#![allow(clippy::expect_used, clippy::unwrap_used)]

use crate::blockchain::block::Block;
use crate::blockchain::block::Transaction;
use crate::proofs::inclusion::{InclusionInput, ProofOfInclusion};
use crate::proofs::liabilities::{LiabilitiesInput, MerkleSumTreeChange, ProofOfLiabilities};
use crate::proofs::setup::{CircuitSetup, PP};
use merkle_sum_tree::{Leaf, MerkleSumTree};
use std::sync::Arc;
pub type Result<T> = std::result::Result<T, failure::Error>;
use std::collections::{HashMap};

pub const MAX_USERS: usize = 4;

pub struct Blockchain {
    current_hash: String,
    current_block_number: i32,
    mempool: Vec<Transaction>,
    chain: HashMap<String, Block>,
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
    pub fn get_balance(&self, address: &str) -> i32 {
        match self.state.get(address) {
            Some(&number) => number,
            _ => 0,
        }
    }

    pub fn get_merkle_sum_tree(&self) -> &MerkleSumTree {
        &self.merkle_sum_tree
    }

    fn clone_merkle_sum_tree(&self) -> MerkleSumTree {
        MerkleSumTree::new(self.merkle_sum_tree.get_leafs().to_vec()).unwrap()
    }

    pub fn get_changes(&self) -> &Vec<MerkleSumTreeChange> {
        &self.changes
    }

    pub fn create_blockchain() -> Result<Blockchain> {
        let mut chain = HashMap::new();
        let mempool = Vec::new();
        let state = HashMap::new();
        let leaf_index = HashMap::new();
        let leaf_0 = Leaf::new("0".to_string(), 0);
        let mut leafs = Vec::with_capacity(MAX_USERS);
        let changes = Vec::new();
        for _ in 0..MAX_USERS {
            leafs.push(leaf_0.clone());
        }
        let merkle_sum_tree = MerkleSumTree::new(leafs.clone()).unwrap();
        let current_block_number = 1;
        let block = Block::new(
            current_block_number,
            mempool.clone(),
            "0000000000000000000000000000000000000000000000000000000000000000",
            leaf_index.clone(),
            Arc::new(MerkleSumTree::new(leafs.clone()).unwrap()),
        )?;
        let block_hash = block.get_hash().to_string();
        let liabilities_proof = None;
        chain.insert(block_hash.clone(), block);
        
        println!("Initializing circuits in parallel...");
        let start_time = std::time::Instant::now();
        
        // Load circuits in parallel using threads
        let liabilities_handle = std::thread::spawn(|| {
            let start = std::time::Instant::now();
            let setup = CircuitSetup::new("liabilities_changes_folding");
            println!("  Liabilities circuit ready in {:?}", start.elapsed());
            setup
        });
        
        let inclusion_handle = std::thread::spawn(|| {
            let start = std::time::Instant::now();
            let setup = CircuitSetup::new("inclusion");
            println!("  Inclusion circuit ready in {:?}", start.elapsed());
            setup
        });
        
        let liabilities_circuit_setup = liabilities_handle.join().unwrap();
        let inclusion_circuit_setup = inclusion_handle.join().unwrap();
        
        println!("All circuits initialized in {:?}", start_time.elapsed());
        
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
        let mempool_transactions = std::mem::take(&mut self.mempool);
        let transaction_count = mempool_transactions.len();
        let _ = self.update_blockchain_data(mempool_transactions.clone());
        self.current_block_number += 1;
        let block = Block::new(
            self.current_block_number,
            mempool_transactions,
            &self.current_hash,
            self.leaf_index.clone(),
            Arc::new(self.clone_merkle_sum_tree()),
        )?;
        let merkle_tree = self.get_merkle_sum_tree();
        println!(
            "block num: {}, root_sum: {}, root_hash: {:?}, num of tx processed: {}",
            self.current_block_number,
            merkle_tree.get_root_sum().unwrap(),
            merkle_tree.get_root_hash().unwrap().to_string(),
            transaction_count
        );
        self.current_hash = block.get_hash().to_string();
        self.chain.insert(block.get_hash().to_string(), block);

        Ok(())
    }

    fn update_blockchain_data(&mut self, transactions: Vec<Transaction>) -> Result<()> { // Must take ownership
        if transactions.len() == 0 {
            return Ok(());
        }
        for transaction in transactions {
            let from = transaction.get_from();
            let to = transaction.get_to();
            let amount: i32 = transaction.get_amount();

            let number_from = match self.state.get(from) {
                Some(&number) => number,
                _ => 0,
            };
            let number_to = match self.state.get(to) {
                Some(&number) => number,
                _ => 0,
            };
            if from != "" {
                if number_from - amount < 0 {
                    println!("Insufficient balance");
                    continue;
                }
                self.update_state(&from, number_from - amount)?;
            }
            self.update_state(&to, number_to + amount)?;
        }
        if self.get_changes().len() == 0 {
            return Ok(());
        }
        let _ = self.prove_merkle_tree();
        Ok(())
    }

    fn update_state(&mut self, address: &str, amount: i32) -> Result<()> {
        let address_string = address.to_string();
        self.state.insert(address_string.clone(), amount);
        let index_option = self.leaf_index.get(address);
        let index: usize;
        let leaf = Leaf::new(address_string.clone(), amount);
        let old_merkle_tree = Arc::new(self.clone_merkle_sum_tree());

        if index_option.is_some() {
            _ = self
                .merkle_sum_tree
                .set_leaf(leaf.clone(), *index_option.unwrap());
            index = *index_option.unwrap();
        } else {
            index = self.merkle_sum_tree.push(leaf.clone()).unwrap();
            self.leaf_index.insert(address_string, index);
        }
        let new_merkle_tree = Arc::new(self.clone_merkle_sum_tree());
        let change = MerkleSumTreeChange::new(index, old_merkle_tree, new_merkle_tree);
        self.liabilities_verified = false;
        self.changes.push(change);
        Ok(())
    }

    fn prove_merkle_tree(&mut self) -> Result<()> {
        let changes = std::mem::take(&mut self.changes);
        let mut liabilities_inputs = vec![];
        for change in changes {
            liabilities_inputs.push(LiabilitiesInput::new(vec![change]).unwrap())
        }
        let circuit_setup = &self.liabilities_circuit_setup;
        let (liabilities_proof, _pp) = ProofOfLiabilities::new(liabilities_inputs, circuit_setup)?;
        self.liabilities_proof = Some(liabilities_proof);
        self.liabilities_verified = true;
        Ok(())
    }

    pub fn add_transaction(&mut self, from: &str, to: &str, amount: i32) -> Result<()> {
        let transaction = Transaction::new(from.to_string(), to.to_string(), amount);
        self.mempool.push(transaction);

        Ok(())
    }

    pub fn get_inclusion_proof(
        &self,
        address: &str,
    ) -> (Option<ProofOfInclusion>, Option<Vec<Block>>, Option<PP>) {
        let index_option = self.leaf_index.get(address);
        let index: usize;
        let mut blocks = vec![];
        if index_option.is_some() {
            index = *index_option.unwrap();
        } else {
            return (None, None, None);
        }
        let mut current_block = self.chain.get(&self.current_hash).unwrap();
        let mut inclusion_inputs = vec![];
        let mut last_root_hash = "".to_string();

        loop {
            let user_leaf = current_block.get_merkle_sum_tree().get_leaf(index);
            match user_leaf {
                Ok(leaf) => {
                    if leaf.get_id() != address {
                        break;
                    }
                },
                Err(_) => {
                    break;
                }
            }
            
            let inclusion_input =
                InclusionInput::new(current_block.get_merkle_sum_tree(), index).unwrap();
            let current_root_hash = inclusion_input.get_root_hash().to_string();
            
            // Only include blocks with unique tree states (ignore consecutive blocks with same tree)
            if current_root_hash != last_root_hash {
                blocks.push(current_block.clone());
                inclusion_inputs.push(inclusion_input.clone());
                last_root_hash = current_root_hash;
            }

            // Move to previous block
            let prev_hash = current_block.get_previous_hash();
            
            if let Some(prev_block) = self.chain.get(prev_hash) {
                current_block = prev_block;
            } else {
                // Previous block not found - this indicates we've reached genesis
                break;
            }
        }
        
        let proof =
            ProofOfInclusion::new(inclusion_inputs, &self.inclusion_circuit_setup);
        
        match proof {
            Ok((proof, client_pp)) => {                                
                return (Some(proof), Some(blocks), Some(client_pp));
            },
            Err(e) => {
                println!("Proof creation failed: {}", e);
                return (None, None, None);
            }
        }
    }

    pub fn get_liabilities_proof(&self) -> (Option<ProofOfLiabilities>, PP) {
        let pp = PP::from_circuit_setup(&self.liabilities_circuit_setup);
        (self.liabilities_proof.clone(), pp)
    }
}

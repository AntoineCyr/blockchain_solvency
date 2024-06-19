use crate::blockchain::blockchain::Blockchain;
use crate::errors::Result;
use crate::proofs::inclusion::ProofOfInclusion;
use crate::proofs::liabilities::ProofOfLiabilities;
use crate::proofs::setup::PP;
use serde::{Deserialize, Serialize};
use std::sync::MutexGuard;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockInclusion {
    balance: i32,
    root_hash: String,
    root_sum: i32,
    block_number: i32,
    timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockchainInclusion {
    balances: Vec<BlockInclusion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockWrapper {
    root_hash: String,
    root_sum: i32,
    block_number: i32,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProofOfInclusionWrapper {
    proof: ProofOfInclusion,
    wrap_blocks: Vec<BlockWrapper>,
    pp: PP,
}

#[derive(Serialize, Deserialize)]
pub struct ProofOfLiabilitiesWrapper {
    proof: ProofOfLiabilities,
    pp: PP,
}

impl BlockWrapper {
    pub fn get_root_hash(&self) -> String {
        self.root_hash.clone()
    }

    pub fn get_root_sum(&self) -> i32 {
        self.root_sum
    }

    pub fn get_block_number(&self) -> i32 {
        self.block_number
    }

    pub fn get_timestamp(&self) -> String {
        self.timestamp.clone()
    }
}

impl ProofOfInclusionWrapper {
    pub fn get_proof(&self) -> ProofOfInclusion {
        self.proof.clone()
    }

    pub fn get_wrap_blocks(&self) -> Vec<BlockWrapper> {
        self.wrap_blocks.clone()
    }

    pub fn get_pp(self) -> PP {
        self.pp
    }

    pub fn serialize(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn deserialize(proof_of_inclusion_wrapper: String) -> Result<ProofOfInclusionWrapper> {
        match serde_json::from_str(&proof_of_inclusion_wrapper) {
            Ok(data) => Ok(data),
            Err(error) => Result::Err(error.into()),
        }
    }
}

impl ProofOfLiabilitiesWrapper {
    pub fn get_proof(&self) -> ProofOfLiabilities {
        self.proof.clone()
    }

    pub fn get_pp(self) -> PP {
        self.pp
    }

    pub fn serialize(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn deserialize(proof_of_liabilitie_wrapper: String) -> Result<ProofOfLiabilitiesWrapper> {
        match serde_json::from_str(&proof_of_liabilitie_wrapper) {
            Ok(data) => Ok(data),
            Err(error) => Result::Err(error.into()),
        }
    }
}

impl BlockInclusion {
    pub fn new(
        balance: i32,
        root_hash: String,
        root_sum: i32,
        block_number: i32,
        timestamp: String,
    ) -> BlockInclusion {
        BlockInclusion {
            balance,
            root_hash,
            root_sum,
            block_number,
            timestamp,
        }
    }
}

impl BlockchainInclusion {
    pub fn new(balances: Vec<BlockInclusion>) -> BlockchainInclusion {
        BlockchainInclusion { balances }
    }

    pub fn serialize(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn deserialize(balance_history: String) -> Result<BlockchainInclusion> {
        match serde_json::from_str(&balance_history) {
            Ok(data) => Ok(data),
            Err(error) => Result::Err(error.into()),
        }
    }
}

pub fn transfer(
    mut bc: MutexGuard<Blockchain>,
    from: &str,
    to: &str,
    amoun_chars: &str,
) -> Result<String> {
    let mut amount = String::from("");
    for c in amoun_chars.chars() {
        if c.is_digit(10) {
            amount.push(c);
        }
    }
    let _ = bc.add_transaction(
        String::from(from),
        String::from(to),
        amount.parse().unwrap(),
    );
    Ok("transaction added to mempool!".to_string())
}

pub fn get_balance_history(bc: MutexGuard<Blockchain>, address_chars: &str) -> Result<String> {
    let mut address = String::from("");
    for c in address_chars.chars() {
        if c.is_alphanumeric() {
            address.push(c);
        }
    }
    let (inclusion_proof, blocks, pp) = bc.get_inclusion_proof(address.clone());
    match inclusion_proof {
        Some(proof) => {
            let mut wrap_blocks = vec![];
            for block in blocks.unwrap() {
                let block_wrapper = BlockWrapper {
                    root_hash: block
                        .get_merkle_sum_tree()
                        .get_root_hash()
                        .unwrap()
                        .to_string(),
                    root_sum: block.get_merkle_sum_tree().get_root_sum().unwrap(),
                    block_number: block.get_block_number(),
                    timestamp: block.get_timestamp(),
                };
                wrap_blocks.push(block_wrapper)
            }
            let proof_wrapper = ProofOfInclusionWrapper {
                proof,
                wrap_blocks,
                pp: pp.unwrap(),
            };
            Ok(proof_wrapper.serialize())
        }
        None => Ok("No liabilities proof".to_string()),
    }
}

pub fn get_balance(bc: MutexGuard<Blockchain>, address_chars: &str) -> Result<String> {
    let mut address = String::from("");
    for c in address_chars.chars() {
        if c.is_alphanumeric() {
            address.push(c);
        }
    }
    let balance = bc.get_balance(String::from(address.clone()));
    let output = format!("balance: {}", balance);
    Ok(output)
}

pub fn get_liabilities_proof(bc: MutexGuard<Blockchain>) -> Result<String> {
    let (proof, pp) = bc.get_liabilities_proof();
    //create type proof + pp
    match proof {
        Some(proof) => {
            let proof_wrapper = ProofOfLiabilitiesWrapper { proof, pp };
            Ok(proof_wrapper.serialize())
        }
        None => Ok("No liabilities proof".to_string()),
    }
}

use crate::blockchain::blockchain::Blockchain;
use crate::errors::Result;
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

#[derive(Serialize, Deserialize)]
pub struct ProofOfLiabilitiesWrapper {
    proof: ProofOfLiabilities,
    pp: PP,
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
    let (inclusion_proof, blocks) = bc.get_inclusion_proof(address.clone());
    let mut output = "".to_string();
    let mut inclusion_outputs = vec![];
    match inclusion_proof {
        Some(proof) => {
            for (inclusion, block) in proof.get_input().iter().zip(blocks.unwrap().iter()) {
                let root_hash = inclusion.get_root_hash();
                let root_sum = inclusion.get_root_sum();
                let balance = inclusion.get_user_balance();
                let block_number = block.get_block_number();
                let timestamp = block.get_timestamp();
                let inclusion_output =
                    BlockInclusion::new(balance, root_hash, root_sum, block_number, timestamp);
                inclusion_outputs.push(inclusion_output);
            }
            let inclusion_output_history = BlockchainInclusion::new(inclusion_outputs);
            output.push_str(&inclusion_output_history.serialize());
        }
        _ => output.push_str("No current balance for user"),
    }
    Ok(output)
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

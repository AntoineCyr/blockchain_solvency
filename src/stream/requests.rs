use crate::blockchain::blockchain::Blockchain;
use crate::errors::Result;
use crate::proofs::inclusion::ProofOfInclusion;
use crate::proofs::liabilities::ProofOfLiabilities;
use crate::proofs::setup::PP;
use serde::{Deserialize, Serialize};
use std::sync::MutexGuard;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockInclusion {
    user_balance: i32,
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
    pub fn get_block_number(&self) -> i32 {
        self.block_number
    }

    pub fn get_timestamp(&self) -> &str {
        &self.timestamp
    }
}

impl ProofOfInclusionWrapper {
    pub fn get_proof(&self) -> &ProofOfInclusion {
        &self.proof
    }

    pub fn into_parts(self) -> (ProofOfInclusion, Vec<BlockWrapper>, PP) {
        (self.proof, self.wrap_blocks, self.pp)
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
        user_balance: i32,
        root_hash: String,
        root_sum: i32,
        block_number: i32,
        timestamp: String,
    ) -> BlockInclusion {
        BlockInclusion {
            user_balance,
            root_hash,
            root_sum,
            block_number,
            timestamp,
        }
    }
    
    pub fn user_balance(&self) -> i32 {
        self.user_balance
    }
    
    pub fn root_sum(&self) -> i32 {
        self.root_sum
    }
    
    pub fn block_number(&self) -> i32 {
        self.block_number
    }
    
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }
}


pub fn transfer(
    mut bc: MutexGuard<Blockchain>,
    from: &str,
    to: &str,
    amoun_chars: &str,
) -> Result<String> {
    let amount: String = amoun_chars.chars().filter(|c| c.is_digit(10)).collect();
    let _ = bc.add_transaction(
        from,
        to,
        amount.parse().unwrap(),
    );
    Ok("transaction added to mempool!".to_string())
}

pub fn get_balance_history(bc: MutexGuard<Blockchain>, address_chars: &str) -> Result<String> {
    let address: String = address_chars.chars().filter(|c| c.is_alphanumeric()).collect();
    let (inclusion_proof, blocks, pp) = bc.get_inclusion_proof(&address);
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
                    timestamp: block.get_timestamp().to_string(),
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
    let address: String = address_chars.chars().filter(|c| c.is_alphanumeric()).collect();
    let balance = bc.get_balance(&address);
    let output = format!("balance: {}", balance);
    Ok(output)
}

pub fn get_liabilities_proof(bc: MutexGuard<Blockchain>) -> Result<String> {
    let (proof, pp) = bc.get_liabilities_proof();
    match proof {
        Some(proof) => {
            let proof_wrapper = ProofOfLiabilitiesWrapper { proof, pp };
            Ok(proof_wrapper.serialize())
        }
        None => Ok("No liabilities proof".to_string()),
    }
}

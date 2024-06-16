use crate::blockchain::blockchain::Blockchain;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::sync::MutexGuard;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug)]
struct InclusionOutput {
    balance: i32,
    root_hash: String,
    root_sum: i32,
    block_number: i32,
    timestamp: SystemTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InclusionOutputHistory {
    balances: Vec<InclusionOutput>,
}

impl InclusionOutput {
    pub fn new(
        balance: i32,
        root_hash: String,
        root_sum: i32,
        block_number: i32,
        timestamp: SystemTime,
    ) -> InclusionOutput {
        InclusionOutput {
            balance,
            root_hash,
            root_sum,
            block_number,
            timestamp,
        }
    }
}

impl InclusionOutputHistory {
    pub fn new(balances: Vec<InclusionOutput>) -> InclusionOutputHistory {
        InclusionOutputHistory { balances }
    }

    pub fn serialize(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn deserialize(balance_history: String) -> Result<InclusionOutputHistory> {
        let deserialized: InclusionOutputHistory = serde_json::from_str(&balance_history).unwrap();
        Ok(deserialized)
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
            format!("Historical Balance of '{address}'\n");
            println!("proof length:{}", proof.get_input().len());
            for (inclusion, block) in proof.get_input().iter().zip(blocks.unwrap().iter()) {
                let root_hash = inclusion.get_root_hash();
                let root_sum = inclusion.get_root_sum();
                let balance = inclusion.get_user_balance();
                let timestamp = block.get_timestamp();
                let block_number = block.get_block_number();
                let inclusion_output =
                    InclusionOutput::new(balance, root_hash, root_sum, block_number, timestamp);
                inclusion_outputs.push(inclusion_output);
            }
            let inclusion_output_history = InclusionOutputHistory::new(inclusion_outputs);
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

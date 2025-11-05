#![allow(clippy::expect_used, clippy::unwrap_used)]
use crate::errors::Result;
use crate::stream::requests::{
    BlockInclusion, ProofOfInclusionWrapper, ProofOfLiabilitiesWrapper,
};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;


pub struct Client {}

impl Client {
    pub fn new() -> Result<Client> {
        Ok(Client {})
    }

    pub fn get_balance(&self, address: &str) {
        match self.get_balance_internal(address) {
            Ok(_) => {},
            Err(e) => eprintln!("Failed to get balance: {}", e),
        }
    }

    fn get_balance_internal(&self, address: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect("127.0.0.1:8888")?;
        let mut buffer: Vec<u8> = Vec::with_capacity(512);
        let input = format!("balance_{address}\n");
        stream.write(input.as_bytes())?;

        let mut reader = BufReader::new(&stream);
        reader.read_until(b'\n', &mut buffer)?;
        print!("{}", str::from_utf8(&buffer)?);
        Ok(())
    }

    pub fn get_balance_history(&self, address: &str) {
        match self.get_balance_history_internal(address) {
            Ok(_) => {},
            Err(e) => eprintln!("Failed to get balance history: {}", e),
        }
    }

    fn get_balance_history_internal(&self, address: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect("127.0.0.1:8888")?;
        let mut buffer: Vec<u8> = Vec::with_capacity(512);
        let input = format!("balance_history_{address}\n");
        stream.write(input.as_bytes())?;

        let mut reader = BufReader::new(&stream);

        reader.read_until(b'\n', &mut buffer)?;
        let data = str::from_utf8(&buffer)?.to_string();
        let deserialized: std::result::Result<ProofOfInclusionWrapper, failure::Error> =
            ProofOfInclusionWrapper::deserialize(data.clone());

        let mut inclusion_outputs = vec![];
        match deserialized {
            Ok(proof_wrapper) => {
                println!("Received inclusion proof for {} unique trees", 
                         proof_wrapper.get_proof().get_inclusion_inputs().len());
                
                // Verify the inclusion proof and extract all step_outs
                println!("Starting client-side verification of inclusion folding...");
                
                // Extract components to verify the proof
                let (proof, wrap_blocks, pp) = proof_wrapper.into_parts();

                let verification_result = proof.verify(pp);
                let blocks = &wrap_blocks;
                
                match verification_result {
                    Ok(()) => {
                        println!("Inclusion proof verification successful!");
                        
                        // Build human-readable history from verified proof
                        for (inclusion, block) in proof
                            .get_inclusion_inputs()
                            .iter()
                            .zip(blocks.iter())
                        {
                            let root_hash = inclusion.get_root_hash();
                            let root_sum = inclusion.get_root_sum();
                            let balance = inclusion.get_user_balance();
                            let block_number = block.get_block_number();
                            let timestamp = block.get_timestamp();
                            let inclusion_output =
                                BlockInclusion::new(balance, root_hash.to_string(), root_sum, block_number, timestamp.to_string());
                            inclusion_outputs.push(inclusion_output);
                        }
                        
                        println!("\n=== Verified Balance History ===");
                        for (i, entry) in inclusion_outputs.iter().enumerate() {
                            println!("{}. Block {} | Balance: {} | Tree Sum: {} | Timestamp: {}", 
                                     i + 1, 
                                     entry.block_number(), 
                                     entry.user_balance(), 
                                     entry.root_sum(), 
                                     entry.timestamp());
                        }
                        println!("=================================\n");
                    }
                    Err(e) => {
                        println!("Inclusion proof verification failed: {}", e);
                    }
                }
            }
            Err(_) => {
                println!("Failed to deserialize inclusion proof from server");
            }
        }
        Ok(())
    }

    pub fn add_transaction(&self, from: &str, to: &str, amount: i32) {
        match self.add_transaction_internal(from, to, amount) {
            Ok(_) => {},
            Err(e) => eprintln!("Failed to add transaction: {}", e),
        }
    }

    fn add_transaction_internal(&self, from: &str, to: &str, amount: i32) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect("127.0.0.1:8888")?;
        let mut buffer: Vec<u8> = Vec::with_capacity(512);
        let input = format!("transfer_{from}_{to}_{amount}\n");

        stream.write(input.as_bytes())?;
        let mut reader = BufReader::new(&stream);
        reader.read_until(b'\n', &mut buffer)?;
        print!("{}", str::from_utf8(&buffer)?);
        Ok(())
    }

    pub fn verify_liabilities(&self) {
        match self.verify_liabilities_internal() {
            Ok(_) => {},
            Err(e) => eprintln!("Failed to verify liabilities: {}", e),
        }
    }

    fn verify_liabilities_internal(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect("127.0.0.1:8888")?;
        let mut buffer: Vec<u8> = Vec::with_capacity(512);
        let input = "verify_filler\n";

        stream.write(input.as_bytes())?;
        let mut reader = BufReader::new(&stream);
        reader.read_until(b'\n', &mut buffer)?;

        let data = str::from_utf8(&buffer)?.to_string();
        let deserialized: std::result::Result<ProofOfLiabilitiesWrapper, failure::Error> =
            ProofOfLiabilitiesWrapper::deserialize(data.clone());

        match deserialized {
            Ok(proof_of_liabilities_wrapper) => {
                let liabilities_proof = proof_of_liabilities_wrapper.get_proof();
                let res = liabilities_proof.verify(proof_of_liabilities_wrapper.get_pp());
                match res {
                    Ok(liabilities_output) => {
                        println!("{:#?}", liabilities_output)
                    }
                    Err(error) => println!("{:#?}", error),
                }
            }
            Err(_) => println!("{}", data),
        }
        Ok(())
    }
}

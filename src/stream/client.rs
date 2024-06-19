#![allow(clippy::expect_used, clippy::unwrap_used)]
use crate::errors::Result;
use crate::stream::requests::{
    BlockInclusion, BlockchainInclusion, ProofOfInclusionWrapper, ProofOfLiabilitiesWrapper,
};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;

pub struct Client {}

impl Client {
    pub fn new() -> Result<Client> {
        Ok(Client {})
    }

    pub fn get_balance(&self, address: String) {
        let mut stream = TcpStream::connect("127.0.0.1:8888").expect("Could not connect to ser$");
        let mut buffer: Vec<u8> = Vec::new();
        let input = format!("balance_{address}");
        stream
            .write(input.as_bytes())
            .expect("Failed to write to server");

        let mut reader = BufReader::new(&stream);

        reader
            .read_until(b'\n', &mut buffer)
            .expect("Could not read into buffer");
        print!(
            "{}",
            str::from_utf8(&buffer).expect("Could not write buffer as string")
        );
    }

    //need to verify
    pub fn get_balance_history(&self, address: String) {
        let mut stream = TcpStream::connect("127.0.0.1:8888").expect("Could not connect to ser$");
        let mut buffer: Vec<u8> = Vec::new();
        let input = format!("balance_history_{address}");
        stream
            .write(input.as_bytes())
            .expect("Failed to write to server");

        let mut reader = BufReader::new(&stream);

        reader
            .read_until(b'\n', &mut buffer)
            .expect("Could not read into buffer");
        let data = format!(
            "{}",
            str::from_utf8(&buffer).expect("Could not write buffer as string")
        );
        let deserialized: Result<ProofOfInclusionWrapper> =
            ProofOfInclusionWrapper::deserialize(data.clone());

        let mut inclusion_outputs = vec![];
        match deserialized {
            Ok(proof_wrapper) => {
                for (inclusion, block) in proof_wrapper
                    .get_proof()
                    .get_inclusion_inputs()
                    .iter()
                    .zip(proof_wrapper.get_wrap_blocks().iter())
                {
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
                println!("{:#?}", inclusion_output_history)
            }
            Err(_) => println!("{}", data),
        }
    }

    pub fn add_transaction(&self, from: String, to: String, amount: i32) {
        let mut stream = TcpStream::connect("127.0.0.1:8888").expect("Could not connect to ser$");
        let mut buffer: Vec<u8> = Vec::new();
        let input = format!("transfer_{from}_{to}_{amount}");

        stream
            .write(input.as_bytes())
            .expect("Failed to write to server");

        let mut reader = BufReader::new(&stream);

        reader
            .read_until(b'\n', &mut buffer)
            .expect("Could not read into buffer");
        print!(
            "{}",
            str::from_utf8(&buffer).expect("Could not write buffer as string")
        );
    }

    pub fn verify_liabilities(&self) {
        let mut stream = TcpStream::connect("127.0.0.1:8888").expect("Could not connect to ser$");
        let mut buffer: Vec<u8> = Vec::new();
        let input = "verify_filler";

        stream
            .write(input.as_bytes())
            .expect("Failed to write to server");
        let mut reader = BufReader::new(&stream);
        reader
            .read_until(b'\n', &mut buffer)
            .expect("Could not read into buffer");

        let data = format!(
            "{}",
            str::from_utf8(&buffer).expect("Could not write buffer as string")
        );
        let deserialized: Result<ProofOfLiabilitiesWrapper> =
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
    }
}

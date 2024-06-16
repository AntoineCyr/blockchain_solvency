#![allow(clippy::expect_used, clippy::unwrap_used)]
use crate::errors::Result;
use crate::stream::requests::BlockchainInclusion;
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
        let deserialized: Result<BlockchainInclusion> =
            BlockchainInclusion::deserialize(data.clone());
        match deserialized {
            Ok(inclusion_output_history) => println!("{:#?}", inclusion_output_history),
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
}

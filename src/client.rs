#![allow(clippy::expect_used, clippy::unwrap_used)]

use bellpepper_core::ConstraintSystem;
use circom_scotia::{calculate_witness, r1cs::CircomConfig, synthesize};
use ff::Field;

use pasta_curves::vesta::Base as Fr;
use std::env::current_dir;

use bellpepper_core::test_cs::TestConstraintSystem;
use bellpepper_core::Comparable;

use crate::errors::Result;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;

pub struct Client {}

impl Client {
    pub fn new() -> Result<Client> {
        Ok(Client {})
    }

    //normalize first word should be command
    pub fn get_balance(&self, address: String) {
        let mut stream = TcpStream::connect("127.0.0.1:8888").expect("Could not connect to ser$");
        let mut buffer: Vec<u8> = Vec::new();
        stream
            .write(address.as_bytes())
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
        stream
            .write(address.as_bytes())
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

    pub fn create_proof(&self) {
        //https://github.com/lurk-lab/circom-scotia
        let root = current_dir().unwrap().join("../circuits/liabilities_js");
        let wtns = root.join("liabilities.wasm");
        let r1cs = root.join("liabilities.r1cs");

        let cfg = CircomConfig::new(wtns, r1cs).unwrap();
        let balance = ("balance".into(), vec![Fr::ZERO, Fr::ZERO]);
        let email_hash = ("emailHash".into(), vec![Fr::ZERO, Fr::ZERO]);
        let input = vec![balance, email_hash];

        let witness = calculate_witness(&cfg, input, true).expect("msg");

        // Create an empty instance for setting it up
        let mut cs = TestConstraintSystem::<Fr>::new();

        let _output = synthesize(
            &mut cs.namespace(|| "liabilities_circom"),
            cfg.r1cs.clone(),
            Some(witness),
        );

        println!("cs: {:?}", cs.get(&cs.aux()[0]));
        println!("cs: {:?}", cs.get(&cs.aux()[1]));
        println!("cs: {:?}", cs.get(&cs.aux()[2]));
        println!("cs: {:?}", cs.get(&cs.aux()[3]));
    }

    pub fn get_leafs(&self) {
        let mut stream = TcpStream::connect("127.0.0.1:8888").expect("Could not connect to ser$");
        let mut buffer: Vec<u8> = Vec::new();
        stream
            .write("get_leafs".as_bytes())
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

    pub fn add_transaction(&self, from: String, to: String, amount: i32) {
        let mut stream = TcpStream::connect("127.0.0.1:8888").expect("Could not connect to ser$");
        let mut buffer: Vec<u8> = Vec::new();
        let input = format!("{from}_{to}_{amount}");

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

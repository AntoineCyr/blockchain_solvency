use crate::blockchain::Blockchain;
use crate::errors::Result;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub struct Server {}

impl Server {
    pub fn new() -> Result<Server> {
        Ok(Server {})
    }

    pub fn run_server(&self) {
        fn handle_client(mut stream: TcpStream, bc: Arc<Mutex<Blockchain>>) -> Result<()> {
            println!("Incoming connection from: {}", stream.peer_addr()?);
            let mut buf = [0; 512];
            let mut bc = bc.lock().unwrap();
            stream.read(&mut buf).unwrap();
            let parts = str::from_utf8(&buf).unwrap().split("_");
            let collection = parts.collect::<Vec<&str>>();
            if collection.len() == 3 {
                let mut amount = String::from("");
                for c in collection[2].chars() {
                    if c.is_digit(10) {
                        amount.push(c);
                    }
                }
                let _ = bc.add_transaction(
                    String::from(collection[0]),
                    String::from(collection[1]),
                    amount.parse().unwrap(),
                );
                stream
                    .write("transaction added to mempool!".as_bytes())
                    .expect("Failed to write response!");
            } else if collection.len() == 1 {
                let mut address = String::from("");
                for c in collection[0].chars() {
                    if c.is_alphanumeric() {
                        address.push(c);
                    }
                }
                let balance = bc.get_balance(String::from(address.clone()));
                let output = format!("Balance of '{address}'; {balance} ");
                stream
                    .write(output.as_bytes())
                    .expect("Failed to write response!");
            } else if collection.len() == 2 {
                //bc.create_proof();
                let leafs = bc.get_leafs();
                let output = format!("Leafs: {:?}", leafs);
                stream
                    .write(output.as_bytes())
                    .expect("Failed to write response!");
            }
            let _ = stream.write(&[b'\n']);
            Ok(())
        }

        let bc = Blockchain::create_blockchain().unwrap();
        let bc = Arc::new(Mutex::new(bc));
        let bc2 = Arc::clone(&bc);
        thread::spawn(move || loop {
            sleep(Duration::new(10, 0));
            let mut bc = bc.lock().unwrap();
            let _ = bc.add_block();
        });

        let listener = TcpListener::bind("0.0.0.0:8888").expect("Could not bind");
        for stream in listener.incoming() {
            match stream {
                Err(e) => {
                    eprintln!("failed: {}", e)
                }
                Ok(stream) => {
                    let bc = Arc::clone(&bc2);
                    thread::spawn(move || {
                        handle_client(stream, bc).unwrap_or_else(|error| eprintln!("{:?}", error));
                    });
                }
            }
        }
    }
}

use crate::blockchain::blockchain::Blockchain;
use crate::errors::Result;
use crate::stream::requests::{get_balance, get_balance_history, get_liabilities_proof, transfer};
use std::io::{BufRead, BufReader, Write};
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
            
            // Use dynamic reading instead of fixed buffer
            let mut request = String::new();
            let mut reader = BufReader::new(&mut stream);
            reader.read_line(&mut request)?;
            
            let bc = bc.lock().map_err(|_| failure::format_err!("Mutex poisoned"))?;
            let parts = request.trim().split('_').collect::<Vec<&str>>();
            
            // Validate request has minimum required parts
            if parts.is_empty() {
                return Err(failure::format_err!("Empty request"));
            }
            let output: Result<String> = match parts[0] {
                "transfer" => {
                    if parts.len() < 4 {
                        Err(failure::format_err!("Transfer requires 4 parameters"))
                    } else {
                        transfer(bc, parts[1], parts[2], parts[3])
                    }
                },
                "verify" => get_liabilities_proof(bc),
                "balance" => {
                    if parts.len() < 2 {
                        Err(failure::format_err!("Balance request requires address"))
                    } else if parts[1] == "history" {
                        if parts.len() < 3 {
                            Err(failure::format_err!("Balance history requires address"))
                        } else {
                            get_balance_history(bc, parts[2])
                        }
                    } else {
                        get_balance(bc, parts[1])
                    }
                }
                _ => Ok("Wrong command".to_string()),
            };
            match output {
                Ok(output_value) => {
                    stream.write(output_value.as_bytes())?;
                }
                Err(e) => {
                    let error_msg = format!("Internal error: {}", e);
                    stream.write(error_msg.as_bytes())?;
                }
            }
            stream.write(&[b'\n'])?;
            Ok(())
        }

        let bc = Blockchain::create_blockchain()
            .expect("Failed to create blockchain");
        let bc = Arc::new(Mutex::new(bc));
        let bc2 = Arc::clone(&bc);
        thread::spawn(move || loop {
            sleep(Duration::new(10, 0));
            if let Ok(mut blockchain) = bc.lock() {
                if let Err(e) = blockchain.add_block() {
                    eprintln!("Failed to add block: {}", e);
                }
            } else {
                eprintln!("Failed to acquire blockchain lock");
            }
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

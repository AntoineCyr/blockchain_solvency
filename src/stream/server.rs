use crate::blockchain::blockchain::Blockchain;
use crate::errors::Result;
use crate::stream::requests::{get_balance, get_balance_history, transfer};
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
            let output: Result<String> = match collection[0] {
                "transfer" => transfer(bc, collection[1], collection[2], collection[3]),
                "balance" => {
                    if collection[1] == "history" {
                        get_balance_history(bc, collection[2])
                    } else {
                        get_balance(bc, collection[1])
                    }
                }
                _ => Ok("Wrong command".to_string()),
            };
            match output {
                Ok(output_value) => {
                    stream
                        .write(output_value.as_bytes())
                        .expect("Failed to write response!");
                }
                Err(_) => {
                    stream
                        .write("Internal error".as_bytes())
                        .expect("Failed to write response!");
                }
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

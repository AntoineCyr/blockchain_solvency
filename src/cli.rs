use crate::client::Client;
use crate::errors::Result;
use crate::server::Server;
use clap::{arg, Command};
use std::process::exit;

pub struct Cli {}

impl Cli {
    pub fn new() -> Result<Cli> {
        Ok(Cli {})
    }
    pub fn run(&mut self) -> Result<()> {
        let matches = Command::new("blockchain-rust-demo")
            .version("0.1")
            .author("acyrntoine@gmail.com")
            .about("simple blockchain state")
            .subcommand(
                Command::new("balance")
                    .about("get balance in the blochain")
                    .arg(arg!(<ADDRESS>"'The Address it get balance for'")),
            )
            .subcommand(Command::new("start-node").about("Create new blokchain"))
            .subcommand(Command::new("leafs").about("Get Leafs of Tree"))
            .subcommand(Command::new("proof").about("Create Proof"))
            .subcommand(
                Command::new("transfer")
                    .about("trasnfer in the blockchain")
                    .arg(arg!(<FROM>" 'Source address'"))
                    .arg(arg!(<TO>" 'Destination address'"))
                    .arg(arg!(<AMOUNT>" 'Amount'")),
            )
            .subcommand(
                Command::new("create-account")
                    .about("create a new account")
                    .arg(arg!(<ID>" 'address'"))
                    .arg(arg!(<AMOUNT>" 'Amount'")),
            )
            .get_matches();

        if let Some(ref _matches) = matches.subcommand_matches("start-node") {
            let server = Server::new()?;
            server.run_server();
        }

        if let Some(ref matches) = matches.subcommand_matches("balance") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let address = String::from(address);
                let client = Client::new()?;
                client.get_balance(address);
            }
        }

        if let Some(ref _matches) = matches.subcommand_matches("leafs") {
            let client = Client::new()?;
            client.get_leafs();
        }

        if let Some(ref _matches) = matches.subcommand_matches("proof") {
            let client = Client::new()?;
            client.create_proof();
        }

        if let Some(ref matches) = matches.subcommand_matches("transfer") {
            let from = if let Some(address) = matches.get_one::<String>("FROM") {
                address
            } else {
                println!("from not supply!: usage");
                exit(1)
            };

            let to = if let Some(address) = matches.get_one::<String>("TO") {
                address
            } else {
                println!("from not supply!: usage");
                exit(1)
            };

            let amount: i32 = if let Some(amount) = matches.get_one::<String>("AMOUNT") {
                amount.parse()?
            } else {
                println!("from not supply!: usage");
                exit(1)
            };

            let client = Client::new()?;
            client.add_transaction(from.clone(), to.clone(), amount);
        }

        if let Some(ref matches) = matches.subcommand_matches("create-account") {
            let id = if let Some(address) = matches.get_one::<String>("ID") {
                address
            } else {
                println!("from not supply!: usage");
                exit(1)
            };

            let amount: i32 = if let Some(amount) = matches.get_one::<String>("AMOUNT") {
                amount.parse()?
            } else {
                println!("from not supply!: usage");
                exit(1)
            };

            let client = Client::new()?;
            client.add_transaction(String::from(""), id.clone(), amount);
        }

        Ok(())
    }
}

use crate::cli::Cli;
use crate::errors::Result;

mod block;
mod blockchain;
mod cli;
mod client;
mod errors;
mod proofs;
mod server;

fn main() -> Result<()> {
    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())
}

use crate::cli::Cli;
use crate::errors::Result;

mod blockchain;
mod block;
mod cli;
mod errors;
mod server;
mod client;

fn main() ->Result<()>{
    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())

}

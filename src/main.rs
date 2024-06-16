use crate::errors::Result;
use crate::stream::cli::Cli;

mod blockchain;
mod errors;
mod proofs;
mod stream;

fn main() -> Result<()> {
    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())
}

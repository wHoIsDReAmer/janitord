use janitord::{
    cli::{Cli, Command},
    core::scan::scan,
};

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Scan(args) => {
            let vec = scan(&args.paths)?;
            println!("{:?}", vec)
        }
    }

    Ok(())
}

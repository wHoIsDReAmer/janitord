use janitord::cli::{Cli, Command};

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Scan(args) => {
            let vec = janitord::scan::scan(&args.paths)?;
            println!("{:?}", vec)
        }
    }

    Ok(())
}

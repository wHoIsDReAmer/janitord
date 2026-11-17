mod cli;

use clap::Parser;

use crate::cli::{Cli, Command};

fn main() {
    match Cli::parse().command {
        Command::Scan(args) => todo!("scan {:?}", args.paths),
    }
}

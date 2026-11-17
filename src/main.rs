use clap::Parser;

use janitord::cli::{Cli, Command};

fn main() {
    match Cli::parse().command {
        Command::Scan(args) => {
            let vec = janitord::scan::scan(&args.paths);
            println!("{:?}", vec)
        }
    }
}

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Scan(ScanArgs),
}

#[derive(Debug, Args)]
pub struct ScanArgs {
    #[arg(value_name = "DIR", default_value = ".")]
    pub paths: Vec<PathBuf>,
}

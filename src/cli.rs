use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgGroup, Args, Parser, Subcommand};
use jiff::Span;

use crate::core::policy::Policy;

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
#[command(group(ArgGroup::new("filter").args(["older_than", "min_size"]).multiple(true)))]
pub struct ScanArgs {
    #[arg(value_name = "DIR", default_value = ".")]
    pub paths: Vec<PathBuf>,

    #[arg(long, value_name = "SPAN")]
    pub older_than: Option<Span>,

    #[arg(long, value_name = "BYTES")]
    pub min_size: Option<u64>,

    #[arg(long, requires = "filter")]
    pub clean: bool,
}

impl ScanArgs {
    pub fn has_filter(&self) -> bool {
        self.older_than.is_some() || self.min_size.is_some()
    }

    pub fn policy(&self) -> Result<Policy> {
        let mut builder = Policy::builder();

        if let Some(span) = self.older_than {
            builder = builder.old_threshold(span);
        }

        if let Some(bytes) = self.min_size {
            builder = builder.size_threshold(bytes);
        }

        builder.build()
    }
}

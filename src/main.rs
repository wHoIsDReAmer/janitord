use janitord::{
    cli::{Cli, Command, ScanArgs},
    core::{Element, policy::Policy, scan::scan},
};

use anyhow::Result;
use clap::Parser;
use jiff::ZonedDateTime;

use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Scan(args) => run_scan(&args)?,
    }

    Ok(())
}

fn run_scan(args: &ScanArgs) -> Result<()> {
    let found = scan(&args.paths)?;
    let targets = if args.has_filter() {
        matching(found, &args.policy()?)?
    } else {
        found
    };

    if !args.clean {
        for element in &targets {
            println!(
                "{:>14}  {:<6?}  {}",
                element.size,
                element.kind,
                element.path.display()
            );
        }
        return Ok(());
    }

    let results: Vec<(&Element, Result<bool>)> = targets
        .par_iter()
        .map(|element| (element, element.delete()))
        .collect();

    for (element, result) in results {
        match result {
            Ok(true) => println!("deleted  {}", element.path.display()),
            Ok(false) => println!("skipped  {} (in use or changed)", element.path.display()),
            Err(err) => eprintln!("failed   {}: {err:#}", element.path.display()),
        }
    }

    Ok(())
}

fn matching(found: Vec<Element>, policy: &Policy) -> Result<Vec<Element>> {
    let now = ZonedDateTime::now();
    let mut targets = Vec::new();

    for element in found {
        if element.deletable(policy, &now)? {
            targets.push(element);
        }
    }

    Ok(targets)
}

mod bookkeeper;
mod cli;
mod error;
mod fs_utils;
mod parser;

use chrono::{Datelike, Local};
use clap::Parser;

use crate::{
    bookkeeper::{Bookkeeper, StatusInfo},
    cli::{Opts, Subcommand},
    error::{Error, Result},
    fs_utils::current_file,
    parser::{Operation, OperationType},
};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(127);
    }
}

fn run() -> Result<()> {
    let cmd = Opts::parse().cmd;
    let day = Local::today().day() as u8;

    match cmd {
        Subcommand::Take {
            amount,
            ref description,
        } => {
            let operation = Operation::new(day, OperationType::Withdraw, amount, description);
            Bookkeeper::new_current()?.add_operation(operation)?;
        }
        Subcommand::Put {
            amount,
            ref description,
        } => {
            let operation = Operation::new(day, OperationType::Deposit, amount, description);
            Bookkeeper::new_current()?.add_operation(operation)?;
        }
        Subcommand::Status { target } => {
            let target = target.unwrap_or_else(|| "current".into());

            match target.as_str() {
                "all" => {
                    for bookkeeper in Bookkeeper::new_all()? {
                        bookkeeper.display_status(StatusInfo::Summary);
                    }
                }
                "current" => Bookkeeper::new_current()?.display_status(StatusInfo::Complete),
                _ => unreachable!(),
            }
        }
    };

    Ok(())
}

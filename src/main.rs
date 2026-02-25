mod cli;
mod commands;
mod config;
mod copy;
mod git;
mod tmux;

use clap::Parser;
use cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Activate { branch_name } => commands::activate::run(&branch_name),
        Command::Create { branch_name } => commands::create::run(&branch_name),
        Command::Teardown { force } => commands::teardown::run(force),
        Command::List => commands::list::run(),
    }
}

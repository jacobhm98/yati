mod cli;
mod commands;
mod completions;
mod config;
mod copy;
mod git;
mod tmux;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;
use cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    CompleteEnv::with_factory(|| Cli::command()).complete();
    let cli = Cli::parse();
    match cli.command {
        Command::Activate { target } => commands::activate::run(&target),
        Command::Create { branch_name } => commands::create::run(&branch_name),
        Command::Deactivate => commands::deactivate::run(),
        Command::Teardown { force } => commands::teardown::run(force),
        Command::List => commands::list::run(),
    }
}

mod cli;
mod commands;
mod completions;
mod config;
mod copy;
mod git;
mod tmux;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;
use cli::{Cli, Command, GenerateKind};

fn main() -> anyhow::Result<()> {
    CompleteEnv::with_factory(|| Cli::command()).complete();
    let cli = Cli::parse();

    if let Some(kind) = cli.generate {
        match kind {
            GenerateKind::Man => {
                let cmd = Cli::command();
                let man = clap_mangen::Man::new(cmd);
                man.render(&mut std::io::stdout())?;
            }
        }
        return Ok(());
    }

    let command = cli.command.unwrap_or_else(|| {
        Cli::command().print_help().ok();
        std::process::exit(0);
    });

    match command {
        Command::Activate { target } => commands::activate::run(&target),
        Command::Create { branch_name, index } => commands::create::run(&branch_name, index),
        Command::Deactivate => commands::deactivate::run(),
        Command::Teardown { force } => commands::teardown::run(force),
        Command::List => commands::list::run(),
    }
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yati", about = "Git worktree manager with tmux integration")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new worktree and branch
    Create {
        /// Name of the branch to create
        branch_name: String,
    },
    /// Tear down the current yati worktree
    Teardown {
        /// Force removal even with uncommitted changes
        #[arg(long)]
        force: bool,
    },
    /// List active yati-managed worktrees for the current project
    List,
}

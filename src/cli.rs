use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::engine::ArgValueCompleter;

#[derive(Parser)]
#[command(
    name = "yati",
    about = "Git worktree manager with tmux and docker-compose integration",
    long_about = "yati manages git worktrees with integrated tmux sessions, docker-compose port \
        isolation, and lifecycle hooks.\n\n\
        Each worktree gets its own tmux session and a unique index (0, 1, 2...). \
        Port environment variables are computed as: base + index * offset, so concurrent \
        worktrees automatically get non-colliding host ports for docker-compose services.\n\n\
        Worktrees are stored under ~/.yati/<project>/<branch>/. \
        Configuration is read from yati.toml at the repository root.",
    after_help = "ENVIRONMENT VARIABLES (injected into tmux sessions):\n  \
        COMPOSE_PROJECT_NAME  Auto-sanitized <project>-<branch> (unless overridden)\n  \
        YATI_PORT_OFFSET      index * ports.offset (for scripting)\n  \
        <PORT_NAME>           base + index * offset (one per [ports] entry)\n  \
        <custom>              From [environment] in yati.toml (supports {{project}}/{{branch}})\n\n\
        CONFIGURATION:\n  \
        Place a yati.toml in your repository root. See example.yati.toml or the README.\n\n\
        More information: https://github.com/jacobhm98/yati"
)]
pub struct Cli {
    /// Generate shell completions or man page and print to stdout
    #[arg(long, value_name = "KIND")]
    pub generate: Option<GenerateKind>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, ValueEnum)]
pub enum GenerateKind {
    Man,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new worktree and branch
    #[command(
        long_about = "Create a new worktree and branch.\n\n\
            Creates a git worktree at ~/.yati/<project>/<branch>/, assigns it an index \
            (auto-incremented or set with --index), and opens it in a new tmux session \
            named <project>/<branch>.\n\n\
            PORT ISOLATION:\n  \
            Each worktree's index determines its port offsets. With [ports] config:\n    \
            offset = 100\n    \
            WEB_PORT = 3000\n  \
            Index 0 → WEB_PORT=3000, Index 1 → WEB_PORT=3100, Index 2 → WEB_PORT=3200\n  \
            These are set as environment variables in the tmux session so docker-compose \
            services pick them up via ${WEB_PORT} references.\n\n\
            HOOKS:\n  \
            post_create hooks run after worktree creation. Plain strings run synchronously \
            before the tmux session attaches. Hooks with async=true run in a background \
            tmux 'setup' window."
    )]
    Create {
        /// Name of the branch to create
        #[arg(add = ArgValueCompleter::new(crate::completions::complete_create_branch))]
        branch_name: String,
        /// Set the session index to a specific value (must not already be in use).
        /// The index determines port offsets: port = base + index * offset.
        #[arg(long)]
        index: Option<u32>,
        /// Profile from [profiles.<name>] in yati.toml to layer over the base config.
        #[arg(long, add = ArgValueCompleter::new(crate::completions::complete_create_profile))]
        profile: Option<String>,
    },
    /// Tear down the current yati worktree
    #[command(
        long_about = "Tear down the current yati worktree.\n\n\
            Removes the git worktree, kills the tmux session, and deletes the branch. \
            Runs pre_teardown hooks before cleanup. Refuses to proceed if the worktree \
            has uncommitted changes unless --force is passed."
    )]
    Teardown {
        /// Force removal even with uncommitted changes
        #[arg(long)]
        force: bool,
    },
    /// Activate an existing worktree by attaching to or creating its tmux session
    Activate {
        /// Branch name or project/branch to activate
        #[arg(add = ArgValueCompleter::new(crate::completions::complete_activate_target))]
        target: String,
    },
    /// Deactivate the current yati session (switch to previous session or detach)
    Deactivate,
    /// List all yati-managed worktrees across all projects
    List,
    /// Write a starter yati.toml to the repository root
    #[command(
        long_about = "Write a starter yati.toml to the repository root.\n\n\
            Creates a yati.toml at the current repository's root, pre-filled with the \
            documented example configuration (copy_files, hooks, [tmux] windows, [ports], \
            [environment], and [profiles]). Refuses to overwrite an existing yati.toml."
    )]
    Init,
}

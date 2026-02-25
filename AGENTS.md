# AGENTS.md

## What is yati?

yati is a CLI tool that manages git worktrees with tmux session integration. It creates worktrees under `~/.yati/<project>/<branch>`, opens them in dedicated tmux sessions, and tears them down cleanly.

## Project structure

```
src/
  main.rs          — entry point, dispatches CLI commands
  cli.rs           — clap-based CLI definition (subcommands: create, teardown, list)
  tmux.rs          — tmux session operations (new_session, kill_session)
  git.rs           — git operations (worktree add/remove/list, repo info, branch validation)
  config.rs        — loads yati.toml configuration
  copy.rs          — recursive file copy with glob-based exclusion
  commands/
    mod.rs         — re-exports command modules
    create.rs      — `yati create <branch>`: creates worktree + tmux session
    teardown.rs    — `yati teardown`: removes worktree + kills tmux session
    list.rs        — `yati list`: lists yati-managed worktrees for the current project
```

## Configuration

Projects can include a `yati.toml` at the repo root with:

- `copy_files` — files/directories to copy from the main worktree into new worktrees
- `exclude` — glob patterns to exclude from copying
- `post_create` — shell commands to run after creating a worktree
- `pre_teardown` — shell commands to run before tearing down a worktree

## Build and test

```sh
cargo build
cargo test
```

There are currently no automated tests. Manual testing:

1. Inside a tmux session and a git repo, run `yati create test-branch`
2. Verify a new tmux session named `<project>/test-branch` is created
3. In the new session, run `yati teardown` to clean up

## Key conventions

- Error handling uses `anyhow` with `.context()` for descriptive error messages
- All external commands (git, tmux) are run via `std::process::Command`
- Worktrees are stored under `~/.yati/<project>/<branch>`
- tmux sessions are named `<project>/<branch>`

## After making changes

- Make sure the README.md is still up to date with the changes you just made.

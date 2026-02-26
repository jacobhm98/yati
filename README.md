# yati

_yāti_ (याति) — Sanskrit for "travels" or "goes forth." In the Rigveda, it describes the journeying of gods between realms. _yāti_ lets you travel between worktrees seamlessly, each one a self-contained world.

### What does it do?

It is a git worktree manager with tmux integration. Creates and manages isolated worktrees for feature branches, each in its own tmux session. For when you want to vibe code multiple branches of the same repository simultaneously but still keep your hand in the code using your regular tmux workflows.

Heavily inspired by [opencode-worktree](https://github.com/kdcokenny/opencode-worktree), but agent agnostic.

## Installation

```sh
git clone git@github.com:jacobhm98/yati.git
cd yati
cargo install --path .
```

## Usage

### Create a worktree

```sh
yati create feature-branch
```

This will:

1. Create a new git worktree at `~/.yati/<project>/feature-branch`
2. Create a new branch `feature-branch` (or use an existing one)
3. Copy any configured files into the worktree
4. Run any `post_create` hooks
5. Open a new tmux session named `<project>/feature-branch`

### Activate a worktree

Switch to an existing worktree:

```sh
yati activate project/feature-branch
```

or, if ran from project root

```sh
yati activate feature-branch
```

This will:

1. Check that a worktree exists at `~/.yati/<project>/feature-branch`
2. If a tmux session already exists, switch to it
3. If the session was lost (e.g., after a reboot), run `post_create` hooks and create a new tmux session
4. Run `post_activate` hooks (runs on every activate, whether the session existed or was recreated)

### Deactivate a worktree

From inside a yati-managed worktree:

```sh
yati deactivate
```

This leaves the current session without destroying it. If you switched from another tmux session, you'll be returned there. If you attached from a bare terminal, you'll be detached from tmux. The session stays alive for later reactivation with `yati activate`.

### Tear down a worktree

From inside a yati-managed worktree:

```sh
yati teardown
```

This runs `pre_teardown` hooks, removes the worktree, deletes the branch, and kills the tmux session. If you came from another session you'll be switched back; otherwise you'll be returned to your original terminal.

Use `--force` to remove even with uncommitted changes:

```sh
yati teardown --force
```

### List worktrees

```sh
yati list
```

Shows all yati-managed worktrees across all projects.

## Shell Completions

yati supports dynamic shell completions for subcommands, flags, worktree targets, and branch names. Run the appropriate setup for your shell once:

### Fish

```fish
echo 'source (COMPLETE=fish yati | psub)' >> ~/.config/fish/completions/yati.fish
```

### Bash

```bash
echo 'source <(COMPLETE=bash yati)' >> ~/.bashrc
```

### Zsh

```zsh
echo 'source <(COMPLETE=zsh yati)' >> ~/.zshrc
```

After restarting your shell (or sourcing the file), `yati <TAB>` will complete subcommands, `yati activate <TAB>` will complete with existing worktree targets, and `yati create <TAB>` will complete with git branch names.

## Configuration

Create a `yati.toml` in your repository root:

```toml
# Files or directories to copy from the main worktree into new worktrees
copy_files = [".env", "node_modules"]

# Patterns to exclude when copying
exclude = ["*.log"]

# Commands to run after creating a worktree
post_create = ["npm install"]

# Commands to run every time a worktree is activated (including after creation)
post_activate = ["docker compose up -d"]

# Commands to run before tearing down a worktree
pre_teardown = ["docker compose down"]

# Tmux windows to create in the session.
# The first window replaces the default window; additional entries create new windows.
[tmux]
windows = [
  { name = "editor", command = "nvim" },
  { name = "server", command = "npm run dev" },
  { name = "claude --continue" },
]
```

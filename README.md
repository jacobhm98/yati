# yati

A git worktree manager with tmux integration. Create isolated worktrees for feature branches, each in its own tmux session.

## Installation

```sh
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

If you're not inside tmux, the session is created detached and you can attach with:

```sh
tmux attach -t '<project>/feature-branch'
```

### Tear down a worktree

From inside a yati-managed worktree:

```sh
yati teardown
```

This removes the worktree, runs `pre_teardown` hooks, and kills the tmux session.

Use `--force` to remove even with uncommitted changes:

```sh
yati teardown --force
```

### List worktrees

```sh
yati list
```

Shows all yati-managed worktrees for the current project.

## Configuration

Create a `yati.toml` in your repository root:

```toml
# Files or directories to copy from the main worktree into new worktrees
copy_files = [".env", "node_modules"]

# Patterns to exclude when copying
exclude = ["*.log"]

# Commands to run after creating a worktree
post_create = ["npm install"]

# Commands to run before tearing down a worktree
pre_teardown = ["docker compose down"]
```

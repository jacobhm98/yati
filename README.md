# yati

_yāti_ (याति) — Sanskrit for "travels" or "goes forth." In the Rigveda, it describes the journeying of gods between realms. _yāti_ lets you travel between worktrees seamlessly, each one a self-contained world.

## What does it do?

It is a git worktree manager with tmux and docker-compose integration. Creates and manages isolated worktrees and environments for feature branches. For when you want to vibe code multiple branches of the same repository simultaneously but maintain proximity to the code.

### Dev environment lifecycle management

yati manages the full lifecycle of a worktree: file copying, `post_create`/`post_activate`/`pre_teardown` hooks, tmux session layout, and per-worktree docker-compose isolation via automatic port offsetting (`[ports]`) and environment variable injection (`[environment]`).

Inspired by [opencode-worktree](https://github.com/kdcokenny/opencode-worktree)

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

You can optionally specify a session index with `--index`. This controls port allocation and must not already be in use:

```sh
yati create --index 3 feature-branch
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

## Docker-Compose Isolation

When you run multiple worktrees of the same project, `docker compose up` in each one will clash — container names, networks, volumes, and host-port bindings all collide. yati solves this by automatically injecting environment variables into every tmux pane of a worktree session.

### What yati does automatically

1. **`COMPOSE_PROJECT_NAME`** is set to `<project>-<branch>` (sanitized for Docker). This isolates container names, networks, and volumes per worktree — no config needed.

2. **`[ports]` table** — each worktree is assigned an auto-incrementing index (0, 1, 2, ...). For each entry in `[ports]`, the env var is set to `base + index * offset`. This gives every worktree unique host ports.

3. **`[environment]` table** — arbitrary env vars with `{{project}}` and `{{branch}}` template support, expanded per worktree.

yati also sets **`YATI_PORT_OFFSET`** to `index * offset` for use in custom scripts.

### Index allocation

Each worktree gets the lowest available index starting at 0. The index is persisted in a `.yati_index` file inside the worktree directory and is freed when the worktree is torn down.

### Using the variables in docker-compose.yml

Reference the env vars in your `docker-compose.yml` with fallback defaults so it still works outside yati:

```yaml
services:
  db:
    ports:
      - "${DB_PORT:-5432}:5432"
  web:
    ports:
      - "${WEB_PORT:-3000}:3000"
```

### Example

With this config:

```toml
[ports]
offset = 100
DB_PORT = 5432
WEB_PORT = 3000
```

- Worktree index 0: `DB_PORT=5432`, `WEB_PORT=3000`
- Worktree index 1: `DB_PORT=5532`, `WEB_PORT=3100`
- Worktree index 2: `DB_PORT=5632`, `WEB_PORT=3200`

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
copy_files = [".env"]

# Patterns to exclude when copying
exclude = ["*.log"]

# Commands to run after creating a worktree.
# Plain strings run synchronously before the tmux session is created.
# Use { command = "...", async = true } to run in a background tmux window.
post_create = [
  "echo setting up",
  { command = "npm install", async = true },
]

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
  { name = "claude" },
]

# Per-worktree port isolation for docker-compose.
# Each worktree gets an index (0, 1, 2...); port = base + index * offset.
[ports]
offset = 100
DB_PORT = 5432
WEB_PORT = 3000

# Arbitrary env vars injected into the tmux session.
# Supports {{project}} and {{branch}} templates.
[environment]
MY_SERVICE_ID = "{{project}}-{{branch}}"
```

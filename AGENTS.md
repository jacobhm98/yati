# AGENTS.md

## What is yati?

yati is a CLI tool that manages git worktrees with tmux session integration and docker-compose port isolation. It creates worktrees under `~/.yati/<project>/<branch>`, opens them in dedicated tmux sessions, and tears them down cleanly.

## Using yati

### Commands

| Command | Purpose |
|---------|---------|
| `yati create <branch> [--index N]` | Create a worktree + tmux session for a branch |
| `yati activate <target>` | Switch to an existing worktree (target: branch or project/branch) |
| `yati deactivate` | Leave current session without destroying it |
| `yati teardown [--force]` | Remove worktree, kill tmux session, delete branch |
| `yati list` | List all yati-managed worktrees across all projects |
| `yati --generate man` | Print man page to stdout |

### Port isolation (docker-compose)

Each worktree gets an auto-assigned **index** (0, 1, 2...) stored in `.yati_index`. The `[ports]` section in `yati.toml` defines base port values and an offset:

```toml
[ports]
offset = 100
DB_PORT = 5432
WEB_PORT = 3000
```

Port env vars are computed as **base + index * offset**:

| Worktree index | DB_PORT | WEB_PORT |
|----------------|---------|----------|
| 0 | 5432 | 3000 |
| 1 | 5532 | 3100 |
| 2 | 5632 | 3200 |

These are injected as environment variables into the tmux session, so `docker-compose.yml` can reference them as `${DB_PORT}`, `${WEB_PORT}`, etc. This means multiple worktrees can run docker-compose services simultaneously without port collisions.

### Environment variables injected into tmux sessions

- `COMPOSE_PROJECT_NAME` — auto-sanitized `<project>-<branch>` (prevents docker container name collisions)
- `YATI_PORT_OFFSET` — `index * offset` (useful for scripting)
- Port variables — one per `[ports]` entry (e.g. `DB_PORT`, `WEB_PORT`)
- Custom variables — from `[environment]` section, supports `{{project}}` and `{{branch}}` templates

### Configuration (yati.toml)

Place a `yati.toml` at the repository root. Sections:

- `copy_files` — files to copy from main worktree (e.g. `.env`, secrets)
- `exclude` — glob patterns to skip during copy
- `post_create` — hooks after worktree creation (plain string = sync, `{command, async=true}` = async in tmux window)
- `post_activate` — hooks on every activation (including after creation)
- `pre_teardown` — hooks before teardown
- `[tmux]` — window definitions (`name`, optional `command`)
- `[ports]` — port isolation config (`offset` + base port variables)
- `[environment]` — custom env vars with `{{project}}`/`{{branch}}` templates

See `example.yati.toml` for a complete reference.

## Project structure

```
src/
  main.rs          — entry point, dispatches CLI commands
  cli.rs           — clap-based CLI definition (subcommands: create, activate, deactivate, teardown, list)
  tmux.rs          — tmux session operations (new_session, kill_session, attach_or_switch, detach, etc.)
  git.rs           — git operations (worktree add/remove/list, repo info, branch validation)
  config.rs        — loads yati.toml configuration
  copy.rs          — recursive file copy with glob-based exclusion
  commands/
    mod.rs         — re-exports command modules
    create.rs      — `yati create <branch>`: creates worktree + tmux session
    activate.rs    — `yati activate <target>`: re-attaches to existing worktree session
    deactivate.rs  — `yati deactivate`: leaves current session (switches back or detaches)
    teardown.rs    — `yati teardown`: removes worktree + kills tmux session
    list.rs        — `yati list`: lists yati-managed worktrees across all projects
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

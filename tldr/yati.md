# yati

> Git worktree manager with tmux sessions and docker-compose port isolation.
> Each worktree gets a unique index; port env vars are computed as base + index * offset.
> More information: <https://github.com/jacobhm98/yati>.

- Create a new worktree and tmux session for a branch:

`yati create {{branch_name}}`

- Create a worktree with a specific index (determines port offsets):

`yati create --index {{2}} {{branch_name}}`

- Switch to an existing worktree's tmux session:

`yati activate {{branch_name}}`

- Switch to a worktree in a specific project:

`yati activate {{project/branch_name}}`

- Leave the current worktree session without destroying it:

`yati deactivate`

- Tear down the current worktree, branch, and tmux session:

`yati teardown`

- Force teardown even with uncommitted changes:

`yati teardown --force`

- List all yati-managed worktrees:

`yati list`

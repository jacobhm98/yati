use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::fs;
use std::process::Command;

use crate::{config, copy, git, tmux};

fn allocate_index(project_name: &str, requested: Option<u32>) -> Result<u32> {
    let yati_base = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".yati")
        .join(project_name);
    let mut used: HashSet<u32> = HashSet::new();
    if let Ok(entries) = fs::read_dir(&yati_base) {
        for entry in entries.flatten() {
            let index_file = entry.path().join(".yati_index");
            if let Ok(contents) = fs::read_to_string(&index_file) {
                if let Ok(idx) = contents.lines().next().unwrap_or("").trim().parse::<u32>() {
                    used.insert(idx);
                }
            }
        }
    }
    match requested {
        Some(idx) => {
            if used.contains(&idx) {
                bail!("Index {} is already in use", idx);
            }
            Ok(idx)
        }
        None => {
            let mut index = 0;
            while used.contains(&index) {
                index += 1;
            }
            Ok(index)
        }
    }
}

pub fn run(branch_name: &str, requested_index: Option<u32>) -> Result<()> {
    let repo_root = git::main_worktree_root()?;
    let project_name = git::main_repo_name()?;
    git::validate_branch_name(branch_name)?;

    let config = config::load_config(&repo_root)?;

    let yati_base = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".yati")
        .join(&project_name);
    let worktree_path = yati_base.join(branch_name);

    if worktree_path.exists() {
        bail!(
            "Worktree path already exists: {}",
            worktree_path.display()
        );
    }

    println!("Creating worktree at {}", worktree_path.display());
    git::worktree_add(&worktree_path, branch_name)?;

    if !config.copy_files.is_empty() {
        println!("Copying configured files...");
        copy::copy_files(&repo_root, &worktree_path, &config.copy_files, &config.exclude)?;
    }

    let (sync_hooks, async_hooks): (Vec<_>, Vec<_>) = config
        .post_create
        .iter()
        .partition(|h| !h.is_async());

    for hook in &sync_hooks {
        let cmd = hook.command();
        println!("Running post_create hook: {}", cmd);
        let status = Command::new("sh")
            .args(["-c", cmd])
            .current_dir(&worktree_path)
            .status()
            .with_context(|| format!("Failed to run hook: {}", cmd))?;
        if !status.success() {
            eprintln!("Warning: post_create hook failed: {}", cmd);
        }
    }

    let session_name = format!("{}/{}", project_name, branch_name);

    let index = allocate_index(&project_name, requested_index)?;
    let mut index_contents = index.to_string();
    let offset = index * config.ports.offset as u32;
    for (name, base) in &config.ports.ports {
        let value = *base as u32 + offset;
        index_contents.push_str(&format!("\n{}={}", name, value));
    }
    fs::write(worktree_path.join(".yati_index"), index_contents)?;

    println!("Creating tmux session '{}'", session_name);
    tmux::new_session(&session_name, &worktree_path)?;
    tmux::setup_environment(&session_name, &config, &project_name, branch_name, index)?;
    tmux::setup_windows(&session_name, &worktree_path, &config.tmux.windows)?;

    if !async_hooks.is_empty() {
        let cmds: Vec<&str> = async_hooks.iter().map(|h| h.command()).collect();
        let hook_cmd = tmux::build_hook_command(&cmds);
        println!("Running async post_create hooks in 'setup' window");
        tmux::create_command_window(&session_name, "setup", &worktree_path, &hook_cmd)?;
    }

    tmux::attach_or_switch(&session_name)?;

    Ok(())
}

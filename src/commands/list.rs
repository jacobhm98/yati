use anyhow::{Context, Result};

use crate::git;

pub fn run() -> Result<()> {
    let project_name = git::repo_name()?;
    let yati_base = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".yati")
        .join(&project_name);

    let entries = git::worktree_list()?;
    let yati_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.path.starts_with(&yati_base))
        .collect();

    if yati_entries.is_empty() {
        println!("No yati-managed worktrees for project '{}'", project_name);
        return Ok(());
    }

    println!(
        "Yati worktrees for '{}':\n",
        project_name
    );
    for entry in &yati_entries {
        let short_head = if entry.head.len() >= 7 {
            &entry.head[..7]
        } else {
            &entry.head
        };
        println!("  {} ({})", entry.branch, short_head);
        println!("    {}", entry.path.display());
    }

    Ok(())
}

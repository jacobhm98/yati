use anyhow::{Context, Result};
use std::path::Path;

fn matches_exclude(relative_path: &str, exclude: &[String]) -> bool {
    for pattern in exclude {
        // Simple glob matching: support *.ext patterns
        if let Some(suffix) = pattern.strip_prefix('*') {
            if relative_path.ends_with(suffix) {
                return true;
            }
        } else if relative_path == pattern || relative_path.ends_with(&format!("/{}", pattern)) {
            return true;
        }
    }
    false
}

fn copy_recursive(src: &Path, dst: &Path, base_src: &Path, exclude: &[String]) -> Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)
            .with_context(|| format!("Failed to create directory {}", dst.display()))?;
        for entry in
            std::fs::read_dir(src).with_context(|| format!("Failed to read {}", src.display()))?
        {
            let entry = entry?;
            let child_src = entry.path();
            let rel = child_src
                .strip_prefix(base_src)
                .unwrap_or(&child_src)
                .to_string_lossy();
            if matches_exclude(&rel, exclude) {
                continue;
            }
            let child_dst = dst.join(entry.file_name());
            copy_recursive(&child_src, &child_dst, base_src, exclude)?;
        }
    } else {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        std::fs::copy(src, dst)
            .with_context(|| format!("Failed to copy {} to {}", src.display(), dst.display()))?;
    }
    Ok(())
}

pub fn copy_files(
    src_root: &Path,
    dst_root: &Path,
    entries: &[String],
    exclude: &[String],
) -> Result<()> {
    for entry in entries {
        let src = src_root.join(entry);
        let dst = dst_root.join(entry);
        if !src.exists() {
            eprintln!("Warning: {} does not exist, skipping", src.display());
            continue;
        }
        let rel = src
            .strip_prefix(src_root)
            .unwrap_or(&src)
            .to_string_lossy();
        if matches_exclude(&rel, exclude) {
            continue;
        }
        copy_recursive(&src, &dst, src_root, exclude)?;
    }
    Ok(())
}

use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::Path;

use crate::asset::Asset;
use crate::audit::format_bytes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub deleted_count: usize,
    pub deleted_bytes: u64,
    pub deleted_bytes_formatted: String,
    pub failed_files: Vec<(String, String)>,
    pub pruned_directories: usize,
    pub is_dry_run: bool,
}

/// Recursively removes empty parent directories starting from `dir` up towards `stop_at`.
/// Never removes `stop_at` itself.
fn prune_empty_parents(mut dir: &Path, stop_at: &Path) -> usize {
    let mut pruned = 0;
    while let Some(parent) = dir.parent() {
        if dir == stop_at || !dir.starts_with(stop_at) {
            break;
        }

        // Check if directory is empty
        match std::fs::read_dir(dir) {
            Ok(mut entries) => {
                if entries.next().is_none() && std::fs::remove_dir(dir).is_ok() {
                    pruned += 1;
                    dir = parent;
                    continue;
                }
                break;
            }
            Err(_) => break,
        }
    }
    pruned
}

pub fn execute_cleanup(
    root: &Path,
    dead_assets: &[Asset],
    dry_run: bool,
    force: bool,
) -> io::Result<CleanupResult> {
    let root = &std::fs::canonicalize(root)?;
    // Validate the complete plan before removing anything, including during previews.
    for asset in dead_assets {
        let resolved = std::fs::canonicalize(&asset.abs_path)?;
        if !resolved.starts_with(root) || !std::fs::symlink_metadata(&asset.abs_path)?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cleanup target must be a regular file inside the project",
            ));
        }
    }
    let total_reclaimable: u64 = dead_assets.iter().map(|a| a.size_bytes).sum();

    if dead_assets.is_empty() {
        return Ok(CleanupResult {
            deleted_count: 0,
            deleted_bytes: 0,
            deleted_bytes_formatted: "0 B".to_string(),
            failed_files: Vec::new(),
            pruned_directories: 0,
            is_dry_run: dry_run,
        });
    }

    if dry_run {
        return Ok(CleanupResult {
            deleted_count: dead_assets.len(),
            deleted_bytes: total_reclaimable,
            deleted_bytes_formatted: format_bytes(total_reclaimable),
            failed_files: Vec::new(),
            pruned_directories: 0,
            is_dry_run: true,
        });
    }

    if !force {
        eprint!(
            "\n⚠️  Are you sure you want to permanently delete {} dead assets (reclaiming {})? [y/N]: ",
            dead_assets.len(),
            format_bytes(total_reclaimable)
        );
        io::stderr().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if !trimmed.eq_ignore_ascii_case("y") && !trimmed.eq_ignore_ascii_case("yes") {
            eprintln!("Cleanup canceled. No files were removed.");
            return Ok(CleanupResult {
                deleted_count: 0,
                deleted_bytes: 0,
                deleted_bytes_formatted: "0 B".to_string(),
                failed_files: Vec::new(),
                pruned_directories: 0,
                is_dry_run: false,
            });
        }
    }

    let mut deleted_count = 0;
    let mut deleted_bytes = 0;
    let mut failed_files = Vec::new();
    let mut pruned_dirs = 0;

    for asset in dead_assets {
        if !asset.abs_path.exists() {
            continue;
        }

        match std::fs::remove_file(&asset.abs_path) {
            Ok(_) => {
                deleted_count += 1;
                deleted_bytes += asset.size_bytes;

                // Try pruning empty parent directories
                if let Some(parent) = asset.abs_path.parent() {
                    pruned_dirs += prune_empty_parents(parent, root);
                }
            }
            Err(e) => {
                failed_files.push((asset.rel_path.clone(), e.to_string()));
            }
        }
    }

    Ok(CleanupResult {
        deleted_count,
        deleted_bytes,
        deleted_bytes_formatted: format_bytes(deleted_bytes),
        failed_files,
        pruned_directories: pruned_dirs,
        is_dry_run: false,
    })
}

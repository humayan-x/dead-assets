use owo_colors::OwoColorize;
use serde::Serialize;

use crate::asset::Asset;
use crate::audit::StorageAudit;
use crate::cleaner::CleanupResult;
use crate::framework::Framework;

#[derive(Debug, Serialize)]
pub struct JsonReport<'a> {
    pub framework: &'static str,
    pub audit: &'a StorageAudit,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup: Option<&'a CleanupResult>,
}

pub fn print_terminal_report(
    framework: Framework,
    audit: &StorageAudit,
    all_assets: &[Asset],
    verbose: bool,
    cleanup_result: Option<&CleanupResult>,
) {
    println!();
    println!(
        "{} {}",
        " dead-assets ".on_black().bold().bright_green(),
        "Static Media Asset Auditor".bold()
    );
    println!(
        "{} {}",
        "Framework:".dimmed(),
        framework.display_name().cyan().bold()
    );
    println!();

    if audit.dead_assets == 0 {
        println!(
            "{}",
            "✨ Clean codebase! All scanned assets are actively referenced."
                .green()
                .bold()
        );
        println!(
            "Scanned {} assets ({}) with zero bloat.",
            audit.total_assets.bright_cyan(),
            audit.total_bytes_formatted.bright_cyan()
        );
        println!();
        return;
    }

    // List dead assets
    println!(
        "{} {} {}:",
        "Found".bold(),
        audit.dead_assets.to_string().red().bold(),
        "dead or unreferenced assets".bold()
    );
    println!("{}", "─".repeat(68).dimmed());

    for asset in &audit.dead_asset_list {
        let kind_badge = match asset.kind {
            crate::asset::AssetKind::Image => format!("[{}]", asset.kind.display_label())
                .yellow()
                .to_string(),
            crate::asset::AssetKind::Svg => format!("[{}]", asset.kind.display_label())
                .magenta()
                .to_string(),
            crate::asset::AssetKind::Font => format!("[{}]", asset.kind.display_label())
                .blue()
                .to_string(),
            crate::asset::AssetKind::Media => format!("[{}]", asset.kind.display_label())
                .bright_red()
                .to_string(),
            crate::asset::AssetKind::Other => format!("[{}]", asset.kind.display_label())
                .dimmed()
                .to_string(),
        };

        println!(
            " {:<8} {:<46} {:>10}",
            kind_badge,
            asset.rel_path.white(),
            crate::audit::format_bytes(asset.size_bytes).bright_red()
        );
    }
    println!("{}", "─".repeat(68).dimmed());

    if verbose {
        println!();
        println!("{}", "Active / Referenced Assets:".green().bold());
        for asset in all_assets.iter().filter(|a| a.is_referenced) {
            let refs = if !asset.referenced_by.is_empty() {
                format!("(referenced by: {})", asset.referenced_by.join(", "))
            } else {
                String::new()
            };
            println!(
                "  {} {:<40} {:>10} {}",
                "✓".green(),
                asset.rel_path,
                crate::audit::format_bytes(asset.size_bytes).dimmed(),
                refs.dimmed()
            );
        }
        println!();
    }

    // Storage Audit Summary Box
    println!();
    println!(
        "{}",
        "┌────────────────────────────────────────────────────────┐".dimmed()
    );
    println!(
        "{}  {:<52} {}",
        "│".dimmed(),
        "Storage Audit Summary".bold(),
        "│".dimmed()
    );
    println!(
        "{}",
        "├────────────────────────────────────────────────────────┤".dimmed()
    );
    println!(
        "{}  {:<26} {:>24}  {}",
        "│".dimmed(),
        "Total Assets Scanned:",
        format!("{} ({})", audit.total_assets, audit.total_bytes_formatted),
        "│".dimmed()
    );
    println!(
        "{}  {:<26} {:>24}  {}",
        "│".dimmed(),
        "Active References:",
        format!(
            "{} ({})",
            audit.referenced_assets,
            crate::audit::format_bytes(audit.referenced_bytes)
        ),
        "│".dimmed()
    );
    println!(
        "{}  {:<26} {:>24}  {}",
        "│".dimmed(),
        "Dead Assets:".red(),
        format!("{} assets", audit.dead_assets).red(),
        "│".dimmed()
    );
    println!(
        "{}  {:<26} {:>24}  {}",
        "│".dimmed(),
        "Reclaimable Storage:".bright_yellow().bold(),
        format!(
            "{} ({:.1}% trim)",
            audit.reclaimable_bytes_formatted, audit.reclaimable_percent
        )
        .bright_green()
        .bold(),
        "│".dimmed()
    );
    println!(
        "{}",
        "└────────────────────────────────────────────────────────┘".dimmed()
    );
    println!();

    // Category breakdown
    print!("Category Breakdown: ");
    let mut parts = Vec::new();
    for (kind, stat) in &audit.by_kind {
        if stat.dead_count > 0 {
            parts.push(format!(
                "{}: {} dead ({})",
                kind,
                stat.dead_count,
                crate::audit::format_bytes(stat.dead_bytes)
            ));
        }
    }
    println!("{}", parts.join("  |  ").dimmed());
    println!();

    if let Some(clean) = cleanup_result {
        for (path, error) in &clean.failed_files {
            eprintln!("Failed to delete {path}: {error}");
        }
        if clean.is_dry_run {
            println!(
                "{}",
                format!(
                    "🔍 Dry-run: {} assets ({}) would be safely deleted.",
                    clean.deleted_count, clean.deleted_bytes_formatted
                )
                .yellow()
                .bold()
            );
        } else if clean.deleted_count > 0 {
            println!(
                "{}",
                format!(
                    "🗑️  Cleaned up {} dead assets! Reclaimed {}.",
                    clean.deleted_count, clean.deleted_bytes_formatted
                )
                .green()
                .bold()
            );
            if clean.pruned_directories > 0 {
                println!(
                    "{}",
                    format!(
                        "📁 Removed {} empty parent directories.",
                        clean.pruned_directories
                    )
                    .dimmed()
                );
            }
        }
    } else {
        println!(
            "💡 Tip: Run {} to automatically prune these unreferenced files.",
            "dead-assets --clean".cyan().bold()
        );
    }
    println!();
}

pub fn print_json_report(
    framework: Framework,
    audit: &StorageAudit,
    cleanup_result: Option<&CleanupResult>,
) {
    let report = JsonReport {
        framework: framework.display_name(),
        audit,
        cleanup: cleanup_result,
    };

    if let Ok(json) = serde_json::to_string_pretty(&report) {
        println!("{}", json);
    }
}

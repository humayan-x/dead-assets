use std::collections::HashSet;
use std::path::PathBuf;

use bpaf::Bpaf;

use dead_assets::asset::discover_assets;
use dead_assets::audit::{StorageAudit, parse_size_threshold};
use dead_assets::cleaner::execute_cleanup;
use dead_assets::framework::Framework;
use dead_assets::reporter::{print_json_report, print_terminal_report};
use dead_assets::scanner::{collect_source_files, scan_references};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
/// dead-assets: Lightning-fast static asset analyzer for Astro, Next.js & Vite
struct CliArgs {
    /// Automatically clean / delete unreferenced assets
    #[bpaf(long, switch)]
    clean: bool,

    /// Force delete without confirmation prompt (useful for CI and scripts)
    #[bpaf(short, long, switch)]
    force: bool,

    /// Dry run: simulate deletion without removing files
    #[bpaf(long("dry-run"), switch)]
    dry_run: bool,

    /// Output results in JSON format
    #[bpaf(long, switch)]
    json: bool,

    /// Verbose output showing active references and referencing files
    #[bpaf(short, long, switch)]
    verbose: bool,

    /// Minimum asset size threshold to report (e.g. 50kb, 1mb, 200000)
    #[bpaf(long, argument("SIZE"))]
    threshold: Option<String>,

    /// Additional asset directories to scan (in addition to public/ and src/assets/)
    #[bpaf(long("asset-dir"), argument("DIR"))]
    asset_dirs: Vec<PathBuf>,

    /// Extra file extensions to treat as media assets (e.g. mp4, webm)
    #[bpaf(long("include-ext"), argument("EXT"))]
    include_ext: Vec<String>,

    /// Target project directory (defaults to current directory)
    #[bpaf(positional("PATH"), fallback(PathBuf::from(".")))]
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli_args().run();

    let threshold_bytes = args
        .threshold
        .as_deref()
        .map(|value| {
            parse_size_threshold(value).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid size threshold: {value}"),
                )
            })
        })
        .transpose()?;

    let root = match std::fs::canonicalize(&args.path) {
        Ok(p) => p,
        Err(_) => args.path.clone(),
    };

    if !root.is_dir() {
        eprintln!(
            "Error: Target path '{}' is not a valid directory.",
            root.display()
        );
        std::process::exit(1);
    }

    // 1. Detect Framework (Next.js, Astro, Vite, Generic)
    let framework = Framework::detect(&root);

    // 2. Discover Media Assets in public/, src/assets/, etc.
    let mut assets = discover_assets(&root, &args.asset_dirs, &args.include_ext)?;

    if assets.is_empty() && !args.json {
        println!(
            "No media assets found in public/ or src/assets/ for '{}'.",
            root.display()
        );
        return Ok(());
    }

    // Check framework-specific implicit conventions (e.g. Next.js metadata routes, favicons)
    for asset in assets.iter_mut() {
        if framework.is_implicit_convention(&asset.rel_path, &asset.file_name) {
            asset.is_referenced = true;
            asset
                .referenced_by
                .push(format!("[{} Convention]", framework.display_name()));
        }
    }

    // 3. Collect Source Code & Template Files
    let asset_paths: HashSet<PathBuf> = assets.iter().map(|a| a.abs_path.clone()).collect();
    let source_files = collect_source_files(&root, &asset_paths)?;

    // 4. Scan Source Files in Parallel with Aho-Corasick & Boundary Validation
    scan_references(&root, &mut assets, &source_files)?;

    // 5. Apply size threshold filter if requested
    if let Some(min_size) = threshold_bytes {
        assets.retain(|a| a.size_bytes >= min_size || a.is_referenced);
    }

    // 6. Compute Storage Audit Metrics
    let audit = StorageAudit::compute(&assets);

    // 7. Execute Cleanup if requested
    let mut cleanup_result = None;
    if args.clean || args.dry_run {
        let is_dry = args.dry_run;
        let res = execute_cleanup(&root, &audit.dead_asset_list, is_dry, args.force)?;
        cleanup_result = Some(res);
    }

    // 8. Output Report (Terminal UI or JSON)
    if args.json {
        print_json_report(framework, &audit, cleanup_result.as_ref());
    } else {
        print_terminal_report(
            framework,
            &audit,
            &assets,
            args.verbose,
            cleanup_result.as_ref(),
        );
    }

    if cleanup_result
        .as_ref()
        .is_some_and(|result| !result.failed_files.is_empty())
    {
        return Err(std::io::Error::other("Some assets could not be deleted").into());
    }
    Ok(())
}

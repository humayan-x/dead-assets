use dead_assets::{
    asset::discover_assets,
    audit::parse_size_threshold,
    cleaner::execute_cleanup,
    scanner::{collect_source_files, scan_references},
};
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "dead-assets-robust-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("public")).unwrap();
        Self(path)
    }
    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn cli(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_dead-assets"))
            .args(args)
            .arg(&self.0)
            .output()
            .unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn overlapping_matches_and_sparse_ids_are_safe() {
    let w = Workspace::new();
    w.write("public/a/logo.svg", "svg");
    w.write("public/b/logo.svg", "svg");
    w.write("index.html", "<img src='logo.svg'>");
    let mut assets = discover_assets(&w.0, &[], &[]).unwrap();
    for asset in &mut assets {
        asset.id += 100;
    }
    scan_references(&w.0, &mut assets, &[w.0.join("index.html")]).unwrap();
    assert!(assets.iter().all(|a| a.is_referenced));
}

#[test]
fn overlapping_asset_roots_are_deduplicated() {
    let w = Workspace::new();
    w.write("public/images/a.png", "image");
    let assets = discover_assets(
        &w.0,
        &[
            PathBuf::from("public/images"),
            PathBuf::from("public/../public"),
        ],
        &[],
    )
    .unwrap();
    assert_eq!(assets.len(), 1);
}

#[test]
fn outside_directories_and_cleanup_targets_are_rejected() {
    let w = Workspace::new();
    let outside = Workspace::new();
    outside.write("public/keep.png", "image");
    assert!(discover_assets(&w.0, std::slice::from_ref(&outside.0), &[]).is_err());
    let assets = discover_assets(&outside.0, &[], &[]).unwrap();
    assert!(execute_cleanup(&w.0, &assets, false, true).is_err());
    assert!(outside.0.join("public/keep.png").exists());
}

#[cfg(unix)]
#[test]
fn symlinked_asset_directory_cannot_escape_project() {
    let w = Workspace::new();
    let outside = Workspace::new();
    outside.write("public/keep.png", "image");
    std::os::unix::fs::symlink(outside.0.join("public"), w.0.join("assets")).unwrap();
    assert!(discover_assets(&w.0, &[], &[]).is_err());
}

#[test]
fn scan_errors_propagate_and_build_directories_are_excluded() {
    let w = Workspace::new();
    w.write("public/a.png", "image");
    for dir in ["node_modules", "dist", "build", ".next", "target"] {
        w.write(&format!("{dir}/bundle.js"), "'/a.png'");
    }
    let mut assets = discover_assets(&w.0, &[], &[]).unwrap();
    assert!(
        collect_source_files(&w.0, &HashSet::new())
            .unwrap()
            .is_empty()
    );
    assert!(scan_references(&w.0, &mut assets, &[w.0.join("missing.js")]).is_err());
    assert!(!assets[0].is_referenced);
}

#[test]
fn invalid_thresholds_are_errors_even_in_empty_projects() {
    let w = Workspace::new();
    for value in ["-1mb", "NaNkb", "infmb", "1e100gb", "nonsense"] {
        assert!(parse_size_threshold(value).is_none(), "{value}");
        assert!(!w.cli(&["--threshold", value]).status.success());
    }
}

#[test]
fn empty_json_and_dry_run_are_machine_readable() {
    let w = Workspace::new();
    let output = w.cli(&["--json"]);
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["audit"]["total_assets"], 0);
    w.write("public/dead.png", "image");
    let output = w.cli(&["--json", "--dry-run"]);
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["cleanup"]["is_dry_run"], true);
    assert!(w.0.join("public/dead.png").exists());
    let output = w.cli(&["--json", "--clean"]);
    assert!(output.status.success());
    serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert!(w.0.join("public/dead.png").exists());
}

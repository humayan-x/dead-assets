use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use dead_assets::asset::discover_assets;
use dead_assets::audit::StorageAudit;
use dead_assets::cleaner::execute_cleanup;
use dead_assets::framework::Framework;
use dead_assets::scanner::{collect_source_files, scan_references};

fn setup_test_workspace() -> PathBuf {
    let base = std::env::temp_dir().join(format!("dead_assets_test_{}", std::process::id()));
    if base.exists() {
        let _ = fs::remove_dir_all(&base);
    }
    fs::create_dir_all(&base).unwrap();

    // 1. Next.js / Astro indicators
    fs::write(base.join("next.config.js"), "module.exports = {};").unwrap();

    // 2. Setup public/ directory
    let public_img = base.join("public").join("images");
    let public_fonts = base.join("public").join("fonts");
    fs::create_dir_all(&public_img).unwrap();
    fs::create_dir_all(&public_fonts).unwrap();

    // Public assets:
    // a) favicon.ico (Next.js convention -> should NOT be dead)
    fs::write(base.join("public").join("favicon.ico"), vec![0u8; 1200]).unwrap();
    // b) used-hero.png (referenced in code -> NOT dead)
    fs::write(public_img.join("used-hero.png"), vec![0u8; 5000]).unwrap();
    // c) abandoned-hero.png (NOT referenced -> DEAD!)
    fs::write(public_img.join("abandoned-hero.png"), vec![0u8; 15000]).unwrap();
    // d) used-font.woff2 (referenced in CSS -> NOT dead)
    fs::write(public_fonts.join("Inter-Regular.woff2"), vec![0u8; 8000]).unwrap();
    // e) unused-font.woff2 (NOT referenced -> DEAD!)
    fs::write(public_fonts.join("Roboto-Thin.woff2"), vec![0u8; 12000]).unwrap();

    // 3. Setup src/assets/ directory
    let src_assets = base.join("src").join("assets");
    fs::create_dir_all(&src_assets).unwrap();
    // f) logo.svg (imported as @/assets/logo.svg -> NOT dead)
    fs::write(src_assets.join("logo.svg"), "<svg>logo</svg>").unwrap();
    // g) leftover-icon.svg (never imported -> DEAD!)
    fs::write(src_assets.join("leftover-icon.svg"), "<svg>leftover</svg>").unwrap();

    // 4. Setup source files referencing assets
    let src_components = base.join("src").join("components");
    let src_styles = base.join("src").join("styles");
    fs::create_dir_all(&src_components).unwrap();
    fs::create_dir_all(&src_styles).unwrap();

    // Header component imports @/assets/logo.svg and references /images/used-hero.png
    fs::write(
        src_components.join("Header.tsx"),
        r#"
        import React from 'react';
        import logo from '@/assets/logo.svg';

        export function Header() {
            return (
                <header>
                    <img src={logo} alt="Logo" />
                    <img src="/images/used-hero.png" alt="Hero" />
                </header>
            );
        }
        "#,
    )
    .unwrap();

    // Global CSS references public font
    fs::write(
        src_styles.join("globals.css"),
        r#"
        @font-face {
            font-family: 'Inter';
            src: url('/fonts/Inter-Regular.woff2') format('woff2');
        }
        "#,
    )
    .unwrap();

    base
}

#[test]
fn test_end_to_end_scanning_and_cleanup() {
    let workspace = setup_test_workspace();

    // 1. Framework detection
    let framework = Framework::detect(&workspace);
    assert_eq!(framework, Framework::NextJs);

    // 2. Asset discovery
    let mut assets = discover_assets(&workspace, &[], &[]).expect("Failed discovering assets");
    assert_eq!(assets.len(), 7); // favicon, used-hero, abandoned-hero, Inter font, Roboto font, logo.svg, leftover-icon.svg

    // 3. Framework implicit conventions (favicon.ico)
    for asset in assets.iter_mut() {
        if framework.is_implicit_convention(&asset.rel_path, &asset.file_name) {
            asset.is_referenced = true;
        }
    }

    // 4. Source scanning
    let asset_paths: HashSet<PathBuf> = assets.iter().map(|a| a.abs_path.clone()).collect();
    let sources = collect_source_files(&workspace, &asset_paths).unwrap();
    assert!(sources.len() >= 2); // Header.tsx, globals.css

    scan_references(&workspace, &mut assets, &sources).unwrap();

    // 5. Verification of active vs dead assets
    let find_asset = |name: &str| -> &dead_assets::asset::Asset {
        assets.iter().find(|a| a.file_name == name).unwrap()
    };

    assert!(
        find_asset("favicon.ico").is_referenced,
        "favicon.ico should be kept by convention"
    );
    assert!(
        find_asset("used-hero.png").is_referenced,
        "used-hero.png should be marked as referenced"
    );
    assert!(
        find_asset("logo.svg").is_referenced,
        "logo.svg should be marked as referenced"
    );
    assert!(
        find_asset("Inter-Regular.woff2").is_referenced,
        "Inter font should be marked as referenced"
    );

    assert!(
        !find_asset("abandoned-hero.png").is_referenced,
        "abandoned-hero.png should be DEAD"
    );
    assert!(
        !find_asset("Roboto-Thin.woff2").is_referenced,
        "Roboto font should be DEAD"
    );
    assert!(
        !find_asset("leftover-icon.svg").is_referenced,
        "leftover-icon.svg should be DEAD"
    );

    // 6. Audit math
    let audit = StorageAudit::compute(&assets);
    assert_eq!(audit.total_assets, 7);
    assert_eq!(audit.referenced_assets, 4);
    assert_eq!(audit.dead_assets, 3);

    let expected_reclaimable = 15000 + 12000 + "<svg>leftover</svg>".len() as u64;
    assert_eq!(audit.reclaimable_bytes, expected_reclaimable);

    // 7. Test dry run cleanup
    let dry_res = execute_cleanup(&workspace, &audit.dead_asset_list, true, true).unwrap();
    assert_eq!(dry_res.deleted_count, 3);
    assert_eq!(dry_res.deleted_bytes, expected_reclaimable);
    // Files must still exist on disk after dry run!
    assert!(workspace.join("public/images/abandoned-hero.png").exists());

    // 8. Test actual cleanup
    let real_res = execute_cleanup(&workspace, &audit.dead_asset_list, false, true).unwrap();
    assert_eq!(real_res.deleted_count, 3);
    assert_eq!(real_res.deleted_bytes, expected_reclaimable);

    // Dead files must now be gone
    assert!(!workspace.join("public/images/abandoned-hero.png").exists());
    assert!(!workspace.join("public/fonts/Roboto-Thin.woff2").exists());
    assert!(!workspace.join("src/assets/leftover-icon.svg").exists());

    // Used files must STILL exist
    assert!(workspace.join("public/favicon.ico").exists());
    assert!(workspace.join("public/images/used-hero.png").exists());
    assert!(workspace.join("src/assets/logo.svg").exists());
    assert!(workspace.join("public/fonts/Inter-Regular.woff2").exists());

    // Cleanup test workspace
    let _ = fs::remove_dir_all(&workspace);
}

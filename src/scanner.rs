use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use aho_corasick::{AhoCorasick, MatchKind};
use rayon::prelude::*;

use crate::asset::Asset;

/// Source code and template extensions to scan for asset references
const SOURCE_EXTENSIONS: &[&str] = &[
    // Script / TypeScript
    "js",
    "jsx",
    "ts",
    "tsx",
    "mjs",
    "cjs",
    "mts",
    "cts",
    // Component frameworks
    "astro",
    "vue",
    "svelte",
    // Markup & templates
    "html",
    "htm",
    // Styles
    "css",
    "scss",
    "sass",
    "less",
    "pcss",
    "postcss",
    "styl",
    // Content & Markdown
    "md",
    "mdx",
    "mdoc",
    "markdown",
    // Data & Config
    "json",
    "jsonc",
    "json5",
    "webmanifest",
    "yaml",
    "yml",
    "toml",
];

/// Collects all candidate source files in the project workspace, respecting .gitignore.
pub fn collect_source_files(
    root: &Path,
    asset_paths: &HashSet<PathBuf>,
) -> std::io::Result<Vec<PathBuf>> {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|entry| {
            !entry.file_type().is_some_and(|t| t.is_dir())
                || !matches!(
                    entry.file_name().to_str(),
                    Some("node_modules" | "dist" | "build" | ".next" | ".git" | "target")
                )
        })
        .build();

    let mut sources = Vec::new();

    for entry in walker {
        let entry = entry.map_err(std::io::Error::other)?;
        if let Some(error) = entry.error() {
            return Err(std::io::Error::other(error.to_string()));
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Never scan assets themselves as source files
        if asset_paths.contains(path) {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if SOURCE_EXTENSIONS.contains(&ext.as_str()) {
            sources.push(path.to_path_buf());
        }
    }

    Ok(sources)
}

/// Checks whether byte is a valid boundary delimiter preceding an asset reference
#[inline]
fn is_valid_leading_boundary(b: u8) -> bool {
    matches!(
        b,
        b'"' | b'\''
            | b'`'
            | b'/'
            | b'\\'
            | b'('
            | b'{'
            | b'<'
            | b'>'
            | b','
            | b'='
            | b':'
            | b'['
            | b' '
            | b'\t'
            | b'\n'
            | b'\r'
            | b'!'
            | b'?'
            | b'&'
            | b';'
            | b'|'
            | b'#'
    )
}

/// Checks whether byte is a valid boundary delimiter trailing an asset reference
#[inline]
fn is_valid_trailing_boundary(b: u8) -> bool {
    matches!(
        b,
        b'"' | b'\''
            | b'`'
            | b'/'
            | b'\\'
            | b'?'
            | b'#'
            | b')'
            | b'}'
            | b'>'
            | b'<'
            | b','
            | b';'
            | b']'
            | b' '
            | b'\t'
            | b'\n'
            | b'\r'
            | b'='
            | b'&'
            | b'|'
    )
}

/// Verifies whether the match at `[start..end]` within `content` has valid word/path boundaries.
/// Prevents false positives such as `silicon.svg` matching `icon.svg` or `icon.svg.bak` matching `icon.svg`.
#[inline]
pub fn has_valid_boundaries(content: &[u8], start: usize, end: usize) -> bool {
    // Leading boundary check
    if start > 0 {
        let prev = content[start - 1];
        if !is_valid_leading_boundary(prev) {
            return false;
        }
    }

    // Trailing boundary check
    if end < content.len() {
        let next = content[end];
        if !is_valid_trailing_boundary(next) {
            return false;
        }
    }

    true
}

/// Normalizes a relative path by resolving `.` and `..` segments
pub fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            c => out.push(c.as_os_str()),
        }
    }
    out
}

/// Scans all collected source files in parallel and marks referenced assets.
pub fn scan_references(
    root: &Path,
    assets: &mut [Asset],
    source_files: &[PathBuf],
) -> std::io::Result<()> {
    if assets.is_empty() || source_files.is_empty() {
        return Ok(());
    }

    // 1. Build pattern to asset ID mapping
    let mut patterns = Vec::new();
    let mut pattern_to_asset_id = Vec::new();

    // Map relative paths from root for direct relative resolution
    let mut rel_path_to_id: HashMap<String, usize> = HashMap::new();

    for (index, asset) in assets.iter().enumerate() {
        rel_path_to_id.insert(asset.rel_path.clone(), index);

        for key in &asset.search_keys {
            patterns.push(key.clone());
            pattern_to_asset_id.push(index);
        }
    }

    // 2. Standard semantics support overlapping and duplicate patterns
    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::Standard)
        .build(&patterns)
        .map_err(std::io::Error::other)?;

    // Use atomic flags for fast thread-safe marked status
    let referenced_flags: Vec<AtomicBool> =
        (0..assets.len()).map(|_| AtomicBool::new(false)).collect();

    // Store sample source references per asset (up to 3 references each for verbose mode)
    let referenced_sources: Vec<Mutex<Vec<String>>> =
        (0..assets.len()).map(|_| Mutex::new(Vec::new())).collect();

    // 3. Process source files concurrently using rayon
    source_files
        .par_iter()
        .try_for_each(|source_path| -> std::io::Result<()> {
            let mut file = File::open(source_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            let source_rel = source_path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| source_path.to_string_lossy().replace('\\', "/"));

            let source_dir = source_path.parent().unwrap_or(root);

            // A. Fast Aho-Corasick scan
            for mat in ac.find_overlapping_iter(&buffer) {
                let start = mat.start();
                let end = mat.end();

                if has_valid_boundaries(&buffer, start, end) {
                    let asset_id = pattern_to_asset_id[mat.pattern().as_usize()];
                    referenced_flags[asset_id].store(true, Ordering::Relaxed);

                    if let Ok(mut sources) = referenced_sources[asset_id].lock()
                        && sources.len() < 3
                        && !sources.contains(&source_rel)
                    {
                        sources.push(source_rel.clone());
                    }
                }
            }

            // B. Relative path literal resolution:
            //    Extract string tokens enclosed in quotes: "...", '...', `...`
            //    If a token starts with `./` or `../`, resolve it against source_dir
            let mut i = 0;
            let len = buffer.len();
            while i < len {
                let quote = buffer[i];
                if quote == b'"' || quote == b'\'' || quote == b'`' {
                    i += 1;
                    let start_idx = i;
                    while i < len && buffer[i] != quote && buffer[i] != b'\n' {
                        if buffer[i] == b'\\' && i + 1 < len {
                            i += 2; // skip escape
                        } else {
                            i += 1;
                        }
                    }

                    if i < len && buffer[i] == quote {
                        let slice = &buffer[start_idx..i];
                        if (slice.starts_with(b"./") || slice.starts_with(b"../"))
                            && let Ok(token_str) = std::str::from_utf8(slice)
                        {
                            // Strip query or hash if present: e.g. "./image.png?v=1"
                            let clean_token = token_str
                                .split('?')
                                .next()
                                .unwrap_or(token_str)
                                .split('#')
                                .next()
                                .unwrap_or(token_str);

                            let combined = source_dir.join(clean_token);
                            let normalized = normalize_path(&combined);

                            if let Ok(rel) = normalized.strip_prefix(root) {
                                let rel_str = rel.to_string_lossy().replace('\\', "/");
                                if let Some(&asset_id) = rel_path_to_id.get(&rel_str) {
                                    referenced_flags[asset_id].store(true, Ordering::Relaxed);
                                    if let Ok(mut sources) = referenced_sources[asset_id].lock()
                                        && sources.len() < 3
                                        && !sources.contains(&source_rel)
                                    {
                                        sources.push(source_rel.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                i += 1;
            }
            Ok(())
        })?;

    // 4. Update assets with detected status
    for (idx, asset) in assets.iter_mut().enumerate() {
        if referenced_flags[idx].load(Ordering::Relaxed) {
            asset.is_referenced = true;
            if let Ok(sources) = referenced_sources[idx].lock() {
                asset.referenced_by = sources.clone();
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_validation() {
        // "icon.svg" inside "import icon from './icon.svg';" -> valid
        let text1 = b"import icon from './icon.svg';";
        let pattern = b"icon.svg";
        let start1 = text1
            .windows(pattern.len())
            .position(|w| w == pattern)
            .unwrap();
        let end1 = start1 + pattern.len();
        assert!(has_valid_boundaries(text1, start1, end1));

        // "icon.svg" inside "silicon.svg" -> INVALID (leading 'l' is alphanumeric)
        let text2 = b"const file = 'silicon.svg';";
        let start2 = text2
            .windows(pattern.len())
            .position(|w| w == pattern)
            .unwrap();
        let end2 = start2 + pattern.len();
        assert!(!has_valid_boundaries(text2, start2, end2));

        // "icon.svg" inside "icon.svg.backup" -> INVALID (trailing '.' followed by alphanumeric)
        let text3 = b"const file = 'icon.svg.backup';";
        let start3 = text3
            .windows(pattern.len())
            .position(|w| w == pattern)
            .unwrap();
        let end3 = start3 + pattern.len();
        assert!(!has_valid_boundaries(text3, start3, end3));

        // JSX <img src="/images/hero.png" />
        let text4 = b"<img src=\"/images/hero.png\" />";
        let pat4 = b"/images/hero.png";
        let start4 = text4.windows(pat4.len()).position(|w| w == pat4).unwrap();
        let end4 = start4 + pat4.len();
        assert!(has_valid_boundaries(text4, start4, end4));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path(Path::new("src/components/../assets/hero.png")),
            PathBuf::from("src/assets/hero.png")
        );
        assert_eq!(
            normalize_path(Path::new("src/./assets/./hero.png")),
            PathBuf::from("src/assets/hero.png")
        );
    }
}

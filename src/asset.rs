use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Image,
    Svg,
    Font,
    Media,
    Other,
}

impl AssetKind {
    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext_lower = ext.to_ascii_lowercase();
        match ext_lower.as_str() {
            // Images
            "png" | "jpg" | "jpeg" | "webp" | "avif" | "gif" | "ico" | "bmp" | "tiff" | "tif" => {
                Some(AssetKind::Image)
            }
            // SVGs
            "svg" => Some(AssetKind::Svg),
            // Fonts
            "woff" | "woff2" | "ttf" | "otf" | "eot" => Some(AssetKind::Font),
            // Audio / Video
            "mp4" | "webm" | "mp3" | "wav" | "ogg" | "m4a" | "aac" | "flac" => {
                Some(AssetKind::Media)
            }
            _ => None,
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            AssetKind::Image => "Image",
            AssetKind::Svg => "SVG",
            AssetKind::Font => "Font",
            AssetKind::Media => "Media",
            AssetKind::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetLocation {
    Public,
    SrcAssets,
    Custom,
}

impl AssetLocation {
    pub fn display_label(&self) -> &'static str {
        match self {
            AssetLocation::Public => "public/",
            AssetLocation::SrcAssets => "src/assets/",
            AssetLocation::Custom => "custom/",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: usize,
    /// Forward-slash normalized relative path from project root, e.g. "public/images/logo.png"
    pub rel_path: String,
    #[serde(skip)]
    pub abs_path: PathBuf,
    pub file_name: String,
    pub file_stem: String,
    pub size_bytes: u64,
    pub kind: AssetKind,
    pub location: AssetLocation,
    /// Web-accessible URL path if located in public/, e.g. "/images/logo.png"
    pub public_url: Option<String>,
    /// Search patterns for this asset
    #[serde(skip)]
    pub search_keys: Vec<String>,
    pub is_referenced: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub referenced_by: Vec<String>,
}

impl Asset {
    pub fn new(
        id: usize,
        rel_path: String,
        abs_path: PathBuf,
        size_bytes: u64,
        kind: AssetKind,
        location: AssetLocation,
    ) -> Self {
        let file_name = abs_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let file_stem = abs_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let public_url = if location == AssetLocation::Public {
            // "public/images/hero.png" -> "/images/hero.png"
            if let Some(stripped) = rel_path.strip_prefix("public/") {
                Some(format!("/{}", stripped))
            } else if rel_path == "public" {
                None
            } else {
                Some(format!("/{}", rel_path))
            }
        } else {
            None
        };

        let mut search_keys = Vec::new();

        // 1. Direct file name, e.g. "hero.png"
        if !file_name.is_empty() {
            search_keys.push(file_name.clone());
        }

        // 2. Web URL path (e.g. "/images/hero.png" and "images/hero.png")
        if let Some(ref url) = public_url {
            search_keys.push(url.clone());
            if let Some(without_slash) = url.strip_prefix('/')
                && !without_slash.is_empty()
            {
                search_keys.push(without_slash.to_string());
            }
        }

        // 3. Relative path from project root, e.g. "src/assets/hero.png"
        search_keys.push(rel_path.clone());

        // 4. Aliased path variants for src/assets:
        //    "src/assets/hero.png" -> "@/assets/hero.png", "~/assets/hero.png", "$/assets/hero.png"
        if let Some(sub) = rel_path.strip_prefix("src/") {
            search_keys.push(format!("@/{}", sub));
            search_keys.push(format!("~/{}", sub));
            search_keys.push(format!("$/{}", sub));
            search_keys.push(sub.to_string());
        }

        // 5. Bare import paths without leading slash
        if let Some(sub) = rel_path.strip_prefix("src/assets/") {
            search_keys.push(format!("./assets/{}", sub));
            search_keys.push(format!("../assets/{}", sub));
            search_keys.push(format!("./{}", sub));
        }

        // Remove duplicates and empty keys
        search_keys.sort();
        search_keys.dedup();
        search_keys.retain(|k| !k.trim().is_empty());

        Self {
            id,
            rel_path,
            abs_path,
            file_name,
            file_stem,
            size_bytes,
            kind,
            location,
            public_url,
            search_keys,
            is_referenced: false,
            referenced_by: Vec::new(),
        }
    }
}

/// Discovers assets inside standard frontend asset folders: `public/`, `src/assets/`,
/// and any explicitly configured custom asset directories.
pub fn discover_assets(
    root: &Path,
    custom_dirs: &[PathBuf],
    custom_exts: &[String],
) -> std::io::Result<Vec<Asset>> {
    let root = &std::fs::canonicalize(root)?;
    let mut candidate_dirs: Vec<(PathBuf, AssetLocation)> = Vec::new();

    let public_dir = root.join("public");
    if public_dir.is_dir() {
        candidate_dirs.push((public_dir, AssetLocation::Public));
    }

    let src_assets_dir = root.join("src").join("assets");
    if src_assets_dir.is_dir() {
        candidate_dirs.push((src_assets_dir, AssetLocation::SrcAssets));
    }

    // Also check for "assets" directly at root (often used in Vite or Astro)
    let root_assets = root.join("assets");
    if root_assets.is_dir() && !candidate_dirs.iter().any(|(d, _)| d == &root_assets) {
        candidate_dirs.push((root_assets, AssetLocation::SrcAssets));
    }

    for custom in custom_dirs {
        let resolved = if custom.is_absolute() {
            custom.clone()
        } else {
            root.join(custom)
        };
        if resolved.is_dir() && !candidate_dirs.iter().any(|(d, _)| d == &resolved) {
            candidate_dirs.push((resolved, AssetLocation::Custom));
        }
    }

    let mut assets = Vec::new();
    let mut asset_id = 0;
    let mut seen = std::collections::HashSet::new();

    for (dir, location) in candidate_dirs {
        let dir = std::fs::canonicalize(dir)?;
        if !dir.starts_with(root) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Asset directories must be inside the project",
            ));
        }
        // Walk the directory recursively
        let walker = ignore::WalkBuilder::new(&dir)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker {
            let entry = entry.map_err(std::io::Error::other)?;
            if let Some(error) = entry.error() {
                return Err(std::io::Error::other(error.to_string()));
            }
            let path = entry.path();
            if !entry.file_type().is_some_and(|t| t.is_file()) || !seen.insert(path.to_path_buf()) {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            let kind = if let Some(k) = AssetKind::from_extension(&ext) {
                k
            } else if custom_exts
                .iter()
                .any(|ce| ce.trim_start_matches('.').eq_ignore_ascii_case(&ext))
            {
                AssetKind::Other
            } else {
                continue;
            };

            let metadata = entry.metadata().map_err(std::io::Error::other)?;

            let rel_path = match path.strip_prefix(root) {
                Ok(r) => r.to_string_lossy().replace('\\', "/"),
                Err(_) => path.to_string_lossy().replace('\\', "/"),
            };

            let asset = Asset::new(
                asset_id,
                rel_path,
                path.to_path_buf(),
                metadata.len(),
                kind,
                location,
            );
            assets.push(asset);
            asset_id += 1;
        }
    }

    Ok(assets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_kind_detection() {
        assert_eq!(AssetKind::from_extension("png"), Some(AssetKind::Image));
        assert_eq!(AssetKind::from_extension("JPG"), Some(AssetKind::Image));
        assert_eq!(AssetKind::from_extension("webp"), Some(AssetKind::Image));
        assert_eq!(AssetKind::from_extension("svg"), Some(AssetKind::Svg));
        assert_eq!(AssetKind::from_extension("woff2"), Some(AssetKind::Font));
        assert_eq!(AssetKind::from_extension("ttf"), Some(AssetKind::Font));
        assert_eq!(AssetKind::from_extension("mp4"), Some(AssetKind::Media));
        assert_eq!(AssetKind::from_extension("unknown"), None);
    }

    #[test]
    fn test_asset_search_keys_public() {
        let asset = Asset::new(
            0,
            "public/images/logo.png".to_string(),
            PathBuf::from("/workspace/public/images/logo.png"),
            1024,
            AssetKind::Image,
            AssetLocation::Public,
        );

        assert_eq!(asset.public_url, Some("/images/logo.png".to_string()));
        assert!(asset.search_keys.contains(&"logo.png".to_string()));
        assert!(asset.search_keys.contains(&"/images/logo.png".to_string()));
        assert!(asset.search_keys.contains(&"images/logo.png".to_string()));
        assert!(
            asset
                .search_keys
                .contains(&"public/images/logo.png".to_string())
        );
    }

    #[test]
    fn test_asset_search_keys_src_assets() {
        let asset = Asset::new(
            1,
            "src/assets/icons/star.svg".to_string(),
            PathBuf::from("/workspace/src/assets/icons/star.svg"),
            512,
            AssetKind::Svg,
            AssetLocation::SrcAssets,
        );

        assert_eq!(asset.public_url, None);
        assert!(asset.search_keys.contains(&"star.svg".to_string()));
        assert!(
            asset
                .search_keys
                .contains(&"@/assets/icons/star.svg".to_string())
        );
        assert!(
            asset
                .search_keys
                .contains(&"~/assets/icons/star.svg".to_string())
        );
        assert!(
            asset
                .search_keys
                .contains(&"src/assets/icons/star.svg".to_string())
        );
    }
}

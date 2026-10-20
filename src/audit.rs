use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::asset::Asset;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryStat {
    pub total_count: usize,
    pub total_bytes: u64,
    pub dead_count: usize,
    pub dead_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAudit {
    pub total_assets: usize,
    pub total_bytes: u64,
    pub total_bytes_formatted: String,
    pub referenced_assets: usize,
    pub referenced_bytes: u64,
    pub dead_assets: usize,
    pub reclaimable_bytes: u64,
    pub reclaimable_bytes_formatted: String,
    pub reclaimable_percent: f64,
    pub by_kind: HashMap<String, CategoryStat>,
    pub by_location: HashMap<String, CategoryStat>,
    pub dead_asset_list: Vec<Asset>,
}

impl StorageAudit {
    pub fn compute(assets: &[Asset]) -> Self {
        let total_assets = assets.len();
        let mut total_bytes: u64 = 0;
        let mut referenced_assets = 0;
        let mut referenced_bytes: u64 = 0;
        let mut dead_assets = 0;
        let mut reclaimable_bytes: u64 = 0;

        let mut by_kind: HashMap<String, CategoryStat> = HashMap::new();
        let mut by_location: HashMap<String, CategoryStat> = HashMap::new();
        let mut dead_asset_list = Vec::new();

        for asset in assets {
            total_bytes += asset.size_bytes;

            let kind_key = asset.kind.display_label().to_string();
            let kind_stat = by_kind.entry(kind_key).or_default();
            kind_stat.total_count += 1;
            kind_stat.total_bytes += asset.size_bytes;

            let loc_key = asset.location.display_label().to_string();
            let loc_stat = by_location.entry(loc_key).or_default();
            loc_stat.total_count += 1;
            loc_stat.total_bytes += asset.size_bytes;

            if asset.is_referenced {
                referenced_assets += 1;
                referenced_bytes += asset.size_bytes;
            } else {
                dead_assets += 1;
                reclaimable_bytes += asset.size_bytes;
                kind_stat.dead_count += 1;
                kind_stat.dead_bytes += asset.size_bytes;
                loc_stat.dead_count += 1;
                loc_stat.dead_bytes += asset.size_bytes;
                dead_asset_list.push(asset.clone());
            }
        }

        // Sort dead assets by size descending (largest savings first)
        dead_asset_list.sort_by_key(|a| (std::cmp::Reverse(a.size_bytes), a.rel_path.clone()));

        let reclaimable_percent = if total_bytes > 0 {
            (reclaimable_bytes as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_assets,
            total_bytes,
            total_bytes_formatted: format_bytes(total_bytes),
            referenced_assets,
            referenced_bytes,
            dead_assets,
            reclaimable_bytes,
            reclaimable_bytes_formatted: format_bytes(reclaimable_bytes),
            reclaimable_percent,
            by_kind,
            by_location,
            dead_asset_list,
        }
    }
}

/// Formats raw byte count into human-readable string (e.g. "1.45 MB", "512.00 KB")
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

/// Parses a size string like "100kb", "1.5mb", "500000" into bytes
pub fn parse_size_threshold(input: &str) -> Option<u64> {
    let s = input.trim().to_ascii_lowercase();
    if s.is_empty() {
        return None;
    }

    for (suffix, multiplier) in [("gb", 1073741824.0), ("mb", 1048576.0), ("kb", 1024.0)] {
        if let Some(val) = s.strip_suffix(suffix) {
            let bytes = val.trim().parse::<f64>().ok()? * multiplier;
            return (bytes.is_finite() && bytes >= 0.0 && bytes < u64::MAX as f64)
                .then_some(bytes as u64);
        }
    }
    s.strip_suffix('b').unwrap_or(&s).trim().parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024 * 2), "2.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 3), "3.00 GB");
    }

    #[test]
    fn test_parse_size_threshold() {
        assert_eq!(parse_size_threshold("500b"), Some(500));
        assert_eq!(parse_size_threshold("1kb"), Some(1024));
        assert_eq!(parse_size_threshold("2mb"), Some(2 * 1024 * 1024));
        assert_eq!(parse_size_threshold("1.5mb"), Some(1572864));
    }
}

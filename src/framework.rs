use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    NextJs,
    Astro,
    Vite,
    Generic,
}

impl Framework {
    pub fn display_name(&self) -> &'static str {
        match self {
            Framework::NextJs => "Next.js",
            Framework::Astro => "Astro",
            Framework::Vite => "Vite",
            Framework::Generic => "Frontend Project",
        }
    }

    /// Detect framework from workspace root files and package.json
    pub fn detect(root: &Path) -> Self {
        // 1. Next.js indicators
        if root.join("next.config.js").is_file()
            || root.join("next.config.mjs").is_file()
            || root.join("next.config.ts").is_file()
            || root.join(".next").is_dir()
        {
            return Framework::NextJs;
        }

        // 2. Astro indicators
        if root.join("astro.config.mjs").is_file()
            || root.join("astro.config.ts").is_file()
            || root.join("astro.config.js").is_file()
            || root.join(".astro").is_dir()
        {
            return Framework::Astro;
        }

        // 3. Vite indicators
        if root.join("vite.config.js").is_file()
            || root.join("vite.config.ts").is_file()
            || root.join("vite.config.mjs").is_file()
            || root.join("vite.config.cjs").is_file()
        {
            return Framework::Vite;
        }

        // 4. Check package.json dependencies
        let pkg_path = root.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if content.contains("\"next\"") {
                return Framework::NextJs;
            }
            if content.contains("\"astro\"") {
                return Framework::Astro;
            }
            if content.contains("\"vite\"") {
                return Framework::Vite;
            }
        }

        Framework::Generic
    }

    /// Returns true if an asset path matches framework-specific implicit conventions
    /// (e.g. Next.js metadata routes, standard favicons, manifest files) that might
    /// never be explicitly imported or linked in code.
    pub fn is_implicit_convention(&self, rel_path: &str, file_name: &str) -> bool {
        let norm = rel_path.replace('\\', "/");
        let name_lower = file_name.to_ascii_lowercase();

        // Universal web standards in public/
        if norm.starts_with("public/") {
            match name_lower.as_str() {
                "favicon.ico"
                | "favicon.svg"
                | "favicon.png"
                | "apple-touch-icon.png"
                | "apple-touch-icon-precomposed.png"
                | "robots.txt"
                | "sitemap.xml"
                | "sitemap.xml.gz"
                | "manifest.json"
                | "manifest.webmanifest"
                | "site.webmanifest"
                | "browserconfig.xml"
                | "crossdomain.xml" => return true,
                _ => {}
            }
        }

        match self {
            Framework::NextJs => {
                // Next.js App Router metadata conventions
                // Can be in app/ or src/app/ or public/
                if norm.starts_with("public/")
                    || norm.starts_with("src/app/")
                    || norm.starts_with("app/")
                {
                    match name_lower.as_str() {
                        "favicon.ico"
                        | "icon.png"
                        | "icon.jpg"
                        | "icon.jpeg"
                        | "icon.svg"
                        | "apple-icon.png"
                        | "apple-icon.jpg"
                        | "apple-icon.svg"
                        | "opengraph-image.png"
                        | "opengraph-image.jpg"
                        | "opengraph-image.alt.txt"
                        | "twitter-image.png"
                        | "twitter-image.jpg"
                        | "twitter-image.alt.txt" => {
                            return true;
                        }
                        _ => {}
                    }
                }
            }
            Framework::Astro => {
                // Astro default static items
                if norm == "public/favicon.svg" || norm == "public/favicon.ico" {
                    return true;
                }
            }
            Framework::Vite => {
                // Vite default template assets
                if norm == "public/vite.svg" || norm == "public/favicon.ico" {
                    return true;
                }
            }
            Framework::Generic => {}
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_conventions() {
        let nextjs = Framework::NextJs;
        assert!(nextjs.is_implicit_convention("public/favicon.ico", "favicon.ico"));
        assert!(nextjs.is_implicit_convention("public/robots.txt", "robots.txt"));
        assert!(
            nextjs.is_implicit_convention("src/app/opengraph-image.png", "opengraph-image.png")
        );
        assert!(!nextjs.is_implicit_convention("public/random-banner.png", "random-banner.png"));

        let astro = Framework::Astro;
        assert!(astro.is_implicit_convention("public/favicon.svg", "favicon.svg"));
        assert!(!astro.is_implicit_convention("public/old-logo.svg", "old-logo.svg"));

        let vite = Framework::Vite;
        assert!(vite.is_implicit_convention("public/vite.svg", "vite.svg"));
        assert!(!vite.is_implicit_convention("public/unused.svg", "unused.svg"));
    }
}

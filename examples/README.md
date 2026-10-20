# Scanner fixtures

These directories demonstrate source layouts and literal asset references for Astro and Next.js. They are input fixtures for dead-assets, not maintained or production-ready application templates. Framework versions in their manifests are sample detection inputs; do not deploy these directories or treat their dependency versions as recommendations.

Image and font files are placeholder data for scanning and size accounting, not functional artwork or usable fonts. The manifests and configuration files help exercise framework detection.

From the repository root:

```bash
cargo run -- examples/astro-app --json
cargo run -- examples/nextjs-app --dry-run
```

These commands do not modify the fixtures. For cleanup tests, use temporary workspaces as the automated tests do. Keep fixtures small and synthetic; document the origin and license of any real media you add.

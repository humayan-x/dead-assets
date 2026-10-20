# dead-assets

Find unreferenced images, SVGs, fonts, and media in frontend projects.

`dead-assets` is a Rust command-line tool for Astro, Next.js, Vite, and similar projects. It scans source files for asset references, reports potential storage savings, and optionally removes files after confirmation.

**Status:** early development (`0.1.0`). npm installation instructions apply after the first publication. Results are cleanup candidates, not proof that a file is unused. Review a dry run before deleting files.

## Quick start

From an installed CLI:

```bash
dead-assets ./my-app                 # Audit without changing files
dead-assets ./my-app --dry-run       # Preview cleanup
dead-assets ./my-app --clean         # Delete after confirmation
dead-assets ./my-app --json          # Machine-readable report
```

With no path, the current directory is scanned. `--clean --force` skips confirmation; `--dry-run` takes precedence over deletion. Keep a recoverable copy of assets before cleanup.

## Installation

### From source

Install a current stable Rust toolchain and the platform's native build tools, then:

```bash
git clone https://github.com/humayan-x/dead-assets.git
cd dead-assets
cargo install --path . --locked
dead-assets --help
```

The standalone executable does not require Node.js. The code uses Rust edition 2024; an older minimum Rust version is not currently tested.

### npm (after publication)

```bash
npx dead-assets ./my-app
# Or install once:
npm install --global dead-assets
```

The npm launcher requires Node.js 20 or newer and bundles the Rust executables. End users do not need Cargo, an install script, or a separate binary download.

The full release workflow targets:

| Platform | Architecture | Build baseline |
| --- | --- | --- |
| Linux | x64 | Ubuntu 22.04 / glibc 2.35+ |
| Linux | ARM64 | Ubuntu 24.04 / glibc 2.39+ |
| macOS | Intel | macOS 15 runner |
| macOS | Apple Silicon | macOS 14 runner |
| Windows | x64 | Windows Server 2022 runner |

These are build/test environments, not a compatibility guarantee for every OS version. Alpine/musl and other architectures need a source build. Local tarballs contain only the build machine's platform and inherit its system requirements; use the workflow artifact for a cross-platform npm release.

## What it scans

- Assets in `public/`, `src/assets/`, and root-level `assets/`, plus directories passed through `--asset-dir`.
- JavaScript, TypeScript, JSX/TSX, Astro, Vue, Svelte, HTML, CSS and related stylesheets, Markdown/MDX, and common data/config formats.
- Literal URLs, filenames, common `@/`, `~/`, and `$/` aliases into `src/`, and relative paths.
- Selected framework conventions, such as favicons, which are retained even without explicit references. App Router directories are not discovered by default; add them with `--asset-dir` when needed.

| Type | Extensions |
| --- | --- |
| Images | `png`, `jpg`, `jpeg`, `webp`, `avif`, `gif`, `ico`, `bmp`, `tiff`, `tif` |
| SVGs | `svg` |
| Fonts | `woff`, `woff2`, `ttf`, `otf`, `eot` |
| Audio/video | `mp4`, `webm`, `mp3`, `wav`, `ogg`, `m4a`, `aac`, `flac` |
| Custom | Extensions supplied with `--include-ext` |

Discovery deduplicates overlapping asset directories and skips symlink files. Asset directories and cleanup targets must resolve inside the project. Source discovery skips hidden entries, respects ignore rules, and excludes `node_modules`, `dist`, `build`, `.next`, `.git`, and `target`. Git ignore rules depend on repository context and may also include user-global exclusions.

## Options

| Option | Behavior |
| --- | --- |
| `--clean` | Delete candidates after confirmation |
| `-f`, `--force` | Skip cleanup confirmation; does not enable cleanup by itself |
| `--dry-run` | Preview deletion without removing files |
| `--json` | Emit a JSON report on stdout |
| `-v`, `--verbose` | Include sampled reference locations in the terminal report when unused assets exist |
| `--threshold SIZE` | Exclude unreferenced assets smaller than the threshold from reporting and cleanup |
| `--asset-dir DIR` | Add an asset directory within the project; repeatable |
| `--include-ext EXT` | Recognize another media extension; repeatable |
| `-h`, `--help` | Print usage |
| `-V`, `--version` | Print version |

```bash
dead-assets ./my-app --threshold 100kb --dry-run
dead-assets ./my-app --asset-dir static --include-ext pdf --json
```

Sizes use binary multiples: `1kb` = 1,024 bytes. Fractions such as `1.5mb` are accepted. Invalid, negative, non-finite, and overflowing thresholds are rejected. Thresholds filter before audit totals are computed; totals therefore describe the filtered set, including all referenced assets.

JSON reports contain `framework`, `audit`, and, when requested, `cleanup`. Empty projects also produce JSON. Prompts and errors go to stderr. Dry-run cleanup counts describe planned deletions. Finding unused assets does **not** cause a nonzero exit status; argument, scan, and cleanup errors do. Scripts that want an unused-asset CI gate should inspect `audit.dead_assets`.

## Limitations

This is a static text scanner, not a JavaScript interpreter or a bundler dependency graph.

- Runtime-generated paths, API/CMS responses, computed filenames, dynamic glob imports, and references in ignored files may be missed.
- Duplicate filenames are retained conservatively: a bare filename match can keep several assets. Comments, documentation, and dead source code can also retain assets.
- Custom alias mappings and framework configuration are not fully resolved. Framework detection and convention handling use heuristics.
- SVGs and other discovered assets are not scanned as source files, so references between assets may be missed.
- Discovery/read errors stop cleanup. Cleanup validates paths before deletion, but validation and deletion are not an atomic transaction. Avoid concurrent filesystem changes during cleanup.

No published benchmark suite currently substantiates a fixed speed or binary-size guarantee. Matching uses Aho-Corasick and files are scanned in parallel with Rayon; performance depends on the project and build configuration.

## Development and contributions

Bug reports, minimal reproductions, documentation improvements, and focused pull requests are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md). Report vulnerabilities privately using [SECURITY.md](SECURITY.md).

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
npm run build:native
npm test
npm run pack:local
```

There are no npm runtime dependencies to install. `Cargo.lock` is committed for repeatable dependency resolution. Tests create temporary projects; do not run cleanup against your real project to test changes.

| Location | Purpose |
| --- | --- |
| `src/` | Rust discovery, scanning, reporting, and cleanup |
| `bin/` | npm command launcher |
| `scripts/` | Native build and npm packaging checks |
| `tests/` | Rust and npm regression tests |
| `examples/` | Scanner fixtures; see [fixture notes](examples/README.md) |
| `.github/workflows/` | Pull-request checks and release builds |

For publishing and the GitHub setup checklist, see [docs/RELEASING.md](docs/RELEASING.md). Changes are recorded in [CHANGELOG.md](CHANGELOG.md).

## License

Available under either the [MIT License](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE), at your option. Dependency notices are collected in [THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt).

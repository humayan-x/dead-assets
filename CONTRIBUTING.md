# Contributing to dead-assets

Contributions are welcome: reproductions of missed references, platform compatibility fixes, documentation, and tests all help. Follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Before starting

Search existing issues and pull requests. For a large feature or a change to cleanup behavior, open an issue describing the problem and proposed behavior first. Small fixes can go straight to a pull request. Use [SECURITY.md](SECURITY.md) for vulnerabilities rather than a public issue.

## Local setup

Use current stable Rust with `rustfmt` and Clippy, a native linker/build toolchain, and Node.js 20+ for the npm wrapper. Fork and clone the repository, then create a branch for your change. The root npm package has no installable dependencies; there is no need to run `npm install`.

```bash
rustup component add rustfmt clippy
cargo test --locked
cargo run -- --help
npm run build:native
npm test
```

The `examples/` directories are scanner fixtures, not maintained application starters. Do not install or upgrade their framework dependencies as part of routine setup.

## Making changes

- Keep changes focused and explain the user-visible problem in the PR.
- Add regression coverage for scanner, cleanup, argument handling, or launcher behavior changes. Use temporary fixtures, never destructive tests against an actual application.
- Treat false declarations of unused assets as potential data-loss bugs. Preserve error propagation before cleanup and checks that limit deletion to the project.
- Keep stdout valid JSON in JSON mode; diagnostics and interactive prompts belong on stderr.
- Update the README or changelog when behavior or compatibility changes.
- Keep `Cargo.lock` tracked. If dependencies change, review the lockfile diff and refresh the third-party notices using the release guide.
- Do not include generated binaries, npm archives, credentials, or unrelated formatting changes.

## Before submitting

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
npm run build:native
npm test
npm run pack:local
```

`npm pack` intentionally fails unless all five platform binaries are present. For local testing, use `npm run pack:local`; see [the release guide](docs/RELEASING.md).

Include a minimal reproduction or before/after description, the checks you ran, and any platform checks you could not run. Documentation-only changes do not need new tests. GitHub Actions runs Rust checks and launcher tests on Linux, macOS, and Windows; maintainers review and merge contributions.

## Licensing

By intentionally submitting a contribution for inclusion, you agree to license it under the project's existing MIT OR Apache-2.0 terms. Do not submit code or assets you lack permission to contribute. Preserve third-party attribution where applicable. No separate CLA is currently required.

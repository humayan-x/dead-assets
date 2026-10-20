# Maintainer and release guide

## Before making the repository public

- Confirm the repository URL. Metadata currently uses `https://github.com/humayan-x/dead-assets`, inherited from the original README; there was no configured Git remote when these files were prepared. If the destination differs, update README.md, Cargo.toml, and package.json.
- Review the existing MIT OR Apache-2.0 licensing choice and third-party attribution. Check provenance and permissions for anything you add, including media fixtures.
- Review `git status --short --untracked-files=all` and the staged diff before committing. `.gitignore` excludes generated files and common local credentials, but does not remove files already tracked or secrets in Git history. Do not commit tokens or npm credentials.
- Commit `Cargo.lock`, source, tests, manifests, docs, and workflow configuration. Do not commit `target/`, `native/`, `dist/`, `node_modules/`, or release `.tgz` files.
- Configure the Git remote and push the initial branch. Keep the default branch as `main`, or update the CI branch filter.
- Add a repository description and relevant topics (Rust, CLI, assets, Astro, Next.js, Vite).

## GitHub settings after the first push

These settings require repository-owner access and cannot be enabled by committing files:

- Enable **Private vulnerability reporting** and confirm security notifications reach a maintainer. Provide a private contact method on the owner's profile for security and conduct reports, or update the policies with a dedicated monitored address.
- Enable Dependabot alerts/security updates and available secret scanning/push protection. The committed Dependabot configuration requests weekly Cargo and GitHub Actions updates; the root npm wrapper has no dependencies and sample application manifests are fixtures.
- After CI has run, protect the default branch with a ruleset requiring pull requests and the three `Test (...)` CI checks. Choose review requirements appropriate to the number of active maintainers.
- Review Actions permissions. Workflows request read-only repository contents and do not use publishing credentials. They run on ordinary `pull_request` events, not privileged `pull_request_target` events.
- Review the Community Standards page and verify that contributor guidelines, licenses, security policy, and templates are visible.

## Validate a release

1. Keep the version in `package.json` and `Cargo.toml` aligned. Run `cargo check` after changing the Rust package version to update `Cargo.lock`.
2. Move the relevant changelog entries into a versioned section only when releasing. Do not invent a release date before publication.
3. If dependencies changed, run `npm run licenses` and review `THIRD-PARTY-LICENSES.txt`. This collects license/notice files from the locked Cargo dependency metadata. Review license changes and any additional attribution obligations; generation is not a license compatibility assessment.
4. Run the checks below and review the package contents. Do not assume a local Linux test validates other platforms.

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
npm run build:native
npm test
npm run pack:local
```

`pack:local` writes `dist/<platform>-<architecture>/<name>-<version>.tgz`. For example, the current Linux x64 package is `dist/linux-x64/dead-assets-0.1.0.tgz`. It contains only the host binary and declares matching OS/CPU restrictions. Its runtime system requirements follow the build host.

## Build the complete npm release

Run **Actions → Build npm package → Run workflow** on the reviewed commit, or push a tag matching the version, such as `v0.1.0`.

The workflow builds/tests Linux x64 and ARM64, macOS Intel and Apple Silicon, and Windows x64. It then combines binaries into one tarball. Download and extract the **npm-package** artifact. The workflow does not publish anything or require an npm token.

`npm pack` from the repository checks version alignment and requires all five native binaries. Do not bypass that check to produce a normal cross-platform release. Local single-platform tarballs are for local verification unless you deliberately want a platform-restricted release.

Check that the intended npm package name is available to your account before the first release. If necessary, change `name` to a scope you control, such as `@your-name/dead-assets`; the executable remains `dead-assets`. The workflow uploads `*.tgz` so scoped package tarball names are supported. Update installation examples after choosing a different name.

From a temporary test project, install the downloaded tarball and run its CLI. Inspect archive contents to ensure it contains only the expected launcher, binaries, documentation, and licenses. Then publish the exact tested artifact:

```bash
npm login
npm publish ./dead-assets-0.1.0.tgz --access public
```

Use the actual tarball filename if the package name/version differs. Publishing requires an npm account authorized for that name and any authentication/2FA requirements set on the account. Never store tokens in the repository. Once published, create the corresponding GitHub release with changelog notes and the exact artifact, and update README status text.

## References

- [GitHub community health guidance](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions)
- [Private vulnerability reporting setup](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository)
- [Cargo lockfile guidance](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
- [Dependabot configuration](https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference)
- [npm package metadata](https://docs.npmjs.com/cli/configuring-npm/package-json/)

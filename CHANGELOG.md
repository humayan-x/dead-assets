# Changelog

User-visible changes are recorded here. No published release is asserted by this file; the initial package version is `0.1.0`.

## Unreleased

### Added

- Rust CLI for discovering media assets, scanning literal references, and reporting storage totals.
- Interactive cleanup, forced cleanup, dry runs, JSON output, and size thresholds.
- Framework convention handling, additional asset directories, and custom extensions.
- npm launcher, local packaging, and a five-platform release-build workflow.
- Contribution guidelines, security policy, code of conduct, issue/PR templates, CI, and dependency update configuration.

### Fixed

- Overlapping reference matches and noncontiguous asset IDs during scanning.
- Duplicate discovery from overlapping asset directories.
- Silent scan errors allowing incomplete scans to proceed to cleanup.
- Cleanup targets and asset directories resolving outside the project.
- Invalid size thresholds, dry-run option spelling, and empty-project JSON output.

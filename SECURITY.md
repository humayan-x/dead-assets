# Security policy

## Reporting a vulnerability

Do not publish exploit details, credentials, or sensitive files in an issue or pull request. Use this repository's **Security → Report a vulnerability** button when private vulnerability reporting is enabled.

If that option is unavailable, ask the repository owner through their GitHub profile for a private reporting channel. A public request should only ask how to contact the maintainer privately; omit the vulnerability details. No security email address has been configured for this project.

Include the affected version or commit, operating system, reproduction steps using synthetic files, expected and actual behavior, and the potential impact. Path traversal, unintended deletion, and unsafe npm distribution behavior are especially relevant. Test only projects and data you control.

This is a volunteer-maintained, early-stage project. There is no guaranteed response time or bug bounty. Security fixes will target the latest code and latest published release; older releases do not have a separate backport commitment.

## Cleanup boundaries

Cleanup permanently removes files identified by a heuristic scan. Review `--dry-run` output, retain a recoverable copy, and avoid concurrent filesystem changes. Computed references and ignored sources may be missed. These documented limitations do not prevent reporting a reproducible data-loss or path-containment bug.

## Maintainer setup

Before accepting public security reports, enable private vulnerability reporting in GitHub repository settings and verify that an owner receives notifications. See [GitHub's setup instructions](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository).

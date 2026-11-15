# Security Policy

[简体中文](SECURITY.md) | **English**

## Supported Versions

plainmark has not yet published any official release, so no security updates are offered at this time.

Once the first version is released, this table will be updated to reflect the actual support scope:

| Version | Supported |
|---|---|
| Latest release | ✅ Receives security updates |
| Older versions | ❌ Please upgrade to the latest version |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public issues.**

Use GitHub's [private vulnerability reporting](https://github.com/lanlan0811/plainmark/security/advisories/new) to submit a report; it is visible only to maintainers.

If you cannot use that feature, you may also notify the maintainer privately through the contact information listed on their GitHub profile.

### Please include

- The type of vulnerability and its impact
- Steps to reproduce or a proof-of-concept
- Affected versions or commits
- If known, a possible fix

### Please do not include

- Details of vulnerabilities already disclosed publicly
- Actual attack data from other people's systems

## Response Process

| Stage | Description |
|---|---|
| Acknowledgment | We confirm and reply as soon as possible after receiving the report |
| Assessment | We determine the scope and severity, and may reach out to you for details |
| Fix | We fix the issue on a private branch and ask you to help verify it |
| Disclosure | After the fix ships in a new version, we credit you publicly (with your consent) |

Please keep the information confidential until the fix is released, so users have time to upgrade.

## Security Design Principles

plainmark's scope keeps its attack surface small. The following principles will be upheld long term:

- **Local first**: Document content is never uploaded or sent over the network; there are no server-side components
- **Least privilege**: We request no system permissions unrelated to functionality
- **Restrained dependencies**: We avoid third-party dependencies unrelated to project scope
- **No telemetry**: No user data is collected or transmitted

## Dependency Security

The project keeps its dependency set small and checks and updates it regularly. If you find an unfixed security issue in one of our third-party dependencies, you are welcome to report it through the process above.

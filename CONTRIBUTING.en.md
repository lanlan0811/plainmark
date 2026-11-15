# Contributing

[简体中文](CONTRIBUTING.md) | **English**

Thanks for your interest in plainmark. This document explains how to get involved.

## Code of Conduct

By participating in this project, you agree to abide by the [Code of Conduct](CODE_OF_CONDUCT.md).

## Project Scope

plainmark aims to be **a Markdown editor that works like Notepad**. Before suggesting a feature, please check whether it fits that scope:

- ✅ Features that make writing simpler and more direct
- ❌ Heavyweight features such as sidebars, plugin systems, accounts, or cloud sync

When trade-offs are needed, "works like Notepad" takes priority over "has every feature".

## Reporting Issues

Please use the [issue templates](https://github.com/lanlan0811/plainmark/issues/new/choose) and include as much of the following as possible:

- **Bug reports**: steps to reproduce, expected behavior, actual behavior, OS version, plainmark version
- **Feature requests**: the use case, why current options fall short, and whether it fits the project scope

Search existing issues first to avoid duplicates.

## Development Setup

### Requirements

- [Node.js](https://nodejs.org/) 20 or later
- [Rust](https://www.rust-lang.org/tools/install) stable toolchain
- Windows 10 or later

### Running Locally

```bash
npm ci                # install frontend dependencies (use ci, not install)
npm run tauri dev     # development mode with hot reload
```

### Checks Before Committing

```bash
npm run build         # frontend build
cargo check --manifest-path src-tauri/Cargo.toml   # Rust compile check
```

## Workflow

1. Fork the repository and branch from `main`, naming branches like `fix/xxx` or `feat/xxx`
2. Make your changes and make sure the checks above pass
3. Write commit messages in **Chinese**, briefly describing *why* rather than *what*
4. Open a Pull Request against `main` and explain the motivation and how you verified the change

### Code Style

- Let tools handle formatting instead of adjusting by hand: `cargo fmt`; for the frontend, follow the existing style
- Line endings are `LF`; this is handled automatically by `.gitattributes`
- Do not add dependencies unrelated to the project scope

## Release Rules

Official versions are built, packaged, and published automatically by GitHub Actions. **Installers built locally are not accepted.**

Pushing a tag matching `v*` triggers the full pipeline:

```
push tag v* → Build (build + package) → all green → Release (publish)
                                          └ failure or cancelled → no release
```

| Stage | Details |
|---|---|
| Trigger | Only a pushed `v*` tag builds and may release; a manual dispatch only builds, never releases |
| Build | `windows-latest`, `npm ci` + `npm run tauri build` |
| Packaging | Both MSI and NSIS installers must be produced; missing either fails the build |
| Release gate | Build and packaging all green, triggered by a tag push, and the tag starts with `v` — all three required |
| Artifact reuse | The release does not rebuild; it reuses the artifacts from the build stage |

Versions follow [Semantic Versioning](https://semver.org/) as `vMAJOR.MINOR.PATCH`, and the tag name must match the `version` in `src-tauri/tauri.conf.json`. Once pushed, a tag is public and must not be deleted or recreated; to fix a release, bump the PATCH number and publish again.

### Release Notes

Before every release, you must commit `docs/releases/<tag>.md` (e.g. `docs/releases/v0.1.0.md`); the Release body is taken from that file.

The release process is therefore:

```bash
# 1. Write and commit the release notes
vim docs/releases/v0.2.0.md
git add docs/releases/v0.2.0.md
git commit -m "docs: 添加 v0.2.0 发布说明"
git push origin main

# 2. Then push the tag — this is what triggers the build and release
git tag v0.2.0
git push origin v0.2.0
```

If the notes file is missing, the release step fails outright. This is intentional: a Release must never go out without notes. Do not edit the Release body on the GitHub website — that would diverge from the file in the repository.

**Maintainer note**: do not remove the all-green gate or the release-notes check from the release workflow, and do not use `continue-on-error` to bypass build failures.

## License

This project is licensed under the [MIT License](LICENSE). By contributing, you agree to license your work under the same terms.

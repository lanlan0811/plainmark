# plainmark

[简体中文](README.md) | **English**

A minimal Markdown desktop editor that feels as simple as Windows Notepad.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Platform: Windows 10+](https://img.shields.io/badge/Platform-Windows%2010%2B-blue.svg)

---

## Overview

plainmark aims to be **a Markdown editor that works like Notepad** — open it and write. No sidebar, no plugin marketplace, no sign-in, no cloud sync.

Most Markdown editors keep growing into IDEs. plainmark cares about one thing only: let you open a file, write, save, and close it.

## Status

> **In development.** No stable release yet.

The application itself is implemented. Download links will be added here once the first version ships.

## Features

- **Notepad-like**: native menu bar (File / Edit / Format / View / Help) and status bar, matching Windows Notepad's layout and shortcuts
- **Markdown syntax highlighting**: colors the source only — no preview pane, the editing surface stays plain text
- **Tabs**: multiple documents in one window, each with its own undo history
- **Lossless encodings**: ANSI / UTF-8 / UTF-8 BOM / UTF-16 LE / UTF-16 BE are detected on open and written back unchanged
- **Switchable line endings**: Windows (CRLF) / Unix (LF) / Macintosh (CR) from the status bar
- **Local first**: files stay on your machine — no network, no uploads
- **Right-click New**: after install, create a Markdown document straight from Explorer's New menu

## Tech Stack

| Layer | Choice |
|---|---|
| Desktop framework | [Tauri](https://tauri.app/) |
| Frontend | [Vue 3](https://vuejs.org/) |
| Backend | Rust |
| Target platform | Windows 10 or later |

Tauri was chosen over Electron for a smaller install size and lower memory usage, matching the "lightweight notepad" goal.

## Building

### Requirements

- [Node.js](https://nodejs.org/) 20 or later
- [Rust](https://www.rust-lang.org/tools/install) stable toolchain
- Windows 10 or later (requires the WebView2 runtime, bundled with Windows 11)

### Local build

```bash
# Install frontend dependencies
npm ci

# Development mode (hot reload)
npm run tauri dev

# Build and package installers
npm run tauri build
```

Build artifacts are placed in `src-tauri/target/release/bundle/`, containing both MSI and NSIS installers.

## Releasing

Official builds, packaging, and releases are **fully automated by GitHub Actions**. Nothing is published from a local machine.

Pushing a tag matching `v*` triggers the whole pipeline:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Once building and packaging both pass, a GitHub Release is created automatically with the installers attached. See [CONTRIBUTING.md](CONTRIBUTING.md#release-rules) for the full rules.

## Contributing

The project is at an early stage, so APIs and structure may change frequently. Issues with suggestions or bug reports are welcome.

- [Contributing Guide](CONTRIBUTING.md) — dev setup, workflow, release rules
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md) — how to report vulnerabilities privately
- [Changelog](CHANGELOG.md)

## License

[MIT](LICENSE) © 2026 lanlan0811

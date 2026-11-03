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

> **Early development.** No releases yet.

The repository currently contains the license, CI/CD configuration, and project documentation. Application code is under development. Download links will be added here once the first version ships.

## Planned Features

- **Minimal UI**: single window, no sidebar, no tabs, no stacked toolbars
- **Markdown syntax highlighting**: keep the source readable while writing
- **Local first**: files stay on your machine — no network, no uploads
- **Lightweight and fast**: quick startup, low memory footprint
- **Works out of the box**: nothing to configure after install

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

Once building and packaging both pass, a GitHub Release is created automatically with the installers attached. See `AGENTS.md` for the full rules.

## Contributing

The project is at an early stage, so APIs and structure may change frequently. Issues with suggestions or bug reports are welcome.

## License

[MIT](LICENSE) © 2026 lanlan0811

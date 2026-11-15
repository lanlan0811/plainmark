# plainmark

[简体中文](README.md) | **English**

A minimal Markdown desktop editor that feels as simple as Windows Notepad.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Platform: Windows 10+](https://img.shields.io/badge/Platform-Windows%2010%2B-blue.svg)
[![Release](https://img.shields.io/github/v/release/lanlan0811/plainmark)](https://github.com/lanlan0811/plainmark/releases/latest)

---

## Overview

plainmark aims to be **a Markdown editor that works like Notepad** — open it and write. No file tree, no plugin marketplace, no sign-in, no cloud sync.

Most Markdown editors keep growing into IDEs. plainmark cares about one thing only: let you open a file, write, save, and close it.

## Download

Grab the latest build from the [Releases](https://github.com/lanlan0811/plainmark/releases/latest) page:

| Installer | Best for |
|---|---|
| `plainmark_<version>_x64-setup.exe` | NSIS installer — small, recommended for individuals |
| `plainmark_<version>_x64_zh-CN.msi` | MSI package — suited to batch distribution in organizations |

Both require **Windows 10 or later**. If you already have an older version, use **Help → Check for Updates** to upgrade automatically — no manual download needed.

## Features

**Notepad-like**

- Native menu bar (File / Edit / Format / View / Help) and status bar, matching Windows Notepad's layout and shortcuts
- Status bar shows line/column, zoom, line endings, and encoding — clickable fields change on the spot
- Unsaved-changes prompt (Save / Don't Save / Cancel), external-change detection, drag-and-drop open, command-line file arguments
- Single instance: launching again opens a new tab in the existing window
- Remembers window size and position, restored on next launch (including maximized)

**Writing**

- **Markdown syntax highlighting**: colors the source only — the editing surface stays plain text and never interrupts you
- **Tabs**: multiple documents in one window, each with its own undo history
- **Visual preview sidebar**: a right-hand preview, resizable (260–900px) and toggleable at any time; width and visibility are remembered
  - GitHub-style tables, task lists, strikethrough, and automatic bare-URL linking
  - Output is sanitized with DOMPurify — scripts and event attributes are always stripped
  - **Drag to reorder**: drag blocks in the preview to change their order; the Markdown source updates to match
- **Drop in images**: drop an image into the window to copy it into an `assets/` folder beside the document, auto-numbering on name clashes and inserting a relative path

**Files**

- **Lossless encodings**: ANSI / UTF-8 / UTF-8 BOM / UTF-16 LE / UTF-16 BE are detected on open and written back unchanged
- **Switchable line endings**: Windows (CRLF) / Unix (LF) / Macintosh (CR) from the status bar
- **Local first**: files stay on your machine — no network, no uploads
- **Right-click New**: after install, create a Markdown document straight from Explorer's New menu

## Tech Stack

| Layer | Choice |
|---|---|
| Desktop framework | [Tauri](https://tauri.app/) |
| Frontend | [Vue 3](https://vuejs.org/) |
| Editor | [CodeMirror 6](https://codemirror.net/) |
| Preview rendering | [markdown-it](https://github.com/markdown-it/markdown-it) + [DOMPurify](https://github.com/cure53/DOMPurify) |
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

Once building and packaging both pass, a GitHub Release is created automatically with the installers attached. See [CONTRIBUTING.en.md](CONTRIBUTING.en.md) for the full rules.

## Contributing

The project is at an early stage, so APIs and structure may change frequently. Issues with suggestions or bug reports are welcome.

- [Contributing Guide](CONTRIBUTING.en.md) — dev setup, workflow, release rules
- [Code of Conduct](CODE_OF_CONDUCT.en.md)
- [Security Policy](SECURITY.en.md) — how to report vulnerabilities privately
- [Changelog](CHANGELOG.en.md)

## Known Limitations

- Printing and page setup are not implemented and are absent from the menu
- The font dialog is a custom UI (WebView2 has no native font picker), so it differs slightly from Notepad's
- Find / Replace uses the editor's built-in panel; Notepad's standalone Chinese dialog is not yet replicated
- plainmark does not take over the `.txt` association, to avoid displacing Notepad as the default
- Remote images in the preview (e.g. shields.io badges) are fetched over the network
- Startup speed is bounded by WebView2 runtime initialization and cannot fully match native Notepad

## License

[MIT](LICENSE) © 2026 lanlan0811

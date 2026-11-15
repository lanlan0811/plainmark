# Changelog

[简体中文](CHANGELOG.md) | **English**

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.7] - 2026-11-04

### Changed

- **Redesigned the About dialog**: adds runtime environment details (OS version, architecture, install type, Tauri version, webview version) and project links; previously it showed only the version number and a one-line description
- **Startup path optimizations**: compilation now optimizes for speed (`opt-level` changed from `s` to `3`); window-size clamping moved off the main thread so it no longer blocks window creation
- Added startup timing instrumentation, enabled with the `PLAINMARK_STARTUP_LOG=1` environment variable, to locate slow stages

## [0.1.6] - 2026-11-03

### Fixed

- **Find / Replace panel displayed in English**: built-in labels such as Find / Replace / next / match case are now localized to Chinese
- **Dialog titles showed "tauri.localhost says"**: previously used browser-native dialogs, whose titles come from the page origin. Alerts and confirmations now use native system dialogs (with Chinese titles and buttons), and Go to Line uses a custom input box
- The About dialog always showed version 0.1.0; it now shows the actual version

## [0.1.5] - 2026-11-03

### Added

- **Remember window size and position**: after resizing or moving the window, the same state is restored on next launch (including maximized)
- **New app icon**: a document with an "M" and a blue caret, with a transparent background so it no longer appears as a white square on dark taskbars

### Fixed

- **Window filled the screen on open**: the default size dropped from 1000×700 to 900×620; on first launch it is clamped to the display's available area (width ≤80%, height ≤78%), and subsequent launches restore the user's adjusted size; New Window now inherits the main window's size

## [0.1.4] - 2026-11-03

### Added

- **Auto-update**: silently checks GitHub Releases after startup and prompts when a new version is found; after confirmation it downloads with progress, installs, and restarts automatically. Help → Check for Updates triggers it manually
- **Installer-type detection**: updates fetch the matching installer (exe / msi) based on how the current app was installed, with no user choice required
- Update packages are verified with a digital signature to prevent installers from being swapped in transit

## [0.1.3] - 2026-11-03

### Improved

- **Eliminated the startup white flash**: the window is shown only after the first frame renders, hiding the blank period during WebView2 initialization; a 3-second fallback forces it to show so the app is never invisible if the frontend fails
- **Faster startup**: six serial waits on the startup path now run in parallel; the preview renderer (markdown-it + DOMPurify, ~128 KB) loads on demand, cutting first-screen JavaScript from 784 KB to 638 KB (a 17% reduction)

> Note: plainmark's startup speed cannot fully match native Notepad — Tauri must start the WebView2 runtime before rendering the UI, and that fixed cost cannot be removed. This optimization addresses the white-flash perception issue.

## [0.1.2] - 2026-11-03

### Fixed

- **Preview did not update after switching tabs or opening a file**: the preview previously synced only when the editor content changed, while switching tabs, opening files, and reloading after external changes went through `setState()`, which did not fire that callback. The preview kept showing stale content or "document is empty" after switching documents until a character was typed. It is now driven by a unified revision number and covers every path.

### Changed

- **Removed the element panel**: the element-template panel added in the previous version was a misreading of the requirement (the intent was to drag existing elements in the preview to adjust layout, not to insert new ones). Drag-to-reorder remains. Removing the panel stops the preview sidebar from being squeezed and also fixes content truncation at the bottom.

## [0.1.1] - 2026-11-03

### Added

- **Visual Markdown preview**: a right-hand sidebar, resizable (260–900px) and toggleable, with width and visibility persisted
  - Renders GitHub-style tables, task lists, and strikethrough, and auto-links bare URLs
  - Output is sanitized with DOMPurify; scripts and event attributes are stripped
  - The preview is read-only; text cannot be edited
  - **Drag to reorder**: drag blocks in the preview to change their order; the Markdown source updates to match
  - **Element panel**: templates for badges, dividers, blockquotes, code blocks, tables, and task lists, which can be dragged into place or inserted by click
  - **Drop in images**: images are copied into an `assets/` folder beside the document, auto-numbered on name clashes, and inserted as relative paths
- View menu gains toggles for Preview Sidebar and Element Panel

### Changed

- **The UI now has its own design**: features still align with Notepad, but the visuals use a distinct design language (brand colors, rounded corners, depth) instead of replicating Notepad's look
- Editor line height changed from 1.2 to 1.6, with more padding

### Fixed

- Duplicate entries appeared in the right-click New menu: only `.md` is registered now, matching Notepad's approach of registering only `.txt`
- Old duplicate registrations were not cleaned up on upgrade: cleanup moved from the uninstall hook to the install hook
- File → Exit closed only the current window: it now exits the whole application
- External-change detection was not implemented: the app now compares modification times when the window regains focus and asks whether to reload
- Unrepresentable characters were saved as `&#nnnn;` in ANSI: they now become `?`, matching Notepad

## [Unreleased]

### Added

- **The application itself**: a Markdown editor built on Tauri 2 + Vue 3 + Rust
  - Native menu bar structured like Windows Notepad: File / Edit / Format / View / Help
  - Tabbed multi-document editing, each tab with its own undo history
  - CodeMirror 6 editor core styled to match Notepad (no line numbers, no active-line highlight, tab size 8)
  - Markdown syntax highlighting (plain-text editing, no preview panel)
  - Encoding support: ANSI / UTF-8 / UTF-8 BOM / UTF-16 LE / UTF-16 BE, detected on open and written back unchanged
  - Line-ending support: Windows (CRLF) / Unix (LF) / Macintosh (CR), switchable from the status bar
  - Status bar shows line, column, zoom, line endings, and encoding
  - Unsaved-changes prompt (Save / Don't Save / Cancel), external-change detection, drag-and-drop open, command-line file arguments
  - Single instance: launching again opens a new tab in the existing window
- **Windows Shell integration**: associates `.md` / `.markdown` and adds "Markdown Document" to the right-click New menu
  - NSIS writes the `ShellNew` key via `windows/hooks.nsh`
  - MSI writes the same key via `windows/fragments/shellnew.wxs`
- **Project infrastructure**: MIT license, Chinese and English READMEs, CI/CD (build, package, and tag-gated release), open-source collaboration docs, app icon

### Notes

The first official version has not been released yet. This section will be replaced with versioned records after release.

# Changelog

All notable changes to the **Titan Planet** project will be documented in this file.

## [26.8.2] - 2026-01-14

### 🏎️ Performance Optimizations
- **10x Faster Rust Reloads**:
  - Enabled **Incremental Compilation** (`CARGO_INCREMENTAL: "1"`) for development builds.
  - Enabled **Multi-core Compilation**: Removed restrictive CPU thread limits to fully utilize system resources.
  - **Optimized Dev Profile**: Added a custom `[profile.dev]` to `Cargo.toml` with `opt-level = 0` and `debug = 1` for significantly faster linking times.
- **Snappier Dev Loop**: 
  - Reduced hot-reload stability threshold from 1000ms to **300ms**.
  - Optimized the ready-signal detection to launch the server immediately after a successful build.

### ✨ Developer Experience (DX) Overhaul
- **Premium "Orbiting" Experience**:
  - Replaced messy build logs with a sleek, custom animated **"Stabilizing" spinner**.
  - Implemented **Silent Builds**: Cargo compilation noise is hidden by default and only automatically revealed if an error occurs.
  - **Smart Log Forwarding**: ASCII art and runtime logs are now flawlessly flushed to the terminal as soon as the server is ready.
- **Clean CLI**: Removed the Node.js `[DEP0190]` security warning by switching to direct process execution instead of shell-wrapping.

### 🐛 Fixes
- Fixed "Premature Orbiting": The dev server now waits for the server to be fully responsive before showing the success checkmark.
- Improved version detection to correctly reflect the Titan CLI version across all project structures.
- Fixed stuck spinner when `cargo` was not found in the path.

## [26.8.0] - 2026-01-14

### 🚀 New Features
- **Hybrid Rust + JS Actions (Beta)**: You can now mix `.js` and `.rs` actions in the same project. Titan automatically compiles and routes them.
  - Added "Rust + JavaScript (Beta)" option to `titan init`.
  - Added support for compiling `app/actions/*.rs` files into the native binary.
  - Unified `t` runtime API usage across both JS and Rust actions.
- **Enhanced Dev Mode UI**:
  - `titan dev` now features a cleaner, more informative startup screen.
  - Added "Orbit Ready" success messages with build time tracking: *"A new orbit is ready for your app in 0.3s"*.
  - Dynamic detection of project type (JS-only vs. Hybrid).
- **Interactive Init**: `titan init` now prompts for template selection if not specified via flags.

### 🛠 Improvements
- **Reduced Verbosity**:
  - Silenced excessive logging during extension scanning.
  - Simplified bundling logs ("Bundling 1 JS actions..." instead of listing every file).
- **Performance**:
  - Validated incremental compilation settings for Windows stability.
  - Optimized file watching for hybrid projects.

### 🐛 Fixes
- Fixed file locking issues (`os error 32`) on Windows during rapid reloads.
- Fixed `getTitanVersion` to correctly resolve the installed CLI version.
- Unified logging logic between JS and Rust templates for consistency.

---

## [26.7.x] - Previous Releases
- Initial stable release of the JavaScript Action Runtime.
- Added `t.fetch`, `t.jwt`, and `t.password` APIs.
- Integrated `titan dev` hot reload server.

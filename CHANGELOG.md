# Changelog

All notable changes to the Inky project will be documented in this file.

## 2.0.0-beta.2

### Added

- **AI agent support for scaffolded projects.** `inky init` now generates an `AGENT.md` file with project conventions, component syntax, and command reference for AI coding assistants. Symlinks are created for auto-discovery by Claude Code (`CLAUDE.md`), Cursor (`.cursorrules`), and GitHub Copilot (`.github/copilot-instructions.md`).
- **`--json` flag** for `build`, `validate`, and `spam-check` commands. Outputs structured JSON with file results, diagnostics (severity, rule, message), and a summary with error/warning counts. Useful for CI pipelines, editor integrations, and AI agents.
- **Stdin support for `validate` and `spam-check`.** Both commands now read from stdin when no input path is given, matching the existing `build` command behavior. Example: `echo '<img src="x.jpg">' | inky validate --json`
- **`serialize` feature in `inky-core`** for optional serde support on `Diagnostic` and `Severity` types.

### Changed

- Repository and readme metadata added to all crate manifests for crates.io.
- SCSS files moved into `inky-cli` crate for self-contained crates.io packaging.

## 2.0.0-beta.1

Complete rewrite of Inky in Rust. Inky v2 is a ground-up reimplementation that replaces the original Node.js package with a fast, portable Rust core distributed as a CLI binary, WASM module, native shared library, and Rust crate.

### Highlights

- **Rewritten in Rust** with a single-pass component transformation engine.
- **27 email components** covering layout, content, media, navigation, and utilities.
- **Modern v2 syntax** using attributes instead of classes (`size="large"` instead of `class="large"`), with full v1 backward compatibility.
- **Built-in CSS inlining** enabled by default.
- **SCSS framework** with per-template variable overrides via `<style type="text/scss">` or `<link rel="stylesheet">`.
- **Layouts, includes, and custom components** for template composition. Custom `<ink-*>` tags resolve to component files with parameter passing and `<yield>` content injection.
- **Data merging** with Jinja2-compatible syntax (powered by MiniJinja). Supports per-template JSON data files.
- **22 validation rules** checking accessibility, rendering quirks (Gmail clipping, Outlook CSS), spam triggers, and more.
- **Hybrid output mode** generating `<div>` layouts with MSO ghost table fallbacks for modern email clients.
- **Bulletproof buttons** with VML fallbacks for Outlook.
- **Plain text generation** for multipart email.
- **Live preview server** (`inky serve`) with auto-reload on file changes.
- **File watcher** (`inky watch`) with intelligent partial/layout dependency tracking.
- **Spam checker** (`inky spam-check`) for common spam trigger detection.
- **v1 to v2 migration tool** (`inky migrate`) with automatic syntax conversion.
- **Project scaffolding** (`inky init`) with working example templates.
- **Template-friendly** — auto-detects and preserves Handlebars, ERB, Jinja2, Twig, Blade, Mailchimp, and Salesforce merge tag syntax.
- **Cross-platform** — runs on macOS, Linux, and Windows.

### Distribution

- **CLI binary** (`inky`) via Homebrew, npm, cargo, and direct download.
- **WASM module** (`inky-wasm`) for browser and Node.js via npm.
- **Native shared library** (`inky-ffi`) with C FFI bindings.
- **Language bindings** for Node.js, PHP, Python, Ruby, and Go.

### CLI Commands

- `inky build` — Transform templates to email-safe HTML.
- `inky validate` — Check templates for common issues.
- `inky spam-check` — Detect spam triggers.
- `inky migrate` — Convert v1 syntax to v2.
- `inky init` — Scaffold a new project.
- `inky watch` — Watch and auto-rebuild on changes.
- `inky serve` — Live preview dev server.

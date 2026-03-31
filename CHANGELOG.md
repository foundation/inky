# Changelog

All notable changes to the Inky project will be documented in this file.

## 2.0.0-beta.6

### Fixed

- Another attempt at fixing the NPM publish via Github Actions.


## 2.0.0-beta.5

### Fixed

- **Media queries preserved in output.** CSS `@media` rules (responsive breakpoints, dark mode) were being silently dropped during CSS inlining. They are now correctly preserved in a `<style>` block.
- **NPM package name.** The WASM crate has been renamed from `inky-wasm` to `inky` so that `wasm-pack build` generates the correct npm package name directly, eliminating the need for post-build patching.

### Changed

- **Minified CSS output.** Inline `style` attributes are now minified (no spaces after colons), and the framework SCSS compiles in compressed mode. This reduces email size and helps stay under Gmail's 102KB clipping limit.
- **HTML comments stripped.** Regular HTML comments are removed from build output. MSO conditional comments (`<!--[if mso]>`) are preserved.
- **Dead vendor prefixes removed.** Removed `-moz-box-sizing`, `-webkit-box-sizing`, `-ms-text-size-adjust`, `-moz-hyphens`, and `-ms-interpolation-mode` from the framework CSS. Kept `-webkit-text-size-adjust` and `-webkit-hyphens` which are still needed for iOS Mail.
- **Code refactoring.** Major internal cleanup: split `validate.rs` into modules, extracted shared helpers, introduced `BuildContext` struct, replaced component dispatch with data-driven table, converted static regexes to `LazyLock`, and deduplicated ~500 lines across the CLI.

## 2.0.0-beta.4

### Changed

- **NPM Package** The npm package will continue to use `inky` instead of `inky-wasm`


## 2.0.0-beta.3

### Changed

- **Deployment Automation** I did quite a bit of work on the release automation. This build has no new features from beta 2. But I want to test the release automation.


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
- **WASM module** (`inky` on npm) for browser and Node.js.
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

# Inky

Inky is an HTML-based templating language that converts simple HTML into complex, responsive email-ready HTML. Designed for [Foundation for Emails](https://get.foundation/emails).

> **Note:** This is the `v2.0` branch — a complete rewrite in Rust. For the legacy JavaScript version, see the `master` branch.

Give Inky simple HTML like this:

```html
<row>
  <columns large="6">Left</columns>
  <columns large="6">Right</columns>
</row>
```

And get email-ready HTML like this:

```html
<table class="row">
  <tbody>
    <tr>
      <th class="small-12 large-6 columns first">
        <table><tbody><tr><th>Left</th></tr></tbody></table>
      </th>
      <th class="small-12 large-6 columns last">
        <table><tbody><tr><th>Right</th></tr></tbody></table>
      </th>
    </tr>
  </tbody>
</table>
```

## What's New in v2

Inky v2 is a ground-up rewrite in Rust. The core engine compiles to a CLI binary, a WASM module, and a native shared library — so it can be used from any language.

- **CLI tool** — `inky build`, `inky validate` from the command line
- **`.inky` file extension** — source templates use `.inky`, compiled output is `.html`
- **CSS inlining** — enabled by default, inlines `<style>` blocks and `<link>` tags
- **Template validation** — catches missing alt text, Gmail clipping risks, Outlook layout issues, and more
- **Language bindings** — planned for PHP, Python, Ruby, Node.js, and Go
- **Same syntax** — all existing v1 templates work without changes

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later)

```bash
# macOS
brew install rust

# Or use rustup (any platform)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Building

```bash
# Clone the repo
git clone https://github.com/foundation/inky.git
cd inky
git checkout feature/2.0-rust

# Build all crates
cargo build

# Build just the CLI
cargo build -p inky-cli

# Build a release (optimized) binary
cargo build -p inky-cli --release
# Binary will be at: target/release/inky
```

## Testing

```bash
# Run all test suites
bin/runtests.sh
```

This runs everything: Rust unit tests, fixture tests, template comparison tests, and CLI integration tests.

To run individual suites:

```bash
# Rust unit & fixture tests
cargo test -p inky-core

# Template comparison tests (requires release build)
cargo build -p inky-cli --release
node tests/compare.js

# CLI integration tests (validate, .inky extension)
bash tests/test-cli.sh
```

## File Extension

Inky source templates use the `.inky` file extension. When you build a `.inky` file, the output is automatically written to a `.html` file in the same directory:

```bash
inky build email.inky        # → writes email.html
inky build email.inky -o out.html  # → writes out.html
```

Both `.inky` and `.html` files are accepted as input. The difference is that `.inky` files auto-generate `.html` output, while `.html` files write to stdout (unless `-o` is specified).

When processing a directory, both `.inky` and `.html` files are picked up.

## CLI Usage

```bash
# Transform a .inky file (auto-outputs .html)
inky build email.inky

# Transform to a specific output file
inky build email.inky -o output.html

# Transform a directory
inky build src/ -o dist/

# Pipe from stdin
echo '<button href="https://example.com">Click</button>' | inky build

# Skip CSS inlining (inlining is on by default)
inky build email.inky --no-inline-css

# Custom column count
inky build email.inky --columns 16

# Validate templates
inky validate email.inky
inky validate src/

# Help
inky --help
inky help build
```

### CSS Inlining

CSS inlining is **enabled by default**. It moves CSS from `<style>` blocks and `<link rel="stylesheet">` tags into inline `style` attributes. Media queries and other at-rules that can't be inlined are preserved in a `<style>` block at the end of `<body>` (to avoid eating into Gmail's ~102KB clipping limit).

```bash
# Default: transform + inline CSS
inky build email.inky

# Skip inlining (for debugging or if you inline separately)
inky build email.inky --no-inline-css
```

The inliner resolves CSS file paths relative to the input file's directory.

### Validation

The `validate` command checks templates for common email issues:

```bash
# Validate a single file
inky validate email.inky

# Validate a directory
inky validate src/
```

| Rule | Severity | What it checks |
|------|----------|----------------|
| `missing-alt` | warning | Images without `alt` text |
| `button-no-href` | error | Buttons without `href` attribute |
| `missing-container` | warning | No `<container>` element (email won't be centered) |
| `missing-preheader` | warning | No preheader/preview text for inbox listings |
| `email-too-large` | warning | HTML > 90KB (Gmail clips at 102KB) |
| `style-block-too-large` | warning | `<style>` block > 8KB (Gmail strips the entire block) |
| `img-no-width` | warning | Images without `width` attribute (breaks Outlook layout) |
| `deep-nesting` | warning | Tables nested > 4 levels (some email clients struggle) |

Exits with code 1 if any warnings or errors are found — useful for CI pipelines.

## Components

### Container

```html
<container>Content</container>
```

### Row

```html
<row>
  <columns large="6">Left</columns>
  <columns large="6">Right</columns>
</row>
```

### Columns

```html
<!-- Full width (default) -->
<columns>Full width content</columns>

<!-- Sized columns -->
<columns small="12" large="4">One third</columns>
<columns small="12" large="8">Two thirds</columns>

<!-- No expander -->
<columns large="12" no-expander>No expander element</columns>
```

### Button

```html
<button href="https://example.com">Click Me</button>
<button href="#" class="small alert expand">Expanded Alert Button</button>
```

### Callout

```html
<callout class="primary">Important message here.</callout>
```

### Menu

```html
<menu>
  <item href="#">Link 1</item>
  <item href="#" target="_blank">Link 2</item>
</menu>
```

### Spacer

```html
<spacer size="16"></spacer>
<spacer size-sm="10" size-lg="20"></spacer>
```

### Horizontal Line

```html
<h-line></h-line>
```

### Wrapper

```html
<wrapper class="header">Content</wrapper>
```

### Block Grid

```html
<block-grid up="3">
  <td>Item 1</td>
  <td>Item 2</td>
  <td>Item 3</td>
</block-grid>
```

### Center

```html
<center>
  <img src="logo.png" alt="Logo">
</center>
```

### Raw

Wrap content in `<raw>` tags to prevent Inky from transforming it:

```html
<raw>
  <table><tr><td>This won't be touched by Inky</td></tr></table>
</raw>
```

## Project Structure

```
inky/
├── Cargo.toml                # Workspace root
├── crates/
│   ├── inky-core/            # Core transformation library
│   │   ├── src/
│   │   │   ├── lib.rs        # Public API: Inky::transform()
│   │   │   ├── components.rs # Component transformations
│   │   │   ├── attrs.rs      # Attribute parsing helpers
│   │   │   ├── config.rs     # Configuration (column count, tag names)
│   │   │   ├── inline.rs     # CSS inlining (feature-gated)
│   │   │   └── validate.rs   # Template validation rules
│   │   └── tests/
│   │       └── fixtures.rs   # JSON fixture test runner
│   │
│   ├── inky-cli/             # CLI binary
│   │   └── src/main.rs       # build, validate commands
│   │
│   ├── inky-wasm/            # WASM bindings (wasm-bindgen)
│   │   └── src/lib.rs
│   │
│   └── inky-ffi/             # C FFI bindings (shared library)
│       ├── src/lib.rs
│       ├── build.rs          # cbindgen header generation
│       └── inky.h            # Generated C header
│
└── tests/
    ├── compare.js            # Template comparison test runner
    ├── test-cli.sh           # CLI integration tests
    ├── fixtures/             # Shared test fixtures
    │   ├── components.json   # 17 component test cases
    │   └── grid.json         # 14 grid test cases
    └── templates/            # Template comparison tests
        ├── *.inky            # 31 source templates
        └── expected/         # Pre-generated expected output
            └── *.html
```

## Crate Overview

| Crate | Purpose | Output |
|-------|---------|--------|
| `inky-core` | Core transformation engine | Rust library |
| `inky-cli` | Command-line tool | `inky` binary |
| `inky-wasm` | Browser/Node.js bindings | `.wasm` module |
| `inky-ffi` | PHP/Python/Ruby bindings | `.so` / `.dylib` / `.dll` |

## Rust API

```rust
use inky_core::{Inky, Config, transform};

// Quick transform with defaults
let html = transform("<button href=\"#\">Click</button>");

// Custom configuration
let config = Config {
    column_count: 16,
    ..Config::default()
};
let inky = Inky::with_config(config);
let html = inky.transform("<row><columns>Content</columns></row>");
```

## Configuration

The default grid uses 12 columns. Override via the CLI (`--columns`) or in code via `Config`.

Tag names for all components can be customized through `Config::components`:

| Component | Default Tag |
|-----------|-------------|
| Button | `button` |
| Row | `row` |
| Columns | `columns` |
| Container | `container` |
| Block Grid | `block-grid` |
| Menu | `menu` |
| Menu Item | `item` |
| Callout | `callout` |
| Spacer | `spacer` |
| Wrapper | `wrapper` |
| H-Line | `h-line` |
| Center | `center` |
| Inky | `inky` |

## License

MIT

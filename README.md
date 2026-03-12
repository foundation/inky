# Inky

Inky is a complete email framework that converts simple HTML into complex, responsive email-ready HTML. It includes a templating engine, built-in responsive CSS, CSS inlining, validation, and a CLI toolchain.

> Inky was formerly known as "Foundation for Emails." Starting with v2, everything is unified under the Inky brand.

> **Note:** This is the `v2.0` branch — a complete rewrite in Rust. For the legacy JavaScript version, see the `master` branch.

Give Inky simple HTML like this:

```html
<row>
  <column lg="6">Left</column>
  <column lg="6">Right</column>
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

- **CLI tool** — `inky build`, `inky watch`, `inky validate`, `inky init` from the command line
- **`.inky` file extension** — source templates use `.inky`, compiled output is `.html`
- **CSS inlining** — enabled by default, inlines `<style>` blocks and `<link>` tags
- **Framework CSS** — built-in responsive email SCSS framework, with per-template variable overrides
- **Layouts & includes** — `<layout>` and `<include>` tags for composable email templates
- **Template variables** — `$name$` placeholders with `$name|default$` fallback syntax
- **Template validation** — catches missing alt text, Gmail clipping risks, Outlook layout issues, and more
- **Template friendly** — auto-detects and preserves ERB, Jinja2, Handlebars, and other merge tag syntax
- **Watch mode** — rebuilds on file changes, including partials and layouts
- **Project scaffolding** — `inky init` creates a ready-to-go project structure
- **Migration tool** — `inky migrate` converts v1 syntax to v2
- **Language bindings** — planned for PHP, Python, Ruby, Node.js, and Go
- **v2 syntax** — cleaner tag names (`<column>`, `<divider>`) with `sm`/`lg` shorthand; v1 syntax still works with a deprecation warning

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later)

```bash
# macOS
brew install rust

# Or use rustup (any platform)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Quick Start

```bash
# Scaffold a new project
inky init my-email

cd my-email
inky build
```

This creates the following structure:

```
my-email/
├── inky.config.json
├── src/
│   ├── layouts/
│   │   └── default.html        # Base layout with <yield>
│   ├── styles/
│   │   └── theme.scss           # SCSS variable overrides
│   ├── partials/
│   │   ├── header.inky          # Reusable header
│   │   └── footer.inky          # Reusable footer
│   └── emails/
│       └── welcome.inky         # Sample email template
└── dist/                        # Compiled output
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

# Skip framework CSS injection
inky build email.inky --no-framework-css

# Custom column count
inky build email.inky --columns 16

# Strict mode (exit 1 on any warnings)
inky build src/ -o dist/ --strict

# Validate templates
inky validate email.inky
inky validate src/

# Watch for changes and rebuild
inky watch src/emails -o dist

# Migrate v1 syntax to v2
inky migrate src/
inky migrate email.inky --in-place

# Scaffold a new project
inky init my-project

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

### Framework CSS

Inky includes a built-in SCSS framework for responsive email styles. By default, the compiled CSS is injected into each email. You can override SCSS variables inline or in a linked `.scss` file (typically in your layout so it applies to all emails):

```html
<!-- Inline in your layout -->
<style type="text/scss">
$primary-color: #ff6600;
$global-font-family: Georgia, serif;
</style>

<!-- Or link to an external file -->
<link rel="stylesheet" href="theme.scss">
```

```html
<container>
  <row>
    <column sm="12" lg="12">
      <h1>Styled Email</h1>
    </column>
  </row>
</container>
```

To disable framework CSS entirely:

```bash
inky build email.inky --no-framework-css
```

### Watch Mode

Watch mode monitors your source files and rebuilds automatically when changes are detected:

```bash
inky watch src/emails -o dist
```

This watches:
- All `.inky` and `.html` files in the input directory
- Any included partials and layout files (even if outside the input directory)

When a partial or layout changes, all templates are rebuilt. When a single email template changes, only that file is rebuilt.

### Layouts

Layouts let you share a common HTML wrapper across all emails. A layout file contains a `<yield>` tag where the email content will be injected:

```html
<!-- src/layouts/default.html -->
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN"
  "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
  <meta name="viewport" content="width=device-width">
  <title>$title|$</title>
</head>
<body>
  <span class="preheader">$preheader|$</span>
  <table class="body" data-made-with-inky>
    <tr>
      <td class="center" align="center" valign="top">
        <center>
          <yield>
        </center>
      </td>
    </tr>
  </table>
</body>
</html>
```

An email template references its layout with a `<layout>` tag at the top of the file:

```html
<!-- src/emails/welcome.inky -->
<layout src="../layouts/default.html" title="Welcome!" preheader="Thanks for signing up.">
<container>
  <row>
    <column sm="12" lg="12">
      <h1>Welcome!</h1>
    </column>
  </row>
</container>
```

### Includes

Use `<include>` tags to pull in reusable partials:

```html
<include src="../partials/header.inky">

<container>
  <row>
    <column sm="12" lg="12">
      <p>Email content here.</p>
    </column>
  </row>
</container>

<include src="../partials/footer.inky">
```

Includes are resolved recursively (partials can include other partials), with a maximum depth of 10 to catch circular dependencies.

### Template Variables

Both `<layout>` and `<include>` tags support passing variables as attributes. Inside the included file, `$name$` placeholders are replaced with the provided values:

```html
<!-- email template -->
<include src="../partials/header.inky" logo="https://example.com/logo.png">
```

```html
<!-- partials/header.inky -->
<wrapper class="header">
  <container>
    <row>
      <column sm="12" lg="12">
        <img src="$logo$" alt="Logo">
      </column>
    </row>
  </container>
</wrapper>
```

Variables can have default values using the pipe syntax `$name|default$`. If the variable isn't provided, the default is used:

```html
<title>$title|My Email$</title>
<span class="preheader">$preheader|$</span>
```

- `$title|My Email$` — falls back to "My Email" if no `title` attribute is passed
- `$preheader|$` — falls back to empty string
- `$name$` — left as-is if not provided (no default)

### Template Friendly

Inky automatically detects and preserves template language syntax — no need to wrap expressions in `<raw>` tags. The following patterns pass through untouched:

| Syntax | Languages |
|--------|-----------|
| `{{ variable }}` | Handlebars, Mustache, Jinja2, Twig, Blade |
| `<%= expression %>` | ERB, EJS |
| `<% code %>` | ERB, EJS, ASP |
| `{% tag %}` | Jinja2, Twig, Nunjucks, Django |
| `${expression}` | ES6 template literals |
| `*\|MERGE_TAG\|*` | Mailchimp |
| `%%variable%%` | Salesforce Marketing Cloud |

```html
<!-- These just work — no <raw> needed -->
<button href="<%= url_for(@user) %>">Profile</button>
<button href="{% url 'profile' %}">Profile</button>
<row>
  <column>Hello {{ user.name }}</column>
</row>
```

The `<raw>` tag is still available for edge cases where auto-detection doesn't cover your syntax.

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
| `v1-syntax` | warning | Detects deprecated v1 syntax (`<columns>`, `large=`, `small=`, `size=`, `<h-line>`) |
| `missing-alt` | warning | Images without `alt` text |
| `button-no-href` | error | Buttons without `href` attribute |
| `missing-container` | warning | No `<container>` element (email won't be centered) |
| `missing-preheader` | warning | No preheader/preview text for inbox listings |
| `email-too-large` | warning | HTML > 90KB (Gmail clips at 102KB) |
| `style-block-too-large` | warning | `<style>` block > 8KB (Gmail strips the entire block) |
| `img-no-width` | warning | Images without `width` attribute (breaks Outlook layout) |
| `deep-nesting` | warning | Tables nested > 5 levels (some email clients struggle) |

Exits with code 1 if any warnings or errors are found — useful for CI pipelines.

### Migration

Migrate v1 templates to v2 syntax:

```bash
# Preview changes (writes to stdout)
inky migrate email.inky

# Migrate a directory to an output directory
inky migrate src/ -o migrated/

# Rewrite files in-place
inky migrate src/ --in-place
```

The migrator converts:
- `<columns>` → `<column>`
- `<h-line>` → `<divider>`
- `large="N"` → `lg="N"`
- `small="N"` → `sm="N"`
- `<spacer size="N">` → `<spacer height="N">`

## Components

### Container

```html
<container>Content</container>
```

### Row

```html
<row>
  <column lg="6">Left</column>
  <column lg="6">Right</column>
</row>
```

### Column

```html
<!-- Full width (default) -->
<column>Full width content</column>

<!-- Sized columns -->
<column sm="12" lg="4">One third</column>
<column sm="12" lg="8">Two thirds</column>

<!-- No expander -->
<column lg="12" no-expander>No expander element</column>
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
<spacer height="16"></spacer>
<spacer size-sm="10" size-lg="20"></spacer>
```

### Divider

```html
<divider></divider>
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
│   │   │   ├── include.rs    # Layout, include, and variable processing
│   │   │   ├── inline.rs     # CSS inlining (feature-gated)
│   │   │   └── validate.rs   # Template validation rules
│   │   └── tests/
│   │       └── fixtures.rs   # JSON fixture test runner
│   │
│   ├── inky-cli/             # CLI binary
│   │   └── src/
│   │       ├── main.rs       # build, validate, migrate, init commands
│   │       ├── init.rs       # Project scaffolding
│   │       ├── watch.rs      # Watch mode with auto-rebuild
│   │       ├── migrate.rs    # v1 → v2 syntax migration
│   │       └── scss.rs       # SCSS compilation and injection
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
let html = inky.transform("<row><column>Content</column></row>");
```

## Configuration

### Project Config (`inky.config.json`)

Place an `inky.config.json` in your project root to set defaults for the CLI. Inky searches upward from the current directory to find it.

```json
{
  "src": "src/emails",
  "dist": "dist",
  "columns": 12
}
```

| Key | Description | Default |
|-----|-------------|---------|
| `src` | Source directory for templates | — |
| `dist` | Output directory for compiled HTML | — |
| `columns` | Number of grid columns | `12` |

With a config file in place, you can simply run:

```bash
inky build
inky watch
```

CLI flags always take priority over config file values.

### Grid Columns

The default grid uses 12 columns. Override via the CLI (`--columns`), `inky.config.json`, or in code via `Config`.

Tag names for all components can be customized through `Config::components`:

| Component | Default Tag |
|-----------|-------------|
| Button | `button` |
| Row | `row` |
| Column | `column` |
| Container | `container` |
| Block Grid | `block-grid` |
| Menu | `menu` |
| Menu Item | `item` |
| Callout | `callout` |
| Spacer | `spacer` |
| Wrapper | `wrapper` |
| Divider | `divider` |
| Center | `center` |

## License

MIT

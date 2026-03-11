# Inky v2 Plan

## Vision

Inky becomes a complete email framework — absorbing what was previously "Foundation for Emails" into a single product and brand. Inky v2 is a ground-up rethink with:

- **Modern syntax** — attributes over classes, cleaner naming, new components
- **Accessibility by default** — `role="presentation"` on all layout tables, alt text validation
- **Dark mode support** — color-scheme meta, compatible style patterns
- **Inky CLI** — build, migrate, validate from the command line
- **Inky Styles** — SCSS/CSS framework for responsive email components (formerly Foundation for Emails)
- **Inky Templates** — Starter email templates
- **Language Bindings** — Official packages for JS, PHP, Python, Ruby, and Go
- **Legacy migration** — `inky migrate` converts v1 syntax to v2 automatically

## Distribution

The Inky engine is written in Rust and distributed as:

1. **CLI binary** — `inky` command, installable via Homebrew, npm, cargo, or direct download
2. **WASM module** — for JS/Node.js/browser
3. **Native shared library** (.so/.dylib/.dll) — for PHP (via FFI), Python (via ctypes), Ruby (via fiddle), Go (via cgo)
4. **Rust crate** — for Rust consumers and as the canonical source of truth

---

## Modern Syntax (v2)

### Design Principles

1. **Attributes over classes** — explicit, parseable, validatable with useful error messages
2. **Accessibility by default** — layout tables always get `role="presentation"`
3. **Consistent naming** — `sm`/`lg` everywhere, singular `<column>`, clear attribute names
4. **Auto-detected template tags** — `{{...}}`, `<%= %>`, `${...}` pass through without `<raw>`
5. **Helpful errors** — "unknown attribute `colr` on button — did you mean `color`?"

### Component Reference

#### Layout

```html
<container>
  <row align="center" dir="rtl">
    <column sm="12" lg="6">Content</column>
    <column sm="12" lg="6">Content</column>
  </row>
</container>
```

- `<column>` (singular, renamed from `<columns>`)
- `sm` / `lg` (shortened from `small` / `large`)
- `align` attribute on `<row>` (was not supported)
- `no-expander` still supported

#### Button

```html
<button href="https://example.com"
        target="_blank"
        size="small"
        color="alert"
        expand
        radius
        hollow>
  Click Me
</button>
```

- `size` attribute: `tiny`, `small`, `default`, `large` (was `class="small"`)
- `color` attribute: `primary`, `secondary`, `success`, `alert`, `warning` (was `class="alert"`)
- `expand`, `radius`, `rounded`, `hollow` remain as bare attributes (was classes)

#### Spacer

```html
<spacer height="16">
<spacer sm="10" lg="20">
```

- `height` (renamed from `size` — clearer intent)
- `sm` / `lg` (renamed from `size-sm` / `size-lg`)

#### Divider

```html
<divider>
<divider class="dotted">
```

- Renamed from `<h-line>` — more intuitive name
- `<h-line>` still works in legacy mode

#### Callout

```html
<callout color="primary">Important message</callout>
```

- `color` attribute (was `class="primary"`)

#### Menu

```html
<menu align="center" direction="vertical">
  <item href="#">Link 1</item>
  <item href="#" target="_blank">Link 2</item>
</menu>
```

- `align="center"` replaces wrapping in `<center>` tag
- `direction="vertical"` (was `class="vertical"`)

#### Wrapper

```html
<wrapper class="header">Content</wrapper>
```

- Unchanged

#### Block Grid

```html
<block-grid up="3">
  <td>Item</td>
</block-grid>
```

- Unchanged

#### Image (NEW)

```html
<image src="hero.jpg" alt="Hero banner" width="600">
<image src="hero.jpg" alt="Hero banner" width="600" retina>
```

- Responsive image with proper width attributes for email clients
- `retina` flag: renders at half the source width for crisp display on high-DPI screens
- `alt` text is required — parser warns if missing

#### Outlook Conditional (NEW)

```html
<outlook>
  <!-- This content only renders in Outlook/mso -->
  <table width="600"><tr><td>Fallback</td></tr></table>
</outlook>

<not-outlook>
  <!-- This content renders everywhere except Outlook -->
  <div style="max-width: 600px;">Modern layout</div>
</not-outlook>
```

- Wraps content in `<!--[if mso]>...<![endif]-->` / `<!--[if !mso]><!-->...<!--<![endif]-->`
- Much cleaner than writing conditional comments by hand

#### Raw

```html
<raw><<LCG Program\TG LCG Coupon Code>></raw>
```

- Unchanged, still available for edge cases
- Most template tags auto-detected (see below)

### Auto-Detected Template Tags

These patterns pass through untouched without needing `<raw>`:

- `{{variable}}` — Handlebars, Mustache, Jinja2, Twig, Blade
- `<%= expression %>` — ERB, EJS
- `${expression}` — ES6 template literals
- `<% code %>` — ERB, EJS, ASP
- `{variable}` — (only when not valid HTML attribute syntax)
- `*|MERGE_TAG|*` — Mailchimp
- `%%variable%%` — Salesforce Marketing Cloud

### Output Changes (vs v1)

All layout tables include accessibility attributes:
```html
<table role="presentation" class="row">
```

Dark mode meta tag added when transforming a full HTML document:
```html
<meta name="color-scheme" content="light dark">
<meta name="supported-color-schemes" content="light dark">
```

---

## Legacy Syntax (v1 Compatibility)

The parser auto-detects legacy syntax. If it encounters `<columns>` (plural) it uses v1 rules for that element. This means existing templates work without changes, but new code should use the modern syntax.

### Migration Tool

`inky migrate` converts v1 → v2 syntax:

| v1 (Legacy) | v2 (Modern) |
|---|---|
| `<columns large="6" small="12">` | `<column lg="6" sm="12">` |
| `<columns>` (plural) | `<column>` (singular) |
| `<button class="small alert expand">` | `<button size="small" color="alert" expand>` |
| `<spacer size="16">` | `<spacer height="16">` |
| `<spacer size-sm="10" size-lg="20">` | `<spacer sm="10" lg="20">` |
| `<h-line>` | `<divider>` |
| `<callout class="primary">` | `<callout color="primary">` |
| `<center><menu class="vertical">` | `<menu align="center" direction="vertical">` |

The migration preserves all other attributes and content. It's a safe, reversible transformation.

---

## Inky CLI (`inky-cli`)

A standalone command-line tool for transforming, migrating, and validating Inky templates.

### Installation

```bash
# Homebrew
brew install inky

# npm (installs the binary via WASM)
npm install -g inky

# Cargo
cargo install inky-cli

# Direct download (GitHub releases)
curl -fsSL https://get.inky.email/install.sh | sh
```

### Commands

```bash
# Transform files
inky build input.html                    # outputs to stdout
inky build input.html -o output.html     # outputs to file
inky build src/ -o dist/                 # transforms a directory
inky build src/ -o dist/ --inline-css    # transform + inline CSS (future)
cat input.html | inky build              # stdin/stdout pipe

# Migrate v1 → v2
inky migrate old.html                    # prints migrated to stdout
inky migrate old.html -o new.html        # writes to file
inky migrate src/ --in-place             # rewrites files in-place

# Validate templates
inky validate input.html                 # checks for issues
inky validate src/                       # validates a directory

# Watch mode
inky watch src/ -o dist/                 # rebuilds on file changes

# Version and help
inky --version
inky help
inky help build
```

### Validation Checks

`inky validate` warns about:
- Images missing `alt` text
- Buttons missing `href`
- Nesting rows more than 2 levels deep (Gmail issues)
- Unknown attributes on Inky components
- Deprecated v1 syntax (suggests v2 equivalent)

### Exit Codes

- `0` — success
- `1` — transform/parse error
- `2` — validation warnings (with `--strict`)

### Configuration File

Optional `inky.config.json` or `inky` key in `package.json`:

```json
{
  "columnCount": 12,
  "syntax": "modern",
  "validate": true
}
```

---

## Project Structure

```
inky/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── inky-core/                # Pure Rust transformation library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Public API: transform(html) -> html
│   │       ├── components.rs     # Component factory (all transformation rules)
│   │       ├── attrs.rs          # Attribute extraction and filtering
│   │       ├── config.rs         # Configuration (column count, syntax mode)
│   │       ├── migrate.rs        # v1 → v2 syntax migration
│   │       └── validate.rs       # Template validation and warnings
│   │
│   ├── inky-cli/                 # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs           # CLI entry point (clap-based)
│   │
│   ├── inky-wasm/                # WASM bindings (wasm-bindgen)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs            # WASM-exported functions
│   │
│   └── inky-ffi/                 # C FFI bindings (shared library)
│       ├── Cargo.toml
│       ├── src/
│       │   └── lib.rs            # extern "C" exported functions
│       ├── build.rs              # cbindgen header generation
│       └── inky.h                # Generated C header
│
├── scss/                         # Inky Styles (formerly Foundation for Emails)
│   ├── inky.scss
│   ├── _global.scss
│   ├── components/
│   │   ├── _button.scss
│   │   ├── _callout.scss
│   │   ├── _menu.scss
│   │   ├── _normalize.scss
│   │   ├── _typography.scss
│   │   ├── _visibility.scss
│   │   └── _dark-mode.scss       # NEW: dark mode utilities
│   ├── grid/
│   │   ├── _grid.scss
│   │   └── _block-grid.scss
│   └── settings/
│       └── _settings.scss
│
├── dist/                         # Prebuilt CSS
│   ├── inky.css
│   └── inky.min.css
│
├── templates/                    # Starter email templates (v2 syntax)
│   ├── basic.html
│   ├── hero.html
│   ├── newsletter.html
│   └── marketing.html
│
├── bindings/
│   ├── node/                     # npm: "inky"
│   │   ├── package.json
│   │   └── index.js
│   ├── php/                      # Composer: "foundation/inky"
│   │   ├── composer.json
│   │   └── src/Inky.php
│   ├── python/                   # PyPI: "inky-email"
│   │   ├── pyproject.toml
│   │   └── src/inky/__init__.py
│   └── ruby/                     # RubyGems: "inky-email"
│       ├── inky.gemspec
│       └── lib/inky.rb
│
├── tests/
│   └── fixtures/                 # Shared test fixtures (JSON)
│       ├── components.json       # Component test cases
│       ├── grid.json             # Grid/column test cases
│       ├── modern.json           # v2 modern syntax test cases
│       └── migration.json        # v1 → v2 migration test cases
│
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
```

---

## Rust Core API

```rust
pub struct Inky {
    config: Config,
}

pub struct Config {
    pub column_count: u32,
    pub syntax: SyntaxMode,
}

pub enum SyntaxMode {
    Modern,     // v2 syntax (default)
    Legacy,     // v1 syntax
    Auto,       // auto-detect per element
}

impl Inky {
    pub fn new() -> Self;
    pub fn with_config(config: Config) -> Self;

    /// Transform Inky HTML into email-safe table HTML.
    pub fn transform(&self, html: &str) -> String;

    /// Migrate v1 syntax to v2 syntax (no table transformation).
    pub fn migrate(&self, html: &str) -> String;

    /// Validate a template and return warnings.
    pub fn validate(&self, html: &str) -> Vec<Warning>;
}

pub struct Warning {
    pub message: String,
    pub severity: Severity,     // Error, Warning, Info
    pub line: Option<usize>,
    pub suggestion: Option<String>,
}

// Convenience functions
pub fn transform(html: &str) -> String;
pub fn migrate(html: &str) -> String;
pub fn validate(html: &str) -> Vec<Warning>;
```

---

## CLI Crate (`inky-cli`)

```toml
[package]
name = "inky-cli"
version = "2.0.0"

[[bin]]
name = "inky"
path = "src/main.rs"

[dependencies]
inky-core = { path = "../inky-core" }
clap = { version = "4", features = ["derive"] }
glob = "0.3"
notify = "7"           # file watching
colored = "2"          # terminal colors
```

Subcommands:
- `build` — transform files (default if input given)
- `migrate` — convert v1 → v2
- `validate` — check for issues
- `watch` — watch and rebuild

---

## Implementation Order

### Stage 1: Core Engine (current — in progress)

| Step | Task | Status |
|------|------|--------|
| 1 | Set up Cargo workspace (`inky-core`, `inky-wasm`, `inky-ffi`) | Done |
| 2 | Implement legacy (v1) component transformations | Done |
| 3 | Extract test cases to JSON fixtures | Done (31 tests) |
| 4 | Pass all fixture tests | Done |
| 5 | Add modern (v2) syntax support — new components and attributes | TODO |
| 6 | Add `role="presentation"` to all layout table output | TODO |
| 7 | Add auto-detection for template merge tags | TODO |
| 8 | Add `<image>`, `<outlook>`, `<not-outlook>`, `<divider>` components | TODO |

### Stage 2: CLI + Migration

| Step | Task |
|------|------|
| 9 | Create `inky-cli` crate with `clap` |
| 10 | Implement `build` command (file/directory/stdin) |
| 11 | Implement `migrate` command (v1 → v2 syntax conversion) |
| 12 | Implement `validate` command |
| 13 | Implement `watch` command |
| 14 | Write modern syntax fixture tests |
| 15 | Write migration fixture tests |

### Stage 3: Distribution

| Step | Task |
|------|------|
| 16 | Build `inky-wasm` with wasm-bindgen |
| 17 | Build `inky-ffi` with cbindgen |
| 18 | Create Node.js wrapper (WASM) |
| 19 | Create PHP Composer package (FFI) |
| 20 | Create Python PyPI package (ctypes) |
| 21 | Create Ruby gem (fiddle) |
| 22 | Create Go module (cgo, separate repo) |

### Stage 4: Styles + Templates

| Step | Task |
|------|------|
| 23 | Move SCSS from `foundation-emails` into `inky/scss/` |
| 24 | Rename entry point to `inky.scss`, update variable prefixes |
| 25 | Add dark mode utilities (`_dark-mode.scss`) |
| 26 | Add styles for new components (image, divider) |
| 27 | Build dist CSS with `sass` command (no gulp) |
| 28 | Create starter templates using v2 syntax |

### Stage 5: Ship It

| Step | Task |
|------|------|
| 29 | Set up CI: test Rust + all bindings on all platforms |
| 30 | Set up release pipeline: cross-compile + publish everywhere |
| 31 | Homebrew formula for `inky` CLI |
| 32 | Write documentation and migration guide |
| 33 | Publish v2.0.0 to npm, crates.io, Packagist, PyPI, RubyGems, Homebrew |
| 34 | Archive `foundation/inky-rb` with pointer to new gem |
| 35 | Archive `foundation/foundation-emails` with pointer to `inky` |
| 36 | Re-enable Dependabot |

### Future (v2.x / v3.0)

| Feature | Description |
|---------|-------------|
| Hybrid output mode | `<div>` layout with Outlook table fallbacks |
| CSS inlining | Built into `inky build --inline-css` |
| Contrast checker | Validates color accessibility |
| Template variables | Built-in `{{var}}` support with data files |
| Live preview | `inky serve` with browser preview |


### Good list of current Email frameworks

https://www.emailonacid.com/blog/article/email-development/best-email-frameworks/
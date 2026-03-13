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

## Legacy Syntax (v1 → v2 Migration)

The v2 parser is **strict v2 only** — it does not support v1 syntax. If the
parser encounters v1 syntax (e.g. `<columns>` instead of `<column>`), it will
output a helpful error message pointing the user to `inky migrate`.

This keeps the parser simple, avoids ambiguity when old and new syntax mix,
and encourages a clean migration. Users run `inky migrate` once and are done.

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
│   │   ├── src/
│   │   │   ├── Inky.php          # Main API (auto-detects best driver)
│   │   │   └── Driver/
│   │   │       ├── DriverInterface.php
│   │   │       ├── ExtensionDriver.php  # PECL C extension (production)
│   │   │       └── FfiDriver.php        # FFI (dev / self-managed servers)
│   │   ├── ext/                  # PHP C extension source
│   │   │   ├── config.m4
│   │   │   ├── inky.c
│   │   │   └── php_inky.h
│   │   ├── stubs/
│   │   │   └── inky.h            # FFI header (copy from inky-ffi)
│   │   └── preload.php           # opcache.preload script for FFI mode
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
}

impl Inky {
    pub fn new() -> Self;
    pub fn with_config(config: Config) -> Self;

    /// Transform v2 Inky HTML into email-safe table HTML.
    /// Returns an error if v1 syntax is detected (directs user to migrate()).
    pub fn transform(&self, html: &str) -> Result<String, Vec<Warning>>;

    /// Migrate v1 syntax to v2 syntax (no table transformation).
    /// This is a text-level conversion — it does not produce table output.
    pub fn migrate(&self, html: &str) -> String;

    /// Transform and inline CSS in one step.
    pub fn transform_and_inline(&self, html: &str, base_path: Option<&Path>) -> Result<String, String>;

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

### Stage 1: Core Engine — COMPLETE

| Step | Task | Status |
|------|------|--------|
| 1 | Set up Cargo workspace (`inky-core`, `inky-wasm`, `inky-ffi`) | Done |
| 2 | Implement v1 component transformations (porting JS behavior) | Done |
| 3 | Extract test cases to JSON fixtures | Done (31 tests) |
| 4 | Pass all fixture tests | Done |
| 5 | Add CSS inlining support (`--inline-css`) | Done |
| 6 | Copy foundation-emails visual test templates for comparison | Done |
| 7 | Fix comparison failures (20/31 passing, 11 remaining edge cases) | Done (31/31) |
| 8 | Convert parser from v1 syntax to strict v2 syntax | Done |
| 9 | Add `role="presentation"` to all layout table output | Done |
| 10 | Add auto-detection for template merge tags | Done |
| 11 | Add `<image>`, `<outlook>`, `<not-outlook>`, `<divider>` components | Done |

### Stage 2: CLI + Migration — COMPLETE

| Step | Task | Status |
|------|------|--------|
| 12 | Create `inky-cli` crate with `clap` | Done |
| 13 | Implement `build` command (file/directory/stdin) | Done |
| 14 | Implement `migrate` command (v1 → v2 syntax conversion) | Done |
| 15 | Implement `validate` command (source + output checks, 16 tests) | Done |
| 16 | Add v1 syntax detection with helpful error messages | Done |
| 17 | Implement `watch` command (with SCSS/layout/include change detection) | Done |
| 18 | Write v2 syntax fixture tests (48 tests across v2-components.json + v2-grid.json) | Done |
| 19 | Write migration fixture tests (17 tests in migration.json) | Done |
| 20 | Implement `init` command (project scaffolding with themes) | Done |
| 21 | Implement `<include>` tag support (template partials/layouts) | Done |
| 22 | Implement `<layout>` + `<yield>` system | Done |
| 23 | Add SCSS compilation and `<link>` support in CLI | Done |
| 24 | Add `inky.config.json` auto-discovery | Done |
| 25 | Blank line cleanup and DRY refactoring | Done |
| 26 | Build WASM playground | Done |

### Stage 3: Distribution — COMPLETE (Go deferred to separate repo)

| Step | Task | Status |
|------|------|--------|
| 27 | Build `inky-wasm` with wasm-bindgen (+ CSS inlining) | Done |
| 28 | Build `inky-ffi` with cbindgen (transform, migrate, validate, inline, version) | Done |
| 29 | Create Node.js wrapper (WASM) — 32 tests | Done |
| 30 | Create PHP Composer package (FFI driver) — 32 tests | Done |
| 31 | Create Python PyPI package (ctypes) — 32 tests | Done |
| 32 | Create Ruby gem (fiddle) — 32 tests | Done |
| 33 | Create Go module (cgo, separate repo) | Deferred |

### Stage 4: Styles + Templates — COMPLETE

| Step | Task | Status |
|------|------|--------|
| 34 | Move SCSS from `foundation-emails` into `inky/scss/` | Done |
| 35 | Rename entry point to `inky.scss`, update variable prefixes | Done |
| 36 | Add dark mode utilities (`_dark-mode.scss`) | Done |
| 37 | Add styles for new components (`_divider.scss` — divider, h-line, image) | Done |
| 38 | Build dist CSS with `sass` command (36KB / 30KB min) | Done |
| 39 | Create starter templates using v2 syntax | Done (init scaffolds themes) |

### Stage 5: Ship It

| Step | Task | Status |
|------|------|--------|
| 40 | Set up CI: test Rust + all bindings on all platforms | TODO |
| 41 | Set up release pipeline with `cargo-dist` (cross-compile + GitHub Releases) | TODO |
| 42 | Create `foundation/homebrew-inky` tap repo with formula | TODO |
| 43 | Automate Homebrew formula updates on release (via `cargo-dist`) | TODO |
| 44 | Write documentation and migration guide | TODO |
| 45 | Publish v2.0.0 to npm, crates.io, Packagist, PyPI, RubyGems, Homebrew | TODO |
| 46 | Archive `foundation/inky-rb` with pointer to new gem | TODO |
| 47 | Archive `foundation/foundation-emails` with pointer to `inky` | TODO |
| 48 | Submit formula to `homebrew-core` (once project is established) | TODO |
| 49 | Re-enable Dependabot | TODO |

---

## PHP Bindings

The PHP package supports two drivers. The `Inky` class auto-detects the best
available driver and provides helpful errors when neither is available.

### Driver Priority

| Priority | Driver | Mechanism | Best For |
|----------|--------|-----------|----------|
| 1 | C Extension | PECL `ext-inky` | Shared hosting, production |
| 2 | FFI | `ext-ffi` + `libinky.so` | Local dev, self-managed servers |

### C Extension (Recommended for Production)

A thin PHP C extension that links against `libinky` (the shared library from
`inky-ffi`). Works on any PHP install without special `php.ini` settings.

```bash
# Install via PECL
pecl install inky

# Or compile from source
cd bindings/php/ext
phpize
./configure
make && make install
```

Add to `php.ini`:
```ini
extension=inky
```

### FFI Driver

Loads `libinky.so` directly via PHP's built-in FFI extension. No compilation
needed, but requires FFI to be enabled.

**Option A: Enable globally** (local dev)
```ini
ffi.enable = true
```

**Option B: Preload mode** (production, more secure)
```ini
ffi.enable = preload
opcache.preload = /path/to/vendor/inky/preload.php
```

Preload mode only allows FFI in the preload script (which runs once at PHP
startup), not in arbitrary runtime code. Note: `opcache.preload` only works
with PHP-FPM, and only one preload file is allowed per installation — if the
app already has a preload script, add an `include` for Inky's preload inside it.

**Important:** `ffi.enable` is a `PHP_INI_SYSTEM` directive — it cannot be
changed at runtime with `ini_set()`. It must be set in `php.ini`, a vhost
config, or via the `-d` CLI flag.

### Auto-Detection and Error Messages

```php
use Inky\Inky;

// Just works — picks the best available driver
$html = Inky::transform('<button href="#">Click</button>');
```

The `Inky` class checks drivers in priority order and throws a clear exception
if none are available:

```
Inky requires either the 'inky' PHP extension (recommended) or the 'ffi'
extension with ffi.enable=true in php.ini. See https://inky.email/php for
setup instructions.
```

### Why Not exec() / CLI Fallback?

Most production PHP hosting blocks `exec()`, `shell_exec()`, `proc_open()`,
and similar functions for security. This makes a CLI-based fallback unreliable
for the PHP audience, which is why the C extension is the primary target.

---

## Homebrew Distribution

### Phase 1: Custom Tap (launch)

Create a `foundation/homebrew-inky` repo with a formula that downloads
prebuilt binaries from GitHub Releases.

```
foundation/homebrew-inky/
└── Formula/
    └── inky.rb
```

Users install with:
```bash
brew tap foundation/inky
brew install inky
```

### Phase 2: Homebrew Core (once established)

Submit a PR to `homebrew/homebrew-core` so users can install with just:
```bash
brew install inky
```

Homebrew Core requires: tagged releases, notable project, builds from source.

### Release Automation

Use [`cargo-dist`](https://github.com/axodotdev/cargo-dist) to automate the
full release pipeline:

1. Cross-compile CLI binaries for all targets:
   - `x86_64-apple-darwin` (macOS Intel)
   - `aarch64-apple-darwin` (macOS Apple Silicon)
   - `x86_64-unknown-linux-gnu` (Linux x86)
   - `aarch64-unknown-linux-gnu` (Linux ARM)
   - `x86_64-pc-windows-msvc` (Windows)
2. Create GitHub Release with tarballs and SHA256 checksums
3. Auto-update the Homebrew formula with new version and hashes
4. Publish to `crates.io`

This runs as a GitHub Action triggered by tagging a release (`v2.0.0`).

### Formula Example

```ruby
class Inky < Formula
  desc "Transform email templates into email-safe HTML"
  homepage "https://github.com/foundation/inky"
  version "2.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/foundation/inky/releases/download/v2.0.0/inky-aarch64-apple-darwin.tar.gz"
      sha256 "..."
    else
      url "https://github.com/foundation/inky/releases/download/v2.0.0/inky-x86_64-apple-darwin.tar.gz"
      sha256 "..."
    end
  end

  on_linux do
    url "https://github.com/foundation/inky/releases/download/v2.0.0/inky-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "..."
  end

  def install
    bin.install "inky"
  end

  test do
    assert_match "Inky", shell_output("#{bin}/inky --version")
  end
end
```

---

### Future (v2.x / v3.0)

| Feature | Description |
|---------|-------------|
| Hybrid output mode | `<div>` layout with Outlook table fallbacks |
| Contrast checker | Validates color accessibility |
| Template variables | Built-in `{{var}}` support with data files |
| Live preview | `inky serve` with browser preview |


### Good list of current Email frameworks

https://www.emailonacid.com/blog/article/email-development/best-email-frameworks/
# Inky

> **Pre-release:** v2 is not yet published. To test the CLI, build from source:
>
> ```bash
> # Requires Rust — https://rustup.rs
> git clone https://github.com/foundation/inky.git
> cd inky
> git checkout feature/2.0-rust
> cargo build -p inky-cli --release
>
> # Copy the binary somewhere on your PATH
> cp target/release/inky /usr/local/bin/
> ```
>
> Then run `inky --version` to verify. Report issues on the `feature/2.0-rust` branch.

Inky is a complete email framework that converts simple HTML into complex, responsive email-ready HTML. It includes a templating engine, built-in responsive CSS, CSS inlining, validation, and a CLI toolchain.

Written in Rust. Distributed as a CLI binary, WASM module, native shared library, and Rust crate — with official bindings for Node.js, PHP, Python, Ruby, and Go.

> Inky was formerly known as "Foundation for Emails." Starting with v2, everything is unified under the Inky brand.

Give Inky simple HTML like this:

```html
<container>
  <row>
    <column lg="6">Left</column>
    <column lg="6">Right</column>
  </row>
</container>
```

And get email-ready HTML like this:

```html
<table role="presentation" align="center" class="container">
  <tbody><tr><td>
    <table role="presentation" class="row">
      <tbody><tr>
        <th class="small-12 large-6 columns first">
          <table role="presentation"><tbody><tr><th>Left</th></tr></tbody></table>
        </th>
        <th class="small-12 large-6 columns last">
          <table role="presentation"><tbody><tr><th>Right</th></tr></tbody></table>
        </th>
      </tr></tbody>
    </table>
  </td></tr></tbody>
</table>
```

## Install

```bash
# Homebrew
brew tap foundation/inky && brew install inky

# Cargo
cargo install inky-cli

# npm (WASM)
npm install inky-email
```

## Quick Start

```bash
# Scaffold a new project
inky init my-email
cd my-email

# Build
inky build

# Watch for changes
inky watch
```

## What's New in v2

- **Rust rewrite** — fast, single binary, cross-platform
- **Modern syntax** — attributes over classes (`size="small"` instead of `class="small"`)
- **27 components** — layout, buttons, cards, alerts, hero sections, social links, video, and more
- **CSS inlining** — built-in, enabled by default
- **SCSS framework** — responsive email styles with per-template variable overrides
- **Layouts & includes** — `<layout>`, `<include>`, custom `<ink-*>` components, and template variables
- **Validation** — catches missing alt text, Gmail clipping risks, Outlook issues
- **Data merging** — optional Jinja2-compatible data merge via MiniJinja (`--data data.json`)
- **Hybrid output** — `--hybrid` mode emits `<div>` layout with Outlook MSO ghost table fallbacks
- **Bulletproof buttons** — VML `<v:roundrect>` fallbacks for styled buttons in Outlook
- **Live preview** — `inky serve` starts a local dev server with auto-reload
- **Template friendly** — auto-preserves ERB, Jinja2, Handlebars, and other merge tag syntax
- **Plain text generation** — auto-generate `.txt` multipart email version (`--plain-text`)
- **Spam checker** — `inky spam-check` detects common spam triggers
- **22 validation rules** — links, accessibility, rendering quirks, Gmail clipping, spam detection
- **Per-template data** — `--data-dir` auto-pairs JSON data files with templates
- **Migration tool** — `inky migrate` converts v1 syntax to v2 automatically
- **Language bindings** — Node.js, PHP, Python, Ruby, Go

## Documentation

- **[Getting Started](docs/getting-started.md)** — Installation, CLI usage, first template
- **[Component Reference](docs/components.md)** — All 27 components with examples
- **[Style Reference](docs/styles.md)** — SCSS variables for colors, typography, layout, and more
- **[Data Merging](docs/data-merging.md)** — Merge JSON data into templates
- **[Hybrid Output](docs/hybrid-output.md)** — `<div>` layout with Outlook fallbacks
- **[Migration Guide](docs/migration.md)** — Upgrading from v1 to v2
- **[Language Bindings](docs/bindings.md)** — Node.js, PHP, Python, Ruby, Go

## Examples

| Language | Repository |
|----------|------------|
| Node.js | [inky-example-node](https://github.com/foundation/inky-example-node) |
| PHP | [inky-example-php](https://github.com/foundation/inky-example-php) |
| Python | [inky-example-python](https://github.com/foundation/inky-example-python) |
| Ruby | [inky-example-ruby](https://github.com/foundation/inky-example-ruby) |
| Go | [inky-example-go](https://github.com/foundation/inky-example-go) |

## Building from Source

```bash
git clone https://github.com/foundation/inky.git
cd inky

# Build the CLI
cargo build -p inky-cli --release
# Binary at: target/release/inky

# Run tests
cargo test --workspace
```

## Architecture

| Crate | Purpose | Output |
|-------|---------|--------|
| `inky-core` | Core transformation engine | Rust library |
| `inky-cli` | Command-line tool | `inky` binary |
| `inky-wasm` | Browser/Node.js bindings | `.wasm` module |
| `inky-ffi` | PHP/Python/Ruby/Go bindings | `.so` / `.dylib` / `.dll` |

## License

MIT

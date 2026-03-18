# Inky v2 Documentation

Inky is a complete email framework that converts simple HTML into complex, responsive email-ready HTML. Written in Rust, it ships as a CLI, WASM module, and native shared library.

## Docs

- **[Getting Started](getting-started.md)** -- Installation, basic usage, layouts, includes, custom components
- **[Component Reference](components.md)** -- Every built-in component with syntax, attributes, and examples
- **[Style Reference](styles.md)** -- All SCSS variables for customizing colors, typography, layout, and more
- **[Data Merging](data-merging.md)** -- Merge JSON data into templates with MiniJinja
- **[Migration Guide](migration.md)** -- Upgrading from Inky v1 to v2
- **[Language Bindings](bindings.md)** -- Node.js, PHP, Python, and Ruby integration

## Quick Links

| Resource | Link |
|----------|------|
| GitHub | [github.com/foundation/inky](https://github.com/foundation/inky) |
| npm | `inky-wasm` |
| Cargo | `inky-cli` |
| Packagist | `foundation/inky` |
| PyPI | `inky-email` |
| RubyGems | `inky-email` |

## Architecture

```
inky-core      Rust library -- the transformation engine
inky-cli       CLI binary (build, watch, validate, migrate, init)
inky-wasm      WASM module for Node.js/browser (powers the npm package)
inky-ffi       C FFI shared library (powers PHP, Python, Ruby bindings)
```

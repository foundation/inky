# Language Bindings

Inky provides official bindings for Node.js, PHP, Python, and Ruby. All bindings expose the same core API surface.

## Common API

Every binding provides these functions:

| Function | Description |
|----------|-------------|
| `transform(html, columns?)` | Transform Inky HTML into email-safe table markup |
| `transformInline(html)` | Transform and inline CSS from `<style>` blocks |
| `migrate(html)` | Migrate v1 syntax to v2 (returns HTML string) |
| `migrateWithDetails(html)` | Migrate v1 syntax, returns `{html, changes[]}` |
| `validate(html)` | Validate template, returns array of diagnostics |
| `version()` | Get the Inky engine version |

Diagnostics from `validate()` are objects/dicts with `severity` (`"warning"` or `"error"`), `rule`, and `message` fields.

---

## Node.js

**Package:** `inky-wasm` on npm
**Engine:** WASM (compiled from Rust via wasm-bindgen)
**Requires:** Node.js (any recent version)

### Install

```bash
npm install inky-wasm
```

### API

```js
const inky = require("inky-wasm");

// Transform
const html = inky.transform('<button href="#">Click</button>');
const html16 = inky.transform('<row><column>Wide</column></row>', { columns: 16 });

// Transform + inline CSS
const inlined = inky.transformInline(`
  <style>.button { background: blue; }</style>
  <button href="#">Click</button>
`);

// Migrate v1 to v2
const migrated = inky.migrate('<columns large="6">Content</columns>');

// Migrate with change details
const result = inky.migrateWithDetails('<columns large="6">Content</columns>');
// result.html    => '<column lg="6">Content</column>'
// result.changes => ['<columns> -> <column>', ...]

// Validate
const diagnostics = inky.validate('<button>No href</button>');
// [{ severity: "error", rule: "button-no-href", message: "..." }]

const diagnostics16 = inky.validate(html, { columns: 16 });

// Version
console.log(inky.version()); // "2.0.0"
```

### TypeScript

Type definitions are included. Key types:

```ts
interface TransformOptions { columns?: number; }
interface ValidateOptions { columns?: number; }
interface Diagnostic { severity: "warning" | "error"; rule: string; message: string; }
interface MigrateResult { html: string; changes: string[]; }
```

---

## PHP

**Package:** `foundation/inky` on Packagist
**Engine:** Native shared library via FFI (or PECL extension)
**Requires:** PHP >= 8.1

### Install

```bash
composer require foundation/inky
```

You also need the `libinky` shared library available. Build it from source:

```bash
cargo build -p inky-ffi --release
# produces target/release/libinky.dylib (macOS) or libinky.so (Linux)
```

### Driver Setup

The PHP package auto-detects the best available driver:

| Priority | Driver | Mechanism | Best For |
|----------|--------|-----------|----------|
| 1 | PECL Extension | `ext-inky` | Shared hosting, production |
| 2 | FFI | `ext-ffi` + `libinky` | Local dev, self-managed servers |

**FFI setup** -- enable in `php.ini`:

```ini
# Option A: Enable globally (dev)
ffi.enable = true

# Option B: Preload mode (production, more secure)
ffi.enable = preload
opcache.preload = /path/to/vendor/inky/preload.php
```

Note: `ffi.enable` is a `PHP_INI_SYSTEM` directive and cannot be changed with `ini_set()`.

### API

```php
use Inky\Inky;

// Transform
$html = Inky::transform('<button href="#">Click</button>');
$html = Inky::transform('<row><column>Content</column></row>', columns: 16);

// Transform + inline CSS
$html = Inky::transformInline('<style>...</style><button href="#">Click</button>');

// Migrate
$html = Inky::migrate('<columns large="6">Content</columns>');

// Migrate with details
$result = Inky::migrateWithDetails('<columns large="6">Content</columns>');
// $result['html']    => '<column lg="6">Content</column>'
// $result['changes'] => ['<columns> -> <column>', ...]

// Validate
$diagnostics = Inky::validate('<button>No href</button>');
// [['severity' => 'error', 'rule' => 'button-no-href', 'message' => '...']]

// Version
echo Inky::version(); // "2.0.0"
```

---

## Python

**Package:** `inky-email` on PyPI
**Engine:** Native shared library via ctypes
**Requires:** Python >= 3.8

### Install

```bash
pip install inky-email
```

You also need the `libinky` shared library. Build from source:

```bash
cargo build -p inky-ffi --release
```

The library searches these paths automatically:
1. `target/release/` (development)
2. Bundled with the package
3. `/usr/local/lib/`
4. `/usr/lib/`

### API

```python
import inky

# Transform
html = inky.transform('<button href="#">Click</button>')
html = inky.transform('<row><column>Content</column></row>', columns=16)

# Transform + inline CSS
html = inky.transform_inline('<style>...</style><button href="#">Click</button>')

# Migrate
html = inky.migrate('<columns large="6">Content</columns>')

# Migrate with details
result = inky.migrate_with_details('<columns large="6">Content</columns>')
# result['html']    => '<column lg="6">Content</column>'
# result['changes'] => ['<columns> -> <column>', ...]

# Validate
diagnostics = inky.validate('<button>No href</button>')
# [{'severity': 'error', 'rule': 'button-no-href', 'message': '...'}]

# Version
print(inky.version())  # "2.0.0"
```

Note: Python uses `snake_case` -- `transform_inline`, `migrate_with_details`.

---

## Ruby

**Package:** `inky-email` on RubyGems
**Engine:** Native shared library via Fiddle
**Requires:** Ruby >= 2.7

### Install

```bash
gem install inky-email
```

Or in your `Gemfile`:

```ruby
gem "inky-email"
```

You also need the `libinky` shared library. Build from source:

```bash
cargo build -p inky-ffi --release
```

The library searches these paths automatically:
1. `target/release/` (development)
2. Bundled with the gem
3. `/usr/local/lib/`
4. `/usr/lib/`

### API

```ruby
require "inky"

# Transform
html = Inky.transform('<button href="#">Click</button>')
html = Inky.transform('<row><column>Content</column></row>', columns: 16)

# Transform + inline CSS
html = Inky.transform_inline('<style>...</style><button href="#">Click</button>')

# Migrate
html = Inky.migrate('<columns large="6">Content</columns>')

# Migrate with details
result = Inky.migrate_with_details('<columns large="6">Content</columns>')
# result[:html]    => '<column lg="6">Content</column>'
# result[:changes] => ['<columns> -> <column>', ...]

# Validate
diagnostics = Inky.validate('<button>No href</button>')
# [{severity: "error", rule: "button-no-href", message: "..."}]

# Version
puts Inky.version  # "2.0.0"
```

---

## Building the Shared Library

PHP, Python, and Ruby bindings all depend on the `libinky` shared library from `inky-ffi`. To build it:

```bash
cd /path/to/inky
cargo build -p inky-ffi --release
```

This produces:
- **macOS:** `target/release/libinky.dylib`
- **Linux:** `target/release/libinky.so`
- **Windows:** `target/release/inky.dll`

For production, copy the library to a system path (`/usr/local/lib/`) or bundle it with your package.

## Building the WASM Module

The Node.js binding uses the WASM module from `inky-wasm`:

```bash
cd crates/inky-wasm
wasm-pack build --target nodejs
```

This generates the `.wasm` file and JS/TS wrapper in `pkg/`.

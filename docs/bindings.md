---
raw: true
title: "Language Bindings"
description: "Official Inky bindings for Node.js, PHP, Python, Ruby, and Go. Transform Inky markup to email HTML in any language."
nav_group: "Guides"
nav_order: 2
---

# Language Bindings

Inky provides official bindings for Node.js, PHP, Python, Ruby, and Go. All bindings expose the same core transform/validate/migrate API, and PHP, Python, Ruby, and Go additionally expose the full `build` pipeline in-process (see [Build Pipeline](#build-pipeline) below).

For complete working examples with build scripts and email sending, see:
[Node.js](https://github.com/foundation/inky-example-node) |
[PHP](https://github.com/foundation/inky-example-php) |
[Python](https://github.com/foundation/inky-example-python) |
[Ruby](https://github.com/foundation/inky-example-ruby) |
[Go](https://github.com/foundation/inky-example-go)

## Common API

Every binding provides these functions:

| Function | Description |
|----------|-------------|
| `transform(html, columns?)` | Transform Inky HTML into email-safe table markup |
| `transformInline(html)` | Transform and inline CSS from `<style>` blocks |
| `transformWithData(html, dataJson)` | Transform with JSON data merge, then inline CSS |
| `transformHybrid(html)` | Transform using hybrid output (div + MSO ghost tables) |
| `toPlainText(html)` | Convert HTML to plain text for multipart email |
| `migrate(html)` | Migrate v1 syntax to v2 (returns HTML string) |
| `migrateWithDetails(html)` | Migrate v1 syntax, returns `{html, changes[]}` |
| `validate(html)` | Validate template, returns array of diagnostics |
| `version()` | Get the Inky engine version |

Diagnostics from `validate()` are objects/dicts with `severity` (`"warning"` or `"error"`), `rule`, and `message` fields.

PHP, Python, Ruby, and Go also provide `build`, described below.

## Build Pipeline

`build` runs the same pipeline as `inky build`: layout resolution, custom `<ink-*>` components, includes, template data merge, framework SCSS compilation and injection, component transform, CSS inlining, and output cleanup. A library consumer calling `build` produces byte-identical output to the CLI.

Every language's `build` takes the same three inputs — `html`, an optional `basePath`, and an options object/map — and the option keys are identical across languages:

| Option | Type | Default | Same as CLI flag |
|--------|------|---------|-------------------|
| `inline_css` | bool | `true` | `--no-inline-css` (inverted) |
| `framework_css` | bool | `true` | `--no-framework-css` (inverted) |
| `components_dir` | string | `"components"` | n/a (config `components`) |
| `columns` | int | `12` | `--columns` |
| `hybrid` | bool | `false` | `--hybrid` |
| `bulletproof_buttons` | bool | `false` | `--bulletproof-buttons` |
| `plain_text` | bool | `false` | `--plain-text` |
| `data` | object/map | none | `--data` (as parsed JSON, not a file path) |

**`basePath` resolution:** `basePath` is a single directory. Every `src`/`href` the pipeline resolves — the layout, includes, custom components, and linked SCSS/CSS files — resolves relative to `basePath`, regardless of which file references it. A partial included from a layout, and a `<link>` inside that partial, both resolve against `basePath`, not against the layout's or partial's own directory. Pass `null`/`nil`/`None`/`""` to disable filesystem resolution entirely (stdin-style input with no layouts/includes/components).

On failure, `build` raises a language-specific typed exception carrying the non-fatal warnings collected before the failure (e.g. an unreadable linked SCSS file) — the same warnings a successful build would report. See each language's section for the exact exception type.

**Node.js/WASM does not expose `build`** — it has no filesystem access, so it can't resolve layouts, includes, or linked SCSS/CSS. Use the `inky` CLI for the full pipeline from a Node project (e.g. shell out to `inky build`), or the Node/WASM binding directly for `transform`/`transformInline`/`transformWithData`/`transformHybrid`/`toPlainText`/`migrate`/`validate`, which remain available in-process. A resolver-callback design for in-process pipeline support in Node is planned but not yet implemented.

---

## Node.js

**Package:** `inky` on npm
**Engine:** WASM (compiled from Rust via wasm-bindgen)
**Requires:** Node.js (any recent version)

No `build` — see [Build Pipeline](#build-pipeline) above for why, and shell out to the `inky` CLI when you need the full pipeline from Node.

### Install

```bash
npm install inky
```

### API

```js
const inky = require("inky");

// Transform
const html = inky.transform('<button href="#">Click</button>');
const html16 = inky.transform('<row><column>Wide</column></row>', { columns: 16 });

// Transform + inline CSS
const inlined = inky.transformInline(`
  <style>.button { background: blue; }</style>
  <button href="#">Click</button>
`);

// Transform with data merge
const merged = inky.transformWithData(
  '<button href="{{ url }}">{{ text }}</button>',
  JSON.stringify({ url: "https://example.com", text: "Click" })
);

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

The FFI driver's `stubs/inky.h` header is regenerated from `cbindgen` on every `cargo build`, so it can't drift from `inky-ffi`'s actual exports.

### API

```php
use Inky\Inky;

// Transform
$html = Inky::transform('<button href="#">Click</button>');
$html = Inky::transform('<row><column>Content</column></row>', columns: 16);

// Transform + inline CSS
$html = Inky::transformInline('<style>...</style><button href="#">Click</button>');

// Transform with data merge
$html = Inky::transformWithData(
    '<button href="{{ url }}">{{ text }}</button>',
    json_encode(['url' => 'https://example.com', 'text' => 'Click'])
);

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

### Build

```php
use Inky\Inky;
use Inky\BuildException;

try {
    $result = Inky::build(
        file_get_contents('src/emails/welcome.inky'),
        'src/emails',                       // basePath — resolves layouts, includes, components, linked SCSS/CSS
        ['data' => ['user' => ['name' => 'Alice']], 'hybrid' => false],
    );
    echo $result->html;
    echo $result->text;      // null unless 'plain_text' => true was passed
    print_r($result->warnings);
} catch (BuildException $e) {
    echo $e->getMessage();
    print_r($e->warnings);   // non-fatal notes collected before the failure
}
```

`Inky::build(string $html, ?string $basePath, array $options = []): BuildResult` throws `BuildException` (a `RuntimeException`) on pipeline failure. See [Build Pipeline](#build-pipeline) for the option keys and `basePath` resolution rule.

### Errors

General functions (`transform`, `migrate`, `validate`, …) throw a plain `RuntimeException` if the native call fails. `build` throws `BuildException` instead, which carries `->warnings`.

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

# Transform with data merge
html = inky.transform_with_data(
    '<button href="{{ url }}">{{ text }}</button>',
    '{"url": "https://example.com", "text": "Click"}'
)

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

### Build

```python
import inky
from inky import InkyBuildError

try:
    result = inky.build(
        open("src/emails/welcome.inky").read(),
        base_path="src/emails",   # resolves layouts, includes, components, linked SCSS/CSS
        data={"user": {"name": "Alice"}},
        hybrid=False,
    )
    print(result.html)
    print(result.text)      # None unless plain_text=True was passed
    print(result.warnings)
except InkyBuildError as e:
    print(str(e))
    print(e.warnings)       # non-fatal notes collected before the failure
```

`inky.build(html, base_path=None, **options)` returns a `BuildResult` dataclass (`html`, `text`, `warnings`) and raises `InkyBuildError` on pipeline failure. See [Build Pipeline](#build-pipeline) for the option keys and `base_path` resolution rule.

### Errors

General functions raise `InkyError` (a `RuntimeError` subclass) if the native call fails. `build` raises `InkyBuildError` (a subclass of `InkyError`) instead, which carries `.warnings`.

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

# Transform with data merge
html = Inky.transform_with_data(
    '<button href="{{ url }}">{{ text }}</button>',
    '{"url": "https://example.com", "text": "Click"}'
)

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

Passing a non-`String` (e.g. `nil`) to any function raises `TypeError` rather than segfaulting.

### Build

```ruby
require "inky"

begin
  result = Inky.build(
    File.read("src/emails/welcome.inky"),
    base_path: "src/emails",   # resolves layouts, includes, components, linked SCSS/CSS
    data: { user: { name: "Alice" } },
    hybrid: false,
  )
  puts result.html
  puts result.text          # nil unless plain_text: true was passed
  puts result.warnings
rescue Inky::BuildError => e
  puts e.message
  puts e.warnings           # non-fatal notes collected before the failure
end
```

`Inky.build(html, base_path: nil, **options)` returns a `BuildResult` struct (`html`, `text`, `warnings`) and raises `Inky::BuildError` on pipeline failure. See [Build Pipeline](#build-pipeline) for the option keys and `base_path` resolution rule.

### Errors

General functions raise `Inky::Error` (a `StandardError` subclass) if the native call fails. `build` raises `Inky::BuildError` instead, which carries `#warnings`.

---

## Go

**Package:** `github.com/foundation/inky-go`
**Engine:** Native shared library via cgo
**Requires:** Go with cgo enabled; a C toolchain

### Install

You need the `libinky` shared library installed where cgo's linker can find it:

```bash
cd /path/to/inky
cargo build -p inky-ffi --release

# macOS
cp target/release/libinky.dylib /usr/local/lib/

# Linux
sudo cp target/release/libinky.so /usr/local/lib/
sudo ldconfig
```

```bash
go get github.com/foundation/inky-go
```

### API

```go
import inky "github.com/foundation/inky-go"

// Transform
html := inky.Transform(`<button href="#">Click</button>`)
html16 := inky.TransformWithColumns(`<row><column>Wide</column></row>`, 16)

// Transform + inline CSS
inlined := inky.TransformInline(`<style>.button{background:blue}</style><button href="#">Click</button>`)

// Transform with data merge
merged := inky.TransformWithData(
    `<button href="{{ url }}">{{ text }}</button>`,
    `{"url": "https://example.com", "text": "Click"}`,
)

// Migrate
migrated := inky.Migrate(`<columns large="6">Content</columns>`)
result, err := inky.MigrateWithDetails(`<columns large="6">Content</columns>`)
// result.HTML, result.Changes

// Validate
diagnostics, err := inky.Validate(`<button>No href</button>`)
// []Diagnostic{{Severity: "error", Rule: "button-no-href", Message: "..."}}

// Version
fmt.Println(inky.Version()) // "2.0.0"
```

### Build

```go
inlineCSS := false
result, err := inky.Build(
    templateHTML,
    "src/emails", // basePath — resolves layouts, includes, components, linked SCSS/CSS
    &inky.BuildOptions{
        InlineCSS: &inlineCSS,
        Data:      map[string]any{"user": map[string]any{"name": "Alice"}},
    },
)
if err != nil {
    var buildErr *inky.BuildError
    if errors.As(err, &buildErr) {
        fmt.Println(buildErr.Message)
        fmt.Println(buildErr.Warnings) // non-fatal notes collected before the failure
    }
    return err
}
fmt.Println(result.HTML)
fmt.Println(result.Text) // "" unless PlainText: true was set
fmt.Println(result.Warnings)
```

`Build(html, basePath string, options *BuildOptions) (BuildResult, error)` returns a `*BuildError` on pipeline failure (satisfies the `error` interface; carries `.Warnings`). `BuildOptions.InlineCSS` and `.FrameworkCSS` are `*bool` — `nil` means "use the engine default (`true`)"; a pointer to `false` disables the feature. See [Build Pipeline](#build-pipeline) for the remaining option keys and the `basePath` resolution rule.

### Errors

`Transform`, `TransformInline`, `TransformWithData`, `TransformHybrid`, `ToPlainText`, `Migrate`, and `Version` return bare strings with no error (a native failure yields an empty string). `MigrateWithDetails` and `Validate` return `(T, error)` for JSON-decode failures. `Build` returns `(BuildResult, error)`, where the error is `*BuildError` on pipeline failure.

---

## Building the Shared Library

PHP, Python, Ruby, and Go bindings all depend on the `libinky` shared library from `inky-ffi`. To build it:

```bash
cd /path/to/inky
cargo build -p inky-ffi --release
```

This produces:
- **macOS:** `target/release/libinky.dylib`
- **Linux:** `target/release/libinky.so`
- **Windows:** `target/release/inky.dll`

For production, copy the library to a system path (`/usr/local/lib/`) or bundle it with your package. The Go binding's cgo `LDFLAGS` link against `/usr/local/lib` specifically (`-linky`), so `libinky` must live there (or on the linker's search path) for `go build`/`go test` to succeed — it can't be bundled inside the Go module.

## Building the WASM Module

The Node.js binding uses the WASM module from `inky-wasm`:

```bash
cd crates/inky-wasm
wasm-pack build --target nodejs
```

This generates the `.wasm` file and JS/TS wrapper in `pkg/`.

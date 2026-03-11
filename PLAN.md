# Inky Rust Rewrite Plan

## Overview

Rewrite the Inky HTML-to-email transpiler in Rust and distribute it as:

1. **WASM module** — for JS/Node.js/browser (drop-in replacement for current npm package)
2. **Native shared library** (.so/.dylib/.dll) — for PHP (via FFI), Python (via ctypes), Ruby (via fiddle), and any other language with C FFI support
3. **Rust crate** — for Rust consumers and as the canonical source of truth

This eliminates the need for language-specific reimplementations (inky-rb, lorenzo/pinky, etc.) and ensures all consumers produce identical output.

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
│   │       ├── parser.rs         # HTML parsing and component detection
│   │       ├── components.rs     # Component factory (all transformation rules)
│   │       ├── column.rs         # Column/grid sizing logic
│   │       ├── attrs.rs          # Attribute extraction and filtering
│   │       └── config.rs         # Configuration (column count, tag names)
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
│       └── inky.h                # Generated C header (via cbindgen)
│
├── bindings/
│   ├── node/                     # npm package wrapper
│   │   ├── package.json
│   │   └── index.js              # JS API wrapping WASM
│   │
│   ├── php/                      # Composer package wrapper
│   │   ├── composer.json
│   │   ├── src/
│   │   │   └── Inky.php          # PHP FFI wrapper class
│   │   ├── tests/
│   │   │   └── InkyTest.php      # PHPUnit tests against shared fixtures
│   │   └── lib/                  # Prebuilt .so/.dylib binaries
│   │
│   ├── python/                   # PyPI package wrapper
│   │   ├── pyproject.toml
│   │   ├── src/
│   │   │   └── inky/__init__.py  # ctypes wrapper
│   │   ├── tests/
│   │   │   └── test_inky.py      # pytest tests against shared fixtures
│   │   └── lib/                  # Prebuilt .so/.dylib binaries
│   │
│   └── ruby/                     # RubyGems package wrapper
│       ├── inky.gemspec
│       ├── lib/
│       │   └── inky.rb           # fiddle wrapper
│       ├── spec/
│       │   └── inky_spec.rb      # RSpec tests against shared fixtures
│       └── ext/                  # Prebuilt .so/.dylib binaries
│
├── tests/
│   ├── fixtures/                 # Shared test fixtures (input/output HTML pairs)
│   │   ├── components.json       # All component test cases
│   │   ├── grid.json             # All grid test cases
│   │   └── parser.json           # General parser test cases
│   └── integration/              # Cross-language integration tests
│
└── .github/
    └── workflows/
        ├── ci.yml                # Test on every push
        └── release.yml           # Build + publish all targets
```

---

## Phase 1: Rust Core (`inky-core`)

The core library is a pure Rust crate with no platform dependencies. It takes an HTML string and returns transformed HTML.

### Public API

```rust
pub struct Inky {
    config: Config,
}

pub struct Config {
    pub column_count: u32,          // default: 12
    pub components: ComponentNames, // customizable tag names
}

pub struct ComponentNames {
    pub button: String,      // default: "button"
    pub row: String,         // default: "row"
    pub columns: String,     // default: "columns"
    pub container: String,   // default: "container"
    pub callout: String,     // default: "callout"
    pub inky: String,        // default: "inky"
    pub block_grid: String,  // default: "block-grid"
    pub menu: String,        // default: "menu"
    pub menu_item: String,   // default: "item"
    pub center: String,      // default: "center"
    pub spacer: String,      // default: "spacer"
    pub wrapper: String,     // default: "wrapper"
    pub h_line: String,      // default: "h-line"
}

impl Inky {
    pub fn new() -> Self;
    pub fn with_config(config: Config) -> Self;
    pub fn transform(&self, html: &str) -> String;
}

// Convenience function
pub fn transform(html: &str) -> String;
```

### Dependencies

```toml
[dependencies]
scraper = "0.22"      # HTML parsing (built on html5ever + selectors)
ego-tree = "0.10"     # Tree traversal (used by scraper)
regex = "1"           # Raw tag extraction
```

The `scraper` crate provides CSS selector-based element querying similar to Cheerio/jQuery, making the port straightforward.

### Core Algorithm (mirrors current JS implementation)

```
1. Extract <raw> blocks → replace with ###RAW{i}### placeholders
2. Parse HTML string into DOM tree (scraper::Html)
3. Loop while custom component elements exist in the tree:
   a. Find first matching component element
   b. Transform it via component factory → HTML string
   c. Replace element in tree with transformed HTML
4. Remove data-parsed attributes from <center> tags
5. Serialize DOM back to HTML string
6. Re-inject raw block content into placeholders
7. Return final HTML string
```

### Component Transformation Rules

Each component maps directly from the current JS implementation:

| Component | Input Tag | Output Summary |
|-----------|-----------|----------------|
| `h-line` | `<h-line>` | `<table class="h-line [classes]"><tr><th>&nbsp;</th></tr></table>` |
| `columns` | `<columns>` | `<th class="small-N large-N columns [first] [last]"><table><tbody><tr><th>[content]</th>[expander]</tr></tbody></table></th>` |
| `row` | `<row>` | `<table class="row [classes]" [attrs]><tbody><tr>[content]</tr></tbody></table>` |
| `button` | `<button>` | Nested table with `<a>`, optional expand/center |
| `container` | `<container>` | `<table align="center" class="container [classes]"><tbody><tr><td>[content]</td></tr></tbody></table>` |
| `inky` | `<inky>` | Easter egg octopus image |
| `block-grid` | `<block-grid>` | `<table class="block-grid up-N [classes]"><tbody><tr>[content]</tr></tbody></table>` |
| `menu` | `<menu>` | Double-nested table structure |
| `item` | `<item>` | `<th class="menu-item [classes]"><a href="...">[content]</a></th>` |
| `center` | `<center>` | Modifies children in-place: adds `align="center"` + `class="float-center"` |
| `callout` | `<callout>` | `<table class="callout"><tbody><tr><th class="callout-inner [classes]">[content]</th><th class="expander"></th></tr></tbody></table>` |
| `spacer` | `<spacer>` | Table with height/font-size/line-height styling. Responsive: two tables with hide/show classes |
| `wrapper` | `<wrapper>` | `<table class="wrapper [classes]" align="center"><tbody><tr><td class="wrapper-inner">[content]</td></tr></tbody></table>` |

### Column Sizing Logic

```rust
fn make_column(element, column_count: u32) -> String {
    let col_count = sibling_column_count + 1;
    let small = attr("small").unwrap_or(column_count);
    let large = attr("large")
        .or(attr("small"))
        .unwrap_or(column_count / col_count);

    let mut classes = vec![
        format!("small-{small}"),
        format!("large-{large}"),
        "columns".to_string(),
    ];

    // Add "first" if no previous column sibling
    // Add "last" if no next column sibling

    // Add expander unless:
    //   - large != column_count, OR
    //   - element contains nested row, OR
    //   - no-expander attribute is set (and not "false")
}
```

### Attribute Filtering

Blacklisted attributes (stripped from output):
```
class, id, href, size, size-sm, size-lg, large, no-expander, small, target
```

All other attributes are passed through to the output HTML.

---

## Phase 2: WASM Bindings (`inky-wasm`)

### Build Target

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
inky-core = { path = "../inky-core" }
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

### Exported Functions

```rust
use wasm_bindgen::prelude::*;
use inky_core::{Inky, Config};

#[wasm_bindgen]
pub fn transform(html: &str) -> String {
    Inky::new().transform(html)
}

#[wasm_bindgen]
pub fn transform_with_config(html: &str, column_count: u32) -> String {
    let config = Config {
        column_count,
        ..Default::default()
    };
    Inky::with_config(config).transform(html)
}
```

### Build Command

```bash
wasm-pack build crates/inky-wasm --target bundler    # for npm/bundlers
wasm-pack build crates/inky-wasm --target web        # for browsers
wasm-pack build crates/inky-wasm --target nodejs      # for Node.js
```

### npm Package Wrapper (`bindings/node/`)

```json
{
  "name": "inky",
  "version": "2.0.0",
  "main": "index.js",
  "types": "index.d.ts"
}
```

```javascript
// bindings/node/index.js
const { transform, transform_with_config } = require('../crates/inky-wasm/pkg');

class Inky {
  constructor(options = {}) {
    this.columnCount = options.columnCount || 12;
  }

  releaseTheKraken(html) {
    return transform_with_config(html, this.columnCount);
  }
}

// Backwards-compatible API
module.exports = function(opts, cb) { /* stream wrapper */ };
module.exports.Inky = Inky;
```

---

## Phase 3: C FFI Bindings (`inky-ffi`)

### Build Target

```toml
[lib]
crate-type = ["cdylib", "staticlib"]
name = "inky"

[dependencies]
inky-core = { path = "../inky-core" }

[build-dependencies]
cbindgen = "0.27"    # Auto-generates C header file
```

### Exported Functions

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use inky_core::Inky;

/// Transform Inky HTML to email-safe HTML.
/// Caller must free the returned string with inky_free().
#[no_mangle]
pub extern "C" fn inky_transform(input: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let html = c_str.to_str().unwrap_or("");
    let result = Inky::new().transform(html);
    CString::new(result).unwrap_or_default().into_raw()
}

/// Transform with custom column count.
/// Caller must free the returned string with inky_free().
#[no_mangle]
pub extern "C" fn inky_transform_with_columns(
    input: *const c_char,
    column_count: u32,
) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let html = c_str.to_str().unwrap_or("");
    let config = inky_core::Config {
        column_count,
        ..Default::default()
    };
    let result = Inky::with_config(config).transform(html);
    CString::new(result).unwrap_or_default().into_raw()
}

/// Free a string returned by inky_transform.
#[no_mangle]
pub extern "C" fn inky_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)); }
    }
}
```

### Generated C Header (`inky.h`)

```c
#ifndef INKY_H
#define INKY_H

#include <stdint.h>

char* inky_transform(const char* input);
char* inky_transform_with_columns(const char* input, uint32_t column_count);
void inky_free(char* ptr);

#endif
```

### Build Commands

```bash
# Linux
cargo build --release -p inky-ffi --target x86_64-unknown-linux-gnu
# Output: target/x86_64-unknown-linux-gnu/release/libinky.so

# macOS (Intel)
cargo build --release -p inky-ffi --target x86_64-apple-darwin
# Output: target/x86_64-apple-darwin/release/libinky.dylib

# macOS (Apple Silicon)
cargo build --release -p inky-ffi --target aarch64-apple-darwin
# Output: target/aarch64-apple-darwin/release/libinky.dylib

# Windows
cargo build --release -p inky-ffi --target x86_64-pc-windows-msvc
# Output: target/x86_64-pc-windows-msvc/release/inky.dll
```

---

## Phase 4: PHP Bindings (`bindings/php/`)

### Composer Package

```json
{
  "name": "foundation/inky",
  "description": "Inky HTML-to-email transpiler for PHP via FFI",
  "type": "library",
  "require": {
    "php": ">=7.4",
    "ext-ffi": "*"
  },
  "autoload": {
    "psr-4": {
      "Foundation\\Inky\\": "src/"
    }
  }
}
```

### PHP Wrapper Class

```php
<?php
// bindings/php/src/Inky.php

namespace Foundation\Inky;

use FFI;

class Inky
{
    private static ?FFI $ffi = null;
    private int $columnCount;

    public function __construct(int $columnCount = 12)
    {
        $this->columnCount = $columnCount;

        if (self::$ffi === null) {
            self::$ffi = FFI::cdef(
                "char* inky_transform(const char* input);
                 char* inky_transform_with_columns(const char* input, uint32_t column_count);
                 void inky_free(char* ptr);",
                self::findLibrary()
            );
        }
    }

    public function transform(string $html): string
    {
        $ptr = self::$ffi->inky_transform_with_columns($html, $this->columnCount);
        $result = FFI::string($ptr);
        self::$ffi->inky_free($ptr);
        return $result;
    }

    public static function convert(string $html): string
    {
        return (new self())->transform($html);
    }

    private static function findLibrary(): string
    {
        $libDir = __DIR__ . '/../lib/';

        if (PHP_OS_FAMILY === 'Darwin') {
            return $libDir . 'libinky.dylib';
        } elseif (PHP_OS_FAMILY === 'Windows') {
            return $libDir . 'inky.dll';
        }

        return $libDir . 'libinky.so';
    }
}
```

### PHP Usage

```php
use Foundation\Inky\Inky;

// Simple one-liner
$html = Inky::convert('<row><columns>Hello</columns></row>');

// With custom column count
$inky = new Inky(columnCount: 16);
$html = $inky->transform('<row><columns large="8">Content</columns></row>');
```

### Requirements

- PHP 7.4+ (FFI extension is bundled with PHP, just needs `ffi.enable=true` in php.ini)
- No additional PHP extensions or PECL installs required
- Prebuilt binaries for linux-x64, darwin-x64, darwin-arm64, windows-x64 are shipped in the Composer package under `lib/`

---

## Phase 5: Python Bindings (`bindings/python/`)

### PyPI Package

```toml
# pyproject.toml
[project]
name = "inky-email"
version = "2.0.0"
description = "Inky HTML-to-email transpiler"
requires-python = ">=3.8"

[build-system]
requires = ["setuptools>=61.0"]
build-backend = "setuptools.backends._legacy:_Backend"
```

### Python Wrapper

```python
# bindings/python/src/inky/__init__.py

import ctypes
import platform
import os
from pathlib import Path

_lib = None

def _load_library():
    global _lib
    if _lib is not None:
        return _lib

    lib_dir = Path(__file__).parent / "lib"
    system = platform.system()

    if system == "Darwin":
        path = lib_dir / "libinky.dylib"
    elif system == "Windows":
        path = lib_dir / "inky.dll"
    else:
        path = lib_dir / "libinky.so"

    _lib = ctypes.CDLL(str(path))
    _lib.inky_transform.argtypes = [ctypes.c_char_p]
    _lib.inky_transform.restype = ctypes.c_void_p
    _lib.inky_transform_with_columns.argtypes = [ctypes.c_char_p, ctypes.c_uint32]
    _lib.inky_transform_with_columns.restype = ctypes.c_void_p
    _lib.inky_free.argtypes = [ctypes.c_void_p]
    _lib.inky_free.restype = None
    return _lib


def transform(html: str, column_count: int = 12) -> str:
    """Transform Inky HTML to email-safe HTML."""
    lib = _load_library()
    ptr = lib.inky_transform_with_columns(html.encode("utf-8"), column_count)
    result = ctypes.cast(ptr, ctypes.c_char_p).value.decode("utf-8")
    lib.inky_free(ptr)
    return result


class Inky:
    """Inky transpiler instance with configurable column count."""

    def __init__(self, column_count: int = 12):
        self.column_count = column_count

    def transform(self, html: str) -> str:
        return transform(html, self.column_count)
```

### Python Usage

```python
from inky import transform, Inky

# Simple one-liner
html = transform('<row><columns>Hello</columns></row>')

# With custom column count
inky = Inky(column_count=16)
html = inky.transform('<row><columns large="8">Content</columns></row>')
```

### Requirements

- Python 3.8+ (ctypes is in the standard library — no pip dependencies)
- Prebuilt binaries shipped in the package under `lib/`

---

## Phase 6: Ruby Bindings (`bindings/ruby/`)

### Gemspec

```ruby
# inky.gemspec
Gem::Specification.new do |s|
  s.name        = "inky-email"
  s.version     = "2.0.0"
  s.summary     = "Inky HTML-to-email transpiler"
  s.description = "Convert simple HTML into responsive email-ready HTML using Foundation for Emails"
  s.authors     = ["Foundation"]
  s.license     = "MIT"
  s.files       = Dir["lib/**/*", "ext/**/*"]
  s.require_paths = ["lib"]
  s.required_ruby_version = ">= 2.7"
end
```

### Ruby Wrapper

```ruby
# bindings/ruby/lib/inky.rb

require "fiddle"
require "fiddle/import"

module Inky
  module Native
    extend Fiddle::Importer

    lib_dir = File.expand_path("../../ext", __FILE__)
    case RUBY_PLATFORM
    when /darwin/
      dlload File.join(lib_dir, "libinky.dylib")
    when /mingw|mswin/
      dlload File.join(lib_dir, "inky.dll")
    else
      dlload File.join(lib_dir, "libinky.so")
    end

    extern "char* inky_transform(const char*)"
    extern "char* inky_transform_with_columns(const char*, unsigned int)"
    extern "void inky_free(char*)"
  end

  def self.transform(html, column_count: 12)
    ptr = Native.inky_transform_with_columns(html, column_count)
    result = ptr.to_s
    Native.inky_free(ptr)
    result
  end

  class Transpiler
    def initialize(column_count: 12)
      @column_count = column_count
    end

    def transform(html)
      Inky.transform(html, column_count: @column_count)
    end

    # Backwards compatibility with inky-rb
    alias_method :release_the_kraken, :transform
  end
end
```

### Ruby Usage

```ruby
require "inky"

# Simple one-liner
html = Inky.transform('<row><columns>Hello</columns></row>')

# With custom column count
inky = Inky::Transpiler.new(column_count: 16)
html = inky.transform('<row><columns large="8">Content</columns></row>')
```

### Requirements

- Ruby 2.7+ (fiddle is in the standard library — no gem dependencies)
- Prebuilt binaries shipped in the gem under `ext/`

---

## Phase 7: Go Bindings (separate repo: `foundation/inky-go`)

Go modules are imported by repo path, so this needs its own repository.

### Module

```go
// go.mod
module github.com/foundation/inky-go

go 1.21
```

### Go Wrapper

```go
// inky.go
package inky

/*
#cgo darwin,amd64  LDFLAGS: -L${SRCDIR}/lib/darwin_amd64 -linky
#cgo darwin,arm64  LDFLAGS: -L${SRCDIR}/lib/darwin_arm64 -linky
#cgo linux,amd64   LDFLAGS: -L${SRCDIR}/lib/linux_amd64 -linky
#cgo windows,amd64 LDFLAGS: -L${SRCDIR}/lib/windows_amd64 -linky

#include "inky.h"
#include <stdlib.h>
*/
import "C"
import "unsafe"

// Transform converts Inky HTML to email-safe HTML.
func Transform(html string) string {
	return TransformWithColumns(html, 12)
}

// TransformWithColumns converts Inky HTML with a custom column count.
func TransformWithColumns(html string, columnCount uint32) string {
	cInput := C.CString(html)
	defer C.free(unsafe.Pointer(cInput))

	cResult := C.inky_transform_with_columns(cInput, C.uint(columnCount))
	defer C.inky_free(cResult)

	return C.GoString(cResult)
}
```

### Go Usage

```go
package main

import (
    "fmt"
    "github.com/foundation/inky-go"
)

func main() {
    html := inky.Transform("<row><columns>Hello</columns></row>")
    fmt.Println(html)
}
```

### Requirements

- Go 1.21+
- cgo enabled (default on most platforms)
- Prebuilt binaries shipped in the module under `lib/`

---

## Phase 8: Shared Test Fixtures

Port all 52 existing test cases into language-agnostic JSON fixtures. Every language binding runs these same fixtures to guarantee identical output.

### Fixture Format

```json
// tests/fixtures/components.json
{
  "tests": [
    {
      "name": "creates a simple button",
      "input": "<button href=\"http://zurb.com\">Button</button>",
      "expected": "<table class=\"button\"><tbody><tr><td><table><tbody><tr><td><a href=\"http://zurb.com\">Button</a></td></tr></tbody></table></td></tr></tbody></table>"
    },
    {
      "name": "creates a spacer with default size",
      "input": "<spacer></spacer>",
      "expected": "<table class=\"spacer\"><tbody><tr><td height=\"16\" style=\"font-size:16px;line-height:16px;\">&nbsp;</td></tr></tbody></table>"
    }
  ]
}
```

### Test Runners

Each binding loads the same JSON fixtures and asserts identical output:

- **Rust:** `cargo test` — loads fixtures, runs through `inky_core::transform()`
- **Node.js:** `npm test` — loads fixtures, runs through WASM binding
- **PHP:** `phpunit` — loads fixtures, runs through FFI binding
- **Python:** `pytest` — loads fixtures, runs through ctypes binding
- **Ruby:** `rspec` — loads fixtures, runs through fiddle binding
- **Go:** `go test` — loads fixtures, runs through cgo binding

---

## Phase 9: CI/CD and Release Pipeline

### GitHub Actions Workflow

```yaml
# .github/workflows/release.yml
# Triggered on version tags (v2.0.0, etc.)

jobs:
  test:
    # Run cargo test on all platforms
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

  build-wasm:
    # wasm-pack build → upload artifact
    # Publish to npm

  build-native:
    # Cross-compile shared libraries for all targets:
    #   - x86_64-unknown-linux-gnu    → libinky.so
    #   - aarch64-unknown-linux-gnu   → libinky.so
    #   - x86_64-apple-darwin         → libinky.dylib
    #   - aarch64-apple-darwin        → libinky.dylib
    #   - x86_64-pc-windows-msvc      → inky.dll
    # Package into GitHub release
    # Publish to crates.io

  test-bindings:
    # For each language binding:
    #   1. Copy prebuilt native library into binding's lib/ directory
    #   2. Run language-specific test suite against shared JSON fixtures
    #   3. Verify output matches Rust core exactly

  publish:
    # After all tests pass:
    #   - npm: publish WASM-based Node.js package
    #   - Packagist: publish PHP Composer package with native binaries
    #   - PyPI: publish Python package with native binaries
    #   - RubyGems: publish gem with native binaries
    #   - crates.io: publish Rust crate
    #   - Go: tag release in foundation/inky-go (binaries committed to repo)
```

### Versioning

- All crates, npm package, and Composer package share the same version number
- Start at **v2.0.0** to signal the rewrite (current JS version is 1.4.2)

---

## Migration Path for Existing Users

### npm (JS/Node.js)

The v2.0.0 npm package maintains backward compatibility:

```javascript
// This still works exactly as before
const { Inky } = require('inky');
const inky = new Inky();
const html = inky.releaseTheKraken(input);

// New simpler API also available
const { transform } = require('inky');
const html = transform(input);
```

Breaking changes:
- Gulp stream integration removed (Gulp usage has declined significantly)
- Cheerio options no longer accepted (Rust uses its own HTML parser)
- Minimum Node.js version: 16+ (for WASM support)

### PHP

For users of `lorenzo/pinky` or `twigphp/inky-extra`:

```php
// Before (pinky)
$html = Pinky\transformString($body)->saveHTML();

// After (foundation/inky)
$html = \Foundation\Inky\Inky::convert($body);
```

### Python

```python
# Before (no official package existed)
# After
from inky import transform
html = transform('<row><columns>Hello</columns></row>')
```

### Ruby

```ruby
# Before (inky-rb)
# require 'inky'
# Inky::Core.new.release_the_kraken(html)

# After
require 'inky'
html = Inky.transform('<row><columns>Hello</columns></row>')
```

### Go

```go
// New — no previous Go support existed
import "github.com/foundation/inky-go"
html := inky.Transform("<row><columns>Hello</columns></row>")
```

### Other Languages

Any language with C FFI support can use the shared library directly without an official binding — the C API is just 3 functions (`inky_transform`, `inky_transform_with_columns`, `inky_free`).

---

## Implementation Order

### Stage 1: Core (do this first — everything else depends on it)

| Step | Task | Scope |
|------|------|-------|
| 1 | Set up Cargo workspace with three crates (`inky-core`, `inky-wasm`, `inky-ffi`) | Scaffolding |
| 2 | Implement `inky-core` with `scraper` crate | ~500-600 lines of Rust |
| 3 | Port all 52 test cases to JSON fixtures | Test data extraction |
| 4 | Write Rust tests against fixtures, achieve parity with JS | Testing |

### Stage 2: Distribution targets (can be done in parallel)

| Step | Task | Scope |
|------|------|-------|
| 5 | Build `inky-wasm` with wasm-bindgen | ~30 lines |
| 6 | Build `inky-ffi` with cbindgen | ~40 lines |

### Stage 3: Language bindings (can all be done in parallel)

| Step | Task | Scope |
|------|------|-------|
| 7 | Create Node.js wrapper package (WASM) | ~50 lines |
| 8 | Create PHP Composer package (FFI) | ~60 lines |
| 9 | Create Python PyPI package (ctypes) | ~40 lines |
| 10 | Create Ruby gem (fiddle) | ~40 lines |
| 11 | Create Go module in separate repo (cgo) | ~50 lines |

### Stage 4: Ship it

| Step | Task | Scope |
|------|------|-------|
| 12 | Set up CI: test Rust + all bindings on all platforms | CI config |
| 13 | Set up release pipeline: cross-compile + publish to all registries | CD config |
| 14 | Write migration guide for each language | Documentation |
| 15 | Publish v2.0.0 to npm, crates.io, Packagist, PyPI, RubyGems | Release |
| 16 | Archive `foundation/inky-rb` with pointer to new gem | Cleanup |
| 17 | Re-enable Dependabot security updates (disabled during 1.x→2.x transition) | Cleanup |

---
raw: true
title: "Hybrid Output Mode"
description: "Generate div-based layouts for modern email clients with automatic Outlook table fallbacks via MSO conditional comments."
nav_group: "Features"
nav_order: 4
---

# Hybrid Output Mode

Hybrid mode generates `<div>`-based layouts for modern email clients with Outlook-specific `<table>` fallbacks wrapped in MSO conditional comments. This produces cleaner HTML, better accessibility, and smaller file sizes compared to pure table-based output.

**Hybrid mode is off by default.** The default output uses pure table-based markup for maximum compatibility.

## Quick Start

```bash
# CLI flag
inky build src/ -o dist/ --hybrid

# Or in inky.config.json
{"src": "src/emails", "dist": "dist", "hybrid": true}
```

## How It Works

In table mode (default), a container outputs:

```html
<table role="presentation" align="center" class="container">
  <tbody><tr><td>...</td></tr></tbody>
</table>
```

In hybrid mode, the same container outputs:

```html
<!--[if mso]><table role="presentation" width="580" align="center"><tr><td><![endif]-->
<div class="container" style="max-width:580px;margin:0 auto;">...</div>
<!--[if mso]></td></tr></table><![endif]-->
```

Modern email clients (Apple Mail, Gmail, Yahoo, etc.) use the `<div>` with CSS. Microsoft Outlook (which uses the Word rendering engine) sees the `<table>` inside the `<!--[if mso]>` conditional comments.

## Supported Components

These layout components have hybrid variants:

| Component | Table Output | Hybrid Output |
|-----------|-------------|---------------|
| `<container>` | `<table>` | `<div>` with `max-width:580px` + MSO table |
| `<row>` | `<table>` with `<tr>` | `<div>` with `font-size:0` + MSO table row |
| `<column>` | `<th>` with inner table | `<div>` with `display:inline-block` + MSO `<td>` |
| `<wrapper>` | `<table>` | `<div>` + MSO table |
| `<block-grid>` | `<table>` with `<tr>` | `<div>` + MSO table |

Other components (button, callout, spacer, etc.) use their standard table-based output in both modes.

## Column Widths

In hybrid mode, columns use percentage-based widths calculated from the grid:

```html
<row>
  <column lg="6">Left</column>
  <column lg="6">Right</column>
</row>
```

Produces (simplified):

```html
<!--[if mso]><table width="100%"><tr><![endif]-->
<div class="row" style="font-size:0;">
  <!--[if mso]><td width="50%" valign="top"><![endif]-->
  <div class="columns" style="display:inline-block;width:100%;max-width:50%;vertical-align:top;">
    Left
  </div>
  <!--[if mso]></td><![endif]-->
  <!--[if mso]><td width="50%" valign="top"><![endif]-->
  <div class="columns" style="display:inline-block;width:100%;max-width:50%;vertical-align:top;">
    Right
  </div>
  <!--[if mso]></td><![endif]-->
</div>
<!--[if mso]></tr></table><![endif]-->
```

## CLI Usage

```bash
# Build with hybrid output
inky build email.inky --hybrid

# Watch with hybrid output
inky watch src/ -o dist/ --hybrid

# Serve with hybrid output
inky serve src/ --hybrid
```

## Configuration

Add `hybrid` to `inky.config.json`:

```json
{
  "src": "src/emails",
  "dist": "dist",
  "hybrid": true
}
```

The `--hybrid` CLI flag overrides the config file.

## Language Bindings

### Node.js (WASM)

```js
const inky = require("inky");
const html = inky.transformHybrid('<container><row><column>Hello</column></row></container>');
```

### FFI (PHP, Python, Ruby)

```python
import inky
html = inky.transform_hybrid('<container><row><column>Hello</column></row></container>')
```

## When to Use Hybrid Mode

**Use hybrid mode when:**
- You want cleaner, more semantic HTML
- Accessibility is a priority (screen readers handle `<div>` better than nested tables)
- You need smaller file sizes (important for Gmail's 102KB clipping limit)
- Your audience primarily uses modern email clients

**Stick with table mode when:**
- You need maximum compatibility with older/niche email clients
- You're targeting environments where Outlook is the primary client (tables render more predictably in Outlook)
- You're migrating from v1 and want identical output behavior

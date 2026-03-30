---
raw: true
title: "Migration Guide: v1 to v2"
nav_group: "Guides"
nav_order: 3
---

# Migration Guide: v1 to v2

## Why Migrate

Inky v2 is a ground-up rewrite in Rust with significant improvements:

- **10-100x faster** -- Rust core compiles to native binary, WASM, or shared library
- **Cleaner syntax** -- attributes over classes, singular `<column>`, `sm`/`lg` shorthand
- **New components** -- video, hero, social, card, alert, badge, accordion, blockquote, preview
- **CLI toolchain** -- `inky build`, `inky watch`, `inky validate`, `inky init`
- **CSS inlining** -- built-in, enabled by default
- **Framework CSS** -- SCSS framework with variable overrides, injected automatically
- **Layouts and includes** -- `<layout>`, `<include>`, template variables
- **Validation** -- catches missing alt text, Gmail clipping, Outlook issues
- **Template-friendly** -- auto-detects Handlebars, ERB, Jinja2, Mailchimp, etc.
- **Cross-platform** -- one engine powers CLI, Node.js, PHP, Python, and Ruby

## Running the Migrator

### Preview changes (stdout)

```bash
inky migrate email.inky
```

### Migrate a directory

```bash
inky migrate src/ -o migrated/
```

### Rewrite files in-place

```bash
inky migrate src/ --in-place
```

### Programmatic migration (any language)

```js
const inky = require("inky");
const result = inky.migrateWithDetails(v1Html);
console.log(result.html);      // migrated HTML
console.log(result.changes);   // ["<columns> -> <column>", ...]
```

## Complete Migration Table

| v1 Syntax | v2 Syntax | Notes |
|-----------|-----------|-------|
| `<columns>` | `<column>` | Plural to singular |
| `</columns>` | `</column>` | Closing tag too |
| `<h-line>` | `<divider>` | Renamed |
| `large="6"` on `<column>` | `lg="6"` | Shorthand |
| `small="12"` on `<column>` | `sm="12"` | Shorthand |
| `<spacer size="16">` | `<spacer height="16">` | Clearer attribute name |
| `<spacer size-sm="10">` | `<spacer sm="10">` | Shorthand |
| `<spacer size-lg="20">` | `<spacer lg="20">` | Shorthand |
| `<button class="small">` | `<button size="small">` | Class to attribute |
| `<button class="alert">` | `<button color="alert">` | Class to attribute |
| `<button class="expand">` | `<button expand>` | Class to bare attribute |
| `<button class="radius">` | `<button radius>` | Class to bare attribute |
| `<button class="rounded">` | `<button rounded>` | Class to bare attribute |
| `<button class="hollow">` | `<button hollow>` | Class to bare attribute |
| `<callout class="primary">` | `<callout color="primary">` | Class to attribute |
| `<menu class="vertical">` | `<menu direction="vertical">` | Class to attribute |
| `<center><menu>...</menu></center>` | `<menu align="center">...</menu>` | Wrapping to attribute |

### Button size values

`tiny`, `small`, `large`

### Button/callout color values

`primary`, `secondary`, `success`, `alert`, `warning`

### Button boolean attributes

`expand`, `expanded`, `radius`, `rounded`, `hollow`

## Before/After Examples

### Basic layout

**v1:**

```html
<container>
  <row>
    <columns large="6" small="12">Left</columns>
    <columns large="6" small="12">Right</columns>
  </row>
</container>
```

**v2:**

```html
<container>
  <row>
    <column lg="6" sm="12">Left</column>
    <column lg="6" sm="12">Right</column>
  </row>
</container>
```

### Button with styles

**v1:**

```html
<button class="small alert expand" href="#">Click Me</button>
```

**v2:**

```html
<button size="small" color="alert" expand href="#">Click Me</button>
```

### Mixed classes (custom classes preserved)

**v1:**

```html
<button class="small alert custom-btn" href="#">Click</button>
```

**v2:**

```html
<button class="custom-btn" size="small" color="alert" href="#">Click</button>
```

Custom CSS classes that aren't migration targets are preserved in the `class` attribute.

### Full template

**v1:**

```html
<container>
  <row>
    <columns large="6" small="12">
      <button class="small alert" href="#">Click</button>
      <spacer size="16"></spacer>
      <h-line></h-line>
      <callout class="primary">Important</callout>
    </columns>
    <columns large="6" small="12">
      <center>
        <menu class="vertical">
          <item href="#">Link</item>
        </menu>
      </center>
    </columns>
  </row>
</container>
```

**v2:**

```html
<container>
  <row>
    <column lg="6" sm="12">
      <button size="small" color="alert" href="#">Click</button>
      <spacer height="16"></spacer>
      <divider></divider>
      <callout color="primary">Important</callout>
    </column>
    <column lg="6" sm="12">
      <menu align="center" direction="vertical">
        <item href="#">Link</item>
      </menu>
    </column>
  </row>
</container>
```

## Breaking Changes

1. **v2 parser is strict** -- it does not accept v1 syntax. Run `inky migrate` first. If v1 tags are encountered, the parser outputs an error pointing to `inky migrate`.

2. **`.inky` file extension** -- Source templates should use `.inky`. The CLI auto-generates `.html` output files. Both `.inky` and `.html` are accepted as input.

3. **CSS inlining is on by default** -- v1 had no built-in inlining. If you handle inlining separately, pass `--no-inline-css`.

4. **Framework CSS is injected by default** -- The built-in SCSS framework is compiled and injected into each email. Disable with `--no-framework-css` if you use your own CSS.

5. **`role="presentation"` on all layout tables** -- v2 adds accessibility attributes to all generated tables. This is a non-breaking output change but may affect CSS selectors or snapshot tests.

## Tips

- Run `inky validate` after migration to catch any remaining issues.
- The migrator is safe to run multiple times -- already-migrated syntax is left unchanged.
- Use `inky migrate --in-place` with version control so you can review the diff.
- The `migrateWithDetails()` API returns a list of changes made, useful for logging or reports.

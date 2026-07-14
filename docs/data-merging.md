---
raw: true
title: "Data Merging"
description: "Merge JSON data into Inky templates using MiniJinja. Preview emails with real data or generate final output with variables."
nav_group: "Features"
nav_order: 3
---

# Data Merging

Inky can merge JSON data into your templates using [MiniJinja](https://github.com/mitsuhiko/minijinja), a Jinja2-compatible template engine. This is useful for previewing emails with real data or generating final output with variables filled in.

**By default, data merging is off.** Without `--data`, Inky preserves all template syntax (`{{ }}`, `{% %}`, etc.) untouched so you can use your own templating environment.

## Quick Example

**Template** (`src/emails/welcome.inky`):

```html
<layout src="../layouts/default.html" title="Welcome">
<container>
  <row>
    <column sm="12" lg="12">
      <h1>Hello {{ user.name }}!</h1>
      {% if show_coupon %}
        <callout>Use code <strong>{{ coupon_code }}</strong> for 20% off.</callout>
      {% endif %}
      <button href="{{ cta_url }}">Get Started</button>
    </column>
  </row>
</container>
```

**Data** (`data.json`):

```json
{
  "user": { "name": "Alice" },
  "show_coupon": true,
  "coupon_code": "WELCOME20",
  "cta_url": "https://example.com/start"
}
```

**Build:**

```bash
inky build src/emails/welcome.inky --data data.json -o dist/welcome.html
```

The output HTML will have all `{{ }}` and `{% %}` tags evaluated with the provided data.

## CLI Usage

### `inky build`

```bash
# Single file with data
inky build email.inky --data data.json

# Directory with data (same data.json merged into every template)
inky build src/ -o dist/ --data data.json

# Directory with per-template data files
inky build src/ -o dist/ --data data/

# Pipe from stdin
echo '<p>Hello {{ name }}</p>' | inky build --data data.json
```

`--data` accepts either a file or a directory. A **file** merges the same data into every template. A **directory** pairs each template with its own JSON file by name -- `data/welcome.json` merges into `src/welcome.inky`, `data/receipt.json` into `src/receipt.inky`, and so on. Templates with no matching file build without data merge.

### `inky watch` and `inky serve`

```bash
inky watch src/emails -o dist --data data.json
inky watch src/emails -o dist --data data/    # per-template data directory
inky serve src/emails --data data/
```

`watch` and `serve` accept `--data` the same way `build` does, including a directory of per-template files. In watch mode, Inky also watches the data file(s); when you edit `data.json` (or a file inside the data directory), the affected templates rebuild automatically. Both commands exit with an error at startup if `--data` points to a missing or unparseable path.

## Configuration

Add `data` to your `inky.config.json` instead of using the `--data` flag. It accepts a file or a directory, same as the CLI flag:

```json
{
  "src": "src/emails",
  "dist": "dist",
  "data": "data.json"
}
```

The CLI `--data` flag overrides the config file value.

## Syntax

Data merging uses Jinja2 syntax, which is nearly identical to Liquid/Shopify/Nunjucks for common operations.

### Variables

```html
{{ user.name }}
{{ order.total }}
{{ company }}
```

### Conditionals

```html
{% if unsubscribe_url %}
  <a href="{{ unsubscribe_url }}">Unsubscribe</a>
{% endif %}

{% if tier == "premium" %}
  <p>Thank you for being a premium member!</p>
{% elif tier == "trial" %}
  <p>Your trial ends soon.</p>
{% else %}
  <p>Upgrade today.</p>
{% endif %}
```

### Loops

```html
{% for item in cart %}
<row>
  <column lg="8">{{ item.name }}</column>
  <column lg="4">{{ item.price }}</column>
</row>
{% endfor %}
```

Looping over `<row>`/`<column>` like this is safe without `<raw>` -- they're plain custom tags at this point in the pipeline, not HTML table markup yet, so there's nothing for the HTML5 parser to relocate.

That's not true if the loop produces literal `<tr>` rows inside a real `<table>`/`<tbody>` -- for example, an order summary you write directly in table markup instead of with `<row>`/`<column>`:

```html
<table>
  <tbody>
    <raw>{% for item in cart %}
    <tr><td>{{ item.name }}</td><td>{{ item.price }}</td></tr>
    {% endfor %}</raw>
  </tbody>
</table>
```

Whether `<raw>` is load-bearing here depends on when the merge happens:

- If you merge data in the same call that builds the template (`inky build --data`, or a binding's `build(..., data: ...)`), the loop expands into real `<tr>` rows *before* Inky's component transform parses the document -- `<raw>` is optional defense-in-depth.
- If you build **without** data first -- e.g. producing a cached template shell for another template engine to render later -- the `{% for %}` text is still literal when Inky's transform parses the document. Bare template text directly inside `<table>`/`<tbody>` is not valid content there, so the HTML5 parser relocates it out of the table, corrupting the shell. `<raw>` is required to keep it in place until the later render step expands it.

### Filters

```html
{{ name | upper }}
{{ name | lower }}
{{ description | truncate(100) }}
{{ price | round(2) }}
{{ tags | join(", ") }}
```

See the [MiniJinja documentation](https://docs.rs/minijinja/latest/minijinja/syntax/index.html) for the full syntax reference.

## How It Works

Data merging runs **after** layout, include, and custom component resolution, but **before** Inky transforms components into table markup. This means you can use template logic around Inky components:

```html
{% if show_hero %}
<hero>
  <h1>{{ headline }}</h1>
</hero>
{% endif %}

<button href="{{ cta_url }}">{{ cta_text }}</button>
```

The full pipeline order is:

1. Layout resolution (`<layout>`)
2. Custom component resolution (`<ink-*>`)
3. Include resolution (`<include>`)
4. **Data merge** (MiniJinja)
5. SCSS extraction + framework CSS injection
6. Inky component transformation
7. CSS inlining

## Missing Keys

By default, missing keys render as empty strings (lenient mode). This means `{{ undefined_var }}` produces no output rather than an error.

## Language Bindings

### Node.js (WASM)

```js
const inky = require("inky");

const html = inky.transformWithData(
  '<button href="{{ url }}">{{ text }}</button>',
  JSON.stringify({ url: "https://example.com", text: "Click" })
);
```

### PHP / Python / Ruby / Go (FFI)

All FFI bindings expose `inky_transform_with_data(html, data_json)` where `data_json` is a JSON string. This runs data merge and CSS inlining only -- no layouts, includes, or framework CSS.

```python
import inky
html = inky.transform_with_data(
    '<button href="{{ url }}">{{ text }}</button>',
    '{"url": "https://example.com", "text": "Click"}'
)
```

For the full pipeline (layouts, includes, framework CSS, and everything else `inky build` does), pass `data` as an option to `build` instead -- a native dict/hash/map, not a JSON string:

```python
result = inky.build(html, base_path="src/emails", data={"user": {"name": "Alice"}})
```

See [Language Bindings](bindings.md#build-pipeline) for `build`'s full option list and per-language signatures (Node.js/WASM has no `build`; use `transformWithData` there, or the CLI for the full pipeline).

## When NOT to Use Data Merging

If you're sending emails through an ESP (SendGrid, Mailchimp, Postmark, etc.) that handles its own template merging, **don't use `--data`**. Instead, let Inky pass your merge tags through untouched (the default behavior), and let the ESP fill in the data at send time.

Data merging is best for:

- **Previewing** emails with sample data during development
- **Generating** final static HTML when you handle sending yourself
- **Testing** that templates render correctly with different data

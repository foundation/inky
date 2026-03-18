# Getting Started

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap foundation/inky
brew install inky
```

### npm

```bash
npm install -g inky-wasm
```

### Cargo (from source)

```bash
cargo install inky-cli
```

### Direct download

```bash
curl -fsSL https://get.inky.email/install.sh | sh
```

## Your First Email

### 1. Scaffold a project

```bash
inky init my-email
cd my-email
```

This creates:

```
my-email/
├── inky.config.json
├── src/
│   ├── layouts/
│   │   └── default.html
│   ├── styles/
│   │   └── theme.scss
│   ├── partials/
│   │   ├── header.inky
│   │   └── footer.inky
│   └── emails/
│       └── welcome.inky
└── dist/
```

### 2. Edit your template

```html
<!-- src/emails/welcome.inky -->
<layout src="../layouts/default.html" title="Welcome!" preheader="Thanks for joining.">
<container>
  <row>
    <column sm="12" lg="12">
      <h1>Welcome!</h1>
      <p>We're glad you're here.</p>
      <button href="https://example.com/get-started">Get Started</button>
    </column>
  </row>
</container>
```

### 3. Build

```bash
inky build
```

Output goes to `dist/`. That's it.

## CLI Commands

### `inky build`

Transform `.inky` and `.html` files into email-ready HTML.

```bash
# Single file (auto-outputs .html alongside .inky)
inky build email.inky

# Single file to specific output
inky build email.inky -o output.html

# Directory to directory
inky build src/ -o dist/

# Pipe from stdin
echo '<button href="#">Click</button>' | inky build

# Skip CSS inlining (on by default)
inky build email.inky --no-inline-css

# Skip framework CSS injection
inky build email.inky --no-framework-css

# Custom column count (default: 12)
inky build email.inky --columns 16

# Strict mode -- exit 1 on warnings
inky build src/ -o dist/ --strict
```

### `inky watch`

Rebuild automatically on file changes.

```bash
inky watch src/emails -o dist
```

Watches all `.inky` and `.html` files, plus any referenced partials and layouts. When a partial or layout changes, all templates rebuild. When a single template changes, only that file rebuilds.

### `inky validate`

Check templates for common email issues.

```bash
inky validate email.inky
inky validate src/
```

| Rule | Severity | What it checks |
|------|----------|----------------|
| `v1-syntax` | warning | Deprecated v1 syntax |
| `missing-alt` | warning | Images without `alt` text |
| `button-no-href` | error | Buttons without `href` |
| `missing-container` | warning | No `<container>` element |
| `missing-preheader` | warning | No preheader/preview text |
| `email-too-large` | warning | HTML > 90KB (Gmail clips at 102KB) |
| `style-block-too-large` | warning | `<style>` > 8KB (Gmail strips entire block) |
| `img-no-width` | warning | Images without `width` (breaks Outlook) |
| `deep-nesting` | warning | Tables nested > 5 levels |

Exit codes: `0` success, `1` errors, `2` warnings (with `--strict`).

### `inky migrate`

Convert v1 syntax to v2. See the [Migration Guide](migration.md).

```bash
inky migrate email.inky             # preview to stdout
inky migrate src/ -o migrated/      # directory to directory
inky migrate src/ --in-place        # rewrite files in-place
```

### `inky init`

Scaffold a new project.

```bash
inky init my-project
```

## CSS

### Framework CSS

Inky includes a built-in SCSS framework for responsive email styles. It is injected automatically during build. Override variables in your layout (see the full [Style Reference](styles.md) for all available variables):

```html
<!-- Inline SCSS overrides -->
<style type="text/scss">
$primary-color: #ff6600;
$global-font-family: Georgia, serif;
</style>

<!-- Or link to an external file -->
<link rel="stylesheet" href="theme.scss">
```

Disable with `--no-framework-css`.

### CSS Inlining

CSS inlining is **on by default**. It moves `<style>` blocks and `<link>` stylesheets into inline `style` attributes. Media queries and at-rules that can't be inlined are preserved in a `<style>` block at the end of `<body>`.

```bash
# Default behavior: transform + inline
inky build email.inky

# Skip inlining
inky build email.inky --no-inline-css
```

## Layouts and Includes

### Layouts

A layout wraps your emails in shared HTML. Use `<yield>` where content should go:

```html
<!-- src/layouts/default.html -->
<!DOCTYPE html>
<html>
<head>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
  <meta name="viewport" content="width=device-width">
  <title>$title|$</title>
</head>
<body>
  <span class="preheader">$preheader|$</span>
  <table class="body">
    <tr><td class="center" align="center" valign="top">
      <center><yield></center>
    </td></tr>
  </table>
</body>
</html>
```

Reference it from your email:

```html
<layout src="../layouts/default.html" title="Hello" preheader="Quick update">
<container>
  <row><column>Content here</column></row>
</container>
```

### Includes (Partials)

```html
<include src="../partials/header.inky" logo="https://example.com/logo.png">

<container>
  <row><column>Email body</column></row>
</container>

<include src="../partials/footer.inky">
```

Partials can include other partials (max depth: 10).

### Custom Components

Create reusable components with the `ink-` prefix. Any tag like `<ink-NAME>` resolves to the file `components/NAME.inky` in your project.

**Define a component** (`src/components/cta.inky`):

```html
<row>
  <column sm="12" lg="12">
    <center>
      <button href="$href$" class="$color|primary$">$text|Learn More$</button>
    </center>
    <yield>
  </column>
</row>
```

**Use it in your email:**

```html
<ink-cta href="https://example.com" text="Get Started">
  <p>Extra content below the button</p>
</ink-cta>
```

- Attributes become template variables (`$href$`, `$text$`, `$color$`)
- Inner content replaces `<yield>` in the component
- Self-closing works too: `<ink-cta href="https://example.com" />`
- Components can nest inside other components
- Variable defaults use the `$name|default$` syntax
- Max nesting depth: 10 (prevents circular references)

Configure the components directory in `inky.config.json`:

```json
{
  "components": "src/components"
}
```

The default directory is `components` relative to the input file.

### Template Variables

Pass variables as attributes on `<layout>` and `<include>` tags. Use `$name$` placeholders inside the referenced file:

```html
$title$              <!-- required, left as-is if not provided -->
$title|My Email$     <!-- falls back to "My Email" -->
$preheader|$         <!-- falls back to empty string -->
```

## Data Merging

Inky can merge JSON data into your templates using `--data`:

```bash
inky build email.inky --data data.json
```

```json
{"user": {"name": "Alice"}, "cta_url": "https://example.com"}
```

```html
<button href="{{ cta_url }}">Hello {{ user.name }}!</button>
```

This is off by default — without `--data`, merge tags pass through untouched. See the full [Data Merging](data-merging.md) guide.

## Template-Friendly

Inky auto-detects and preserves common template syntaxes. No `<raw>` tags needed:

| Syntax | Languages |
|--------|-----------|
| `{{ variable }}` | Handlebars, Mustache, Jinja2, Twig, Blade |
| `<%= expression %>` | ERB, EJS |
| `<% code %>` | ERB, EJS, ASP |
| `{% tag %}` | Jinja2, Twig, Nunjucks, Django |
| `${expression}` | ES6 template literals |
| `*\|MERGE_TAG\|*` | Mailchimp |
| `%%variable%%` | Salesforce Marketing Cloud |

## Configuration File

Place `inky.config.json` in your project root:

```json
{
  "src": "src/emails",
  "dist": "dist",
  "columns": 12,
  "data": "data.json"
}
```

The `data` field is optional. When set, templates are merged with the JSON data during build. See [Data Merging](data-merging.md).

With this in place, just run `inky build` or `inky watch` with no arguments.

CLI flags always override config file values.

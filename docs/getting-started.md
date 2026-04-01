---
raw: true
title: "Installation & Usage"
description: "Install Inky via Homebrew, npm, or Cargo. Scaffold an email project, build templates, and preview with live reload."
nav_group: "Getting Started"
nav_order: 1
---

# Getting Started

## Installation

### Homebrew (macOS/Linux)

```bash
brew install foundation/inky/inky
```

### npm

```bash
npm install -g inky
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

```sh
my-email/
├── inky.config.json
├── AGENT.md
├── CLAUDE.md → AGENT.md
├── .cursorrules → AGENT.md
├── .github/
│   └── copilot-instructions.md → AGENT.md
├── src/
│   ├── layouts/
│   │   └── default.html
│   ├── styles/
│   │   └── theme.scss
│   ├── partials/
│   │   ├── header.inky
│   │   └── footer.inky
│   ├── components/
│   │   └── cta.inky
│   └── emails/
│       └── welcome.inky
├── data/
│   └── welcome.json
└── dist/
```

`AGENT.md` contains project conventions and component syntax for AI coding assistants. The symlinks (`CLAUDE.md`, `.cursorrules`, `copilot-instructions.md`) ensure the instructions are auto-discovered by Claude Code, Cursor, and GitHub Copilot respectively.

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

# Hybrid output (div + MSO ghost tables)
inky build email.inky --hybrid

# Generate plain text version alongside HTML
inky build src/ -o dist/ --plain-text

# Use per-template data files (data/welcome.json for src/welcome.inky)
inky build src/ -o dist/ --data data/

# VML bulletproof buttons for Outlook
inky build email.inky --bulletproof-buttons

# JSON output (for AI agents and scripts)
inky build email.inky --json
echo '<button href="#">Hi</button>' | inky build --json
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

# Pipe from stdin
echo '<img src="photo.jpg">' | inky validate

# JSON output
inky validate src/ --json
echo '<img src="photo.jpg">' | inky validate --json
```

| Rule | Severity | What it checks |
|------|----------|----------------|
| `v1-syntax` | warning | Deprecated v1 syntax |
| `missing-alt` | warning | Images without `alt` text |
| `generic-alt` | warning | Generic alt text like "image", "logo", or single character |
| `button-no-href` | error | Buttons without `href` |
| `empty-link` | error/warning | Empty href (error) or placeholder `#` href (warning) |
| `insecure-link` | warning | Links using `http://` instead of `https://` |
| `bad-shortlink` | warning | URL shorteners that get blocked (bit.ly, youtu.be, t.co, etc.) |
| `mailto-in-button` | warning | `mailto:` href on a `<button>` component |
| `missing-container` | warning | No `<container>` element |
| `missing-preheader` | warning | No preheader/preview text |
| `gmail-clipping` | warning/error | HTML approaching or exceeding Gmail's 102KB clip limit |
| `style-block-too-large` | warning | `<style>` > 8KB (Gmail strips entire block) |
| `img-no-width` | warning | Images without `width` (breaks Outlook) |
| `deep-nesting` | warning | Tables nested > 5 levels |
| `low-contrast` | warning | Text/background color fails WCAG AA contrast ratio |
| `outlook-unsupported-css` | warning | CSS grid, flexbox, or border-radius (Outlook ignores) |
| `gmail-strips-class` | warning | Class names with `.` or `:` that Gmail strips |
| `spam-all-caps` | warning | Over 20% of text is ALL CAPS |
| `spam-exclamation` | warning | Three or more consecutive exclamation marks |
| `spam-image-heavy` | warning | High image-to-text ratio |
| `spam-missing-unsubscribe` | warning | No unsubscribe link found |
| `spam-suspicious-phrases` | warning | 3+ common spam trigger phrases detected |

Exit codes: `0` success, `1` errors, `2` warnings (with `--strict`).

### `inky serve`

Start a local dev server with live preview and auto-reload.

```bash
inky serve src/emails
inky serve src/emails --port 8080
inky serve src/emails --data data.json
```

Opens an index page at `http://localhost:3000` listing all templates. Click any template to preview the rendered output. Edits to source files or data automatically trigger a browser reload.

### `inky spam-check`

Check templates for common spam triggers.

```bash
inky spam-check email.inky
inky spam-check src/

# Pipe from stdin
echo '<p>FREE MONEY!!!</p>' | inky spam-check

# JSON output
inky spam-check src/ --json
echo '<p>FREE MONEY!!!</p>' | inky spam-check --json
```

Checks for ALL CAPS text, excessive exclamation marks, high image-to-text ratio, missing unsubscribe link, and common spam trigger phrases. Exit code `1` if any issues found.

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

## JSON Output

The `build`, `validate`, and `spam-check` commands support `--json` for machine-readable output. This is useful for CI pipelines, editor integrations, and AI agents.

```bash
inky validate src/ --json
```

```json
{
  "files": [
    {
      "path": "src/emails/welcome.inky",
      "diagnostics": [
        {
          "severity": "warning",
          "rule": "missing-alt",
          "message": "1 image(s) missing alt text"
        }
      ]
    }
  ],
  "summary": {
    "files": 1,
    "errors": 0,
    "warnings": 1
  }
}
```

For `build --json`, each file entry also includes an `html` field with the transformed output.

## Configuration File

Place `inky.config.json` in your project root:

```json
{
  "src": "src/emails",
  "dist": "dist",
  "columns": 12,
  "data": "data.json",
  "data_dir": "data",
  "hybrid": false,
  "plain_text": false,
  "bulletproof_buttons": false
}
```

Optional fields:
- `data` — merge all templates with a single JSON data file (see [Data Merging](data-merging.md))
- `data_dir` — directory of per-template JSON data files (`data/welcome.json` pairs with `src/welcome.inky`)
- `hybrid` — use hybrid `<div>` + MSO ghost table output (see [Hybrid Output](hybrid-output.md))
- `plain_text` — generate `.txt` plain text version alongside each HTML file
- `bulletproof_buttons` — generate VML bulletproof buttons for Outlook on all `<button>` components

With this in place, just run `inky build` or `inky watch` with no arguments.

CLI flags always override config file values.

# Component Reference

Every Inky component transforms simple HTML into email-safe table markup. All layout tables include `role="presentation"` for accessibility.

---

## Layout Components

### `<container>`

Centers content and sets the max width.

**Attributes:** `class`

```html
<container>Content here</container>
```

Output:

```html
<table role="presentation" align="center" class="container">
  <tbody><tr><td>Content here</td></tr></tbody>
</table>
```

---

### `<row>`

Creates a horizontal row of columns.

**Attributes:** `class`

```html
<row>
  <column lg="6">Left</column>
  <column lg="6">Right</column>
</row>
```

Output:

```html
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
```

---

### `<column>`

A grid column inside a `<row>`. Based on a 12-column grid (configurable).

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `sm` | `12` | Small screen column span |
| `lg` | auto | Large screen column span (auto-calculated from sibling count) |
| `no-expander` | -- | Suppress the expander element on full-width columns |
| `class` | -- | Additional CSS classes |

```html
<!-- Explicit sizing -->
<column sm="12" lg="4">One third</column>
<column sm="12" lg="8">Two thirds</column>

<!-- Full width, no expander -->
<column lg="12" no-expander>No expander</column>

<!-- Auto-sized (12 / number of siblings) -->
<row>
  <column>Half</column>
  <column>Half</column>
</row>
```

---

### `<wrapper>`

Wraps content in a full-width background section.

**Attributes:** `class`

```html
<wrapper class="header">Content</wrapper>
```

Output:

```html
<table role="presentation" class="wrapper header" align="center">
  <tbody><tr><td class="wrapper-inner">Content</td></tr></tbody>
</table>
```

---

### `<block-grid>`

Equal-width item grid.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `up` | -- | Number of items per row |
| `class` | -- | Additional CSS classes |

```html
<block-grid up="3">
  <td>Item 1</td>
  <td>Item 2</td>
  <td>Item 3</td>
</block-grid>
```

Output:

```html
<table role="presentation" class="block-grid up-3">
  <tbody><tr>
    <td>Item 1</td>
    <td>Item 2</td>
    <td>Item 3</td>
  </tr></tbody>
</table>
```

---

## Content Components

### `<button>`

A bulletproof email button.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `href` | -- | Link URL |
| `target` | -- | Link target (`_blank`, etc.) |
| `size` | -- | `tiny`, `small`, `large` |
| `color` | -- | `primary`, `secondary`, `success`, `alert`, `warning` |
| `expand` | -- | Full-width button (bare attribute) |
| `radius` | -- | Rounded corners (bare attribute) |
| `rounded` | -- | Pill-shaped (bare attribute) |
| `hollow` | -- | Outline style (bare attribute) |
| `class` | -- | Additional CSS classes |

```html
<button href="https://example.com">Click Me</button>

<button href="#" size="small" color="alert" expand>
  Expanded Alert Button
</button>
```

Output:

```html
<table role="presentation" class="button">
  <tbody><tr><td>
    <table role="presentation"><tbody><tr><td>
      <a href="https://example.com">Click Me</a>
    </td></tr></tbody></table>
  </td></tr></tbody>
</table>
```

---

### `<spacer>`

Vertical whitespace.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `height` | `16` | Height in pixels |
| `sm` | -- | Small screen height (responsive) |
| `lg` | -- | Large screen height (responsive) |
| `class` | -- | Additional CSS classes |

```html
<spacer height="16"></spacer>
<spacer sm="10" lg="20"></spacer>
```

Output (simple):

```html
<table role="presentation" class="spacer">
  <tbody><tr><td height="16" style="font-size:16px;line-height:16px;">&nbsp;</td></tr></tbody>
</table>
```

Output (responsive): generates two tables -- one with `hide-for-large` and one with `show-for-large`.

---

### `<divider>`

A horizontal rule.

**Attributes:** `class`

```html
<divider></divider>
```

Output:

```html
<table role="presentation" class="divider">
  <tbody><tr><th>&nbsp;</th></tr></tbody>
</table>
```

---

### `<callout>`

A highlighted content box.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `color` | -- | `primary`, `secondary`, `success`, `alert`, `warning` |
| `class` | -- | Additional CSS classes |

```html
<callout color="primary">Important message here.</callout>
```

Output:

```html
<table role="presentation" class="callout">
  <tbody><tr>
    <th class="callout-inner primary">Important message here.</th>
    <th class="expander"></th>
  </tr></tbody>
</table>
```

---

### `<menu>` / `<item>`

Horizontal or vertical navigation menu.

**Menu attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `direction` | -- | `vertical` for vertical layout |
| `align` | -- | `center` to center the menu |
| `class` | -- | Additional CSS classes |

**Item attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `href` | -- | Link URL |
| `target` | -- | Link target |
| `class` | -- | Additional CSS classes |

```html
<menu align="center" direction="vertical">
  <item href="/about">About</item>
  <item href="/contact" target="_blank">Contact</item>
</menu>
```

---

### `<image>`

Responsive image with proper width handling for email clients.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `src` | -- | Image URL |
| `alt` | -- | Alt text (required) |
| `width` | -- | Width in pixels |
| `retina` | -- | Renders at half width for high-DPI displays (bare attribute) |
| `class` | -- | Additional CSS classes |

```html
<image src="hero.jpg" alt="Hero banner" width="600">
<image src="logo.png" alt="Logo" width="400" retina>
```

The `retina` flag sets the `width` attribute to half the source value (e.g., `width="400"` with `retina` displays at 200px), so the full-resolution image is shown at half size on retina screens.

---

### `<raw>`

Prevents Inky from transforming the contents.

```html
<raw>
  <table><tr><td>This won't be touched by Inky</td></tr></table>
</raw>
```

The `<raw>` wrapper is stripped from the output. Most template syntaxes (`{{ }}`, `<%= %>`, etc.) are auto-detected and don't need `<raw>`.

---

## Media Components

### `<video>`

HTML5 video with poster fallback for email clients.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `src` | -- | Video file URL (mp4) |
| `poster` | -- | Poster image URL |
| `href` | `src` value | Link destination for fallback |
| `width` | `600` | Width in pixels |
| `alt` | `Video` | Alt text for poster image |

```html
<video src="movie.mp4" poster="poster.jpg" href="https://example.com/watch" width="600" alt="Watch now">
```

Apple Mail/iOS play the video natively. All other clients show the poster image linked to `href`.

---

### `<hero>`

Full-width background image section with VML fallback for Outlook.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `background` | -- | Background image URL |
| `width` | `600` | Width in pixels |
| `height` | `400` | Height in pixels |
| `class` | -- | Additional CSS classes |

```html
<hero background="hero.jpg" width="600" height="400">
  <h1>Welcome</h1>
  <p>Hero section with background image</p>
</hero>
```

Uses CSS `background-image` for modern clients and VML `v:rect` for Outlook.

---

## Social Components

### `<social>` / `<social-link>`

A row of social media icon links.

**`<social>` attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `align` | `center` | Horizontal alignment |
| `class` | -- | Additional CSS classes |

**`<social-link>` attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `platform` | -- | Platform name (see list below) |
| `href` | `#` | Profile/page URL |
| `icon` | -- | Custom icon image URL |
| `color` | platform default | Override the default platform color |

```html
<social align="center">
  <social-link platform="facebook" href="https://fb.com/mypage">Facebook</social-link>
  <social-link platform="twitter" href="https://twitter.com/me">Twitter</social-link>
  <social-link platform="instagram" href="https://ig.com/me" icon="custom-ig.png">IG</social-link>
</social>
```

If no label text is provided, the platform name is auto-capitalized.

#### Supported Platforms (19)

| Platform | Default Color | Platform | Default Color |
|----------|--------------|----------|--------------|
| `facebook` | `#3b5998` | `pinterest` | `#bd081c` |
| `twitter` | `#1da1f2` | `snapchat` | `#fffc00` |
| `x` | `#000000` | `threads` | `#000000` |
| `instagram` | `#e1306c` | `mastodon` | `#6364ff` |
| `linkedin` | `#0077b5` | `bluesky` | `#0085ff` |
| `youtube` | `#ff0000` | `discord` | `#5865f2` |
| `github` | `#333333` | `whatsapp` | `#25d366` |
| `tiktok` | `#000000` | `telegram` | `#0088cc` |
| `reddit` | `#ff4500` | `dribbble` | `#ea4c89` |
| `behance` | `#1769ff` | | |

---

## Cards and Content Components

### `<card>`

A card with optional image, title, and body.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `image` | -- | Card image URL |
| `title` | -- | Card title text |
| `href` | -- | Makes the image clickable |
| `class` | -- | Additional CSS classes |

```html
<card image="photo.jpg" title="Card Title" href="https://example.com">
  Card body content goes here.
</card>
```

Output: a bordered table with separate rows for image, title, and body. The image is wrapped in a link when `href` is provided.

---

### `<alert>`

A notification/alert banner.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `type` | `info` | `success`, `warning`, `error`, `info` |
| `color` | type default | Override background color |
| `class` | -- | Additional CSS classes |

**Default colors by type:**

| Type | Color |
|------|-------|
| `success` | `#dff0d8` |
| `warning` | `#fcf8e3` |
| `error` | `#f2dede` |
| `info` | `#d9edf7` |

```html
<alert type="success">Operation completed!</alert>
<alert type="warning" color="#fff3cd">Custom color warning</alert>
```

---

### `<badge>`

An inline pill/label element.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `color` | `#333333` | Background color |
| `text-color` | `#ffffff` | Text color |
| `class` | -- | Additional CSS classes |

```html
<badge color="#e74c3c" text-color="#ffffff">New</badge>
<badge>Default</badge>
```

Output:

```html
<span class="badge" style="display: inline-block; padding: 2px 8px; background-color: #e74c3c; color: #ffffff; border-radius: 12px; font-size: 12px; font-weight: bold; line-height: 1.4;">New</span>
```

---

### `<accordion>` / `<accordion-item>`

Collapsible content sections.

**`<accordion>` attributes:** `class`

**`<accordion-item>` attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `title` | `Untitled` | Section title |
| `class` | -- | Additional CSS classes |

```html
<accordion>
  <accordion-item title="Section 1">
    Content for section 1.
  </accordion-item>
  <accordion-item title="Section 2">
    Content for section 2.
  </accordion-item>
</accordion>
```

Each item renders as a table row with a title bar and content area.

---

### `<blockquote>`

A styled quotation with left border.

**Attributes:**

| Attribute | Default | Description |
|-----------|---------|-------------|
| `cite` | -- | Attribution text |
| `color` | `#cccccc` | Left border color |
| `class` | -- | Additional CSS classes |

```html
<blockquote cite="Jane Doe" color="#3498db">
  This is a quoted passage.
</blockquote>
```

Output:

```html
<table role="presentation" class="blockquote" width="100%" cellpadding="0" cellspacing="0" style="border-left: 4px solid #3498db;">
  <tbody>
    <tr><td style="padding: 0 0 0 16px; font-style: italic;">This is a quoted passage.</td></tr>
    <tr><td style="padding: 8px 0 0 16px; font-size: 14px; color: #999999;">&mdash; Jane Doe</td></tr>
  </tbody>
</table>
```

---

## Conditional Components

### `<outlook>`

Content that only renders in Microsoft Outlook (mso).

```html
<outlook>
  <table width="600"><tr><td>Outlook-only fallback</td></tr></table>
</outlook>
```

Output:

```html
<!--[if mso]>
<table width="600"><tr><td>Outlook-only fallback</td></tr></table>
<![endif]-->
```

---

### `<not-outlook>`

Content that renders everywhere *except* Outlook.

```html
<not-outlook>
  <div style="max-width: 600px;">Modern layout</div>
</not-outlook>
```

Output:

```html
<!--[if !mso]><!-->
<div style="max-width: 600px;">Modern layout</div>
<!--<![endif]-->
```

---

### `<preview>`

Hidden preheader text visible in inbox preview but invisible in the email body.

```html
<preview>Check out our latest sale!</preview>
```

Output: a hidden `<div>` with the text, followed by zero-width space padding to prevent body content from leaking into the inbox preview.

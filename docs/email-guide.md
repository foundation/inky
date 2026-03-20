---
raw: true
title: "Email Development Guide"
---

# Email Development Guide

A practical guide to building HTML emails that render correctly across every major client. These tips apply whether you're using Inky's CLI, language bindings, or writing raw HTML.

---

## How Email HTML Differs from Web HTML

Email clients are not browsers. Most strip `<style>` blocks, ignore `<div>` layout, and have no JavaScript support. The key differences:

- **Tables for layout** — `<table>`, `<tr>`, `<td>`/`<th>` are the only reliable layout primitives. Inky generates these for you from simple tags like `<row>` and `<column>`.
- **Inline CSS** — Gmail, Outlook.com, and many others strip `<style>` tags. All critical styles must be inlined on each element. Inky does this automatically (`--no-inline-css` to disable).
- **No JavaScript** — Every email client strips `<script>` tags entirely.
- **No `<div>` layout** — `<div>` has no reliable rendering in Outlook 2007–2019 (Word rendering engine). Inky's default table mode avoids this. The `--hybrid` mode uses `<div>` with MSO ghost-table fallbacks for Outlook.
- **Limited CSS** — No flexbox, no grid, no `position`, no `float` (in most clients), no CSS variables. Stick to `width`, `padding`, `margin`, `background`, `color`, `font-*`, `text-align`, `border`, and `vertical-align`.

---

## Container Width

Email content should be **580–600px wide**. Inky defaults to 580px (`$global-width` in SCSS). This ensures content fits in the preview pane of desktop clients without horizontal scrolling.

---

## The Grid

Inky uses a **12-column responsive grid**. Columns stack on mobile (below 596px) and sit side-by-side on desktop.

```html
<container>
  <row>
    <column lg="8" sm="12">Main content</column>
    <column lg="4" sm="12">Sidebar</column>
  </row>
</container>
```

- `lg` sets the desktop width (out of 12).
- `sm` sets the mobile width (out of 12). Defaults to 12 (full width) if omitted.
- The breakpoint is **596px** by default (`$global-breakpoint` in SCSS).

---

## CSS Support by Client

### Universally safe

`background-color`, `border`, `color`, `font-family`, `font-size`, `font-style`, `font-weight`, `line-height`, `padding`, `text-align`, `text-decoration`, `vertical-align`, `width`

### Safe with caveats

| Property | Notes |
|----------|-------|
| `margin` | Works in most clients. **Outlook ignores margin on `<p>` and `<div>`**. Use `padding` on `<td>`/`<th>` or the `<spacer>` component instead. |
| `background-image` | Works in Apple Mail, iOS Mail, Outlook.com, Yahoo. **Outlook desktop ignores it** (use VML fallback). |
| `border-radius` | Works everywhere except **Outlook desktop**. Inky's bulletproof buttons use VML `<v:roundrect>` to handle this. |
| `max-width` | Ignored by **Outlook desktop**. Use fixed `width` on tables. |
| `rgba()` / `hsla()` | Ignored by **Outlook desktop** and some older clients. Use full 6-digit hex colors (`#ff0000`, not `#f00`). |

### Not supported

`float`, `position`, `display: flex`, `display: grid`, `calc()`, `var()`, `@keyframes`, `box-shadow` (Outlook), `object-fit`

### CSS selector rules

When writing custom CSS for emails:

- Apply classes and IDs to `<table>` and `<td>`/`<th>` elements, **not** `<tr>` elements — many clients ignore styles on `<tr>`.
- Apply `padding` only to `<td>`/`<th>` cells, not to `<table>` or `<tr>`.
- Keep selectors simple — some clients strip complex selectors. Single class selectors (`.my-class`) are safest.
- Gmail prefixes all class names (e.g., `.button` becomes `.m_button`), so avoid overly generic names that might collide.

---

## Outlook Quirks

Outlook 2007–2019 on Windows uses the **Microsoft Word rendering engine**, which has severe HTML/CSS limitations. Key issues:

### Margin and padding
- Outlook desktop ignores `margin` on many elements. Use `padding` on table cells.
- **Outlook.com** has a separate quirk: it strips lowercase `margin` but respects capitalized `Margin`. If you write custom CSS for Outlook.com compatibility, include both:
  ```css
  margin: 10px;
  Margin: 10px;
  ```
- For vertical spacing, use the `<spacer>` component — it generates a table row with a fixed height, which is reliable everywhere.

### Conditional comments
Outlook supports `<!--[if mso]>` conditional comments. Inky uses these internally for VML buttons and hybrid mode fallbacks. You can also use the `<outlook>` and `<not-outlook>` components:

```html
<outlook>This only renders in Outlook.</outlook>
<not-outlook>This renders everywhere else.</not-outlook>
```

### Background images
Outlook ignores CSS `background-image`. For hero sections with background images, use Inky's `<hero>` component, which generates VML fallbacks automatically.

### DPI scaling
Outlook on high-DPI displays can scale images unexpectedly. Always set explicit `width` and `height` attributes on `<img>` tags.

---

## Images

### Always set dimensions
```html
<image src="photo.jpg" alt="Description" width="300" height="200" />
```
Explicit `width` and `height` prevent layout shifts and Outlook scaling issues.

### Always include alt text
Alt text displays when images are blocked (many corporate email clients block images by default). Inky's validator catches missing `alt` attributes.

### Style your alt text
When images are blocked, you can make alt text more visually appealing by adding inline styles directly to the `<img>` tag:

```html
<img src="hero.jpg" alt="Spring Collection"
  width="600" height="200"
  style="font-size: 24px; font-family: Georgia, serif; font-weight: bold; color: #1a73b5;" />
```

This way, even with images blocked, readers see styled text rather than plain alt text.

### Use absolute image URLs
Always use full URLs for images, not relative paths. Email clients load images from your server — relative paths won't resolve. Host images on your own server or CDN:

```html
<!-- Good -->
<image src="https://cdn.example.com/hero.jpg" alt="Hero" width="600" />

<!-- Bad — won't work in email clients -->
<image src="images/hero.jpg" alt="Hero" width="600" />
```

### Retina images
For sharp images on high-DPI screens, use images that are **2x** the display dimensions and constrain with `width`:

```html
<image src="logo@2x.png" alt="Logo" width="200" />
```

The source image should be 400px wide.

### Animated GIFs
Animated GIFs work in most clients. **Outlook desktop shows only the first frame** — make sure the first frame conveys your message.

### File size
Keep total email size (HTML + images) under **100KB** if possible. Gmail clips emails larger than **102KB**. Inky's validator warns about Gmail clipping risk.

---

## Buttons

Never use an image as a button — images get blocked. Inky generates **bulletproof buttons** using nested tables with VML fallbacks for Outlook:

```html
<button href="https://example.com" color="primary" size="large">Click Here</button>
```

This renders as a styled, clickable button in every client, including Outlook.

**Important:** Always include the `href` attribute. Outlook.com requires it for proper rendering.

---

## Typography

### Font stacks
Use web-safe font stacks. Custom web fonts (`@font-face`) work in Apple Mail, iOS Mail, and some Android clients, but are ignored by Gmail and Outlook.

Safe defaults: `Helvetica, Arial, sans-serif` or `Georgia, Times, serif`.

### Font sizes
- Body text: **14–16px** minimum. Many mobile clients enforce a minimum anyway.
- The Inky SCSS framework defaults to 16px (`$global-font-size`).
- Use `<small>` or the Inky typography size classes for secondary text.

### Line height
Use `line-height` as a unitless multiplier or pixel value. Percentage-based line-height can be inconsistent across clients.

---

## Colors

- Use **6-digit hex** codes: `#ff6600`, not `#f60`. Shorthand hex (`#f60`) fails in some older clients.
- `rgb()` works in most clients.
- `rgba()` and `hsla()` are **not supported in Outlook desktop**. Use solid colors with hex fallbacks.

---

## iOS Auto-Detected Links

iOS Mail automatically detects dates, times, addresses, and phone numbers and turns them into blue, underlined links. This can clash with your email's design (e.g., a white date on a dark background suddenly turns blue).

Inky's SCSS framework includes a fix for this — the `$remove-ios-blue` variable (default: `true`) generates CSS that strips the auto-link styling. If you need to customize the behavior, override it in your SCSS:

```scss
// Disable the iOS blue link fix
$remove-ios-blue: false;
```

If you're writing custom CSS, the workaround is to wrap the text in an `<a>` tag styled to match the surrounding text:

```html
<a href="#" style="color: #ffffff; text-decoration: none;">March 25, 2026</a>
```

---

## Responsive Design

Inky uses a **desktop-first** approach: the base layout is for large screens, and a `@media` query at `max-width: 596px` handles mobile.

### What works on mobile
- Columns stack to full width automatically.
- Font sizes can be adjusted with responsive classes.
- `<spacer>` supports responsive heights: `<spacer sm="10" lg="20">`.
- Visibility classes show/hide content per breakpoint (see below).

### Visibility classes

Show or hide content based on screen size:

| Class | Behavior |
|-------|----------|
| `.show-for-large` | Visible on desktop only, hidden on mobile |
| `.hide-for-large` | Visible on mobile only, hidden on desktop |

```html
<row>
  <column lg="12">
    <!-- Desktop-only image -->
    <div class="show-for-large">
      <image src="banner-wide.jpg" alt="Banner" width="580" />
    </div>
    <!-- Mobile-only image -->
    <div class="hide-for-large">
      <image src="banner-narrow.jpg" alt="Banner" width="300" />
    </div>
  </column>
</row>
```

**Outlook warning:** Outlook desktop does not support the CSS media queries that power visibility classes. Content hidden with `.show-for-large` or `.hide-for-large` may still appear in Outlook. To properly hide content from Outlook, wrap it in conditional comments using the `<not-outlook>` component:

```html
<not-outlook>
  <div class="hide-for-large">
    Mobile-only content (also hidden from Outlook)
  </div>
</not-outlook>
```

**Important:** Apply visibility classes to a wrapper element (`<div>`, `<td>`), not directly to an `<img>` tag — some clients ignore classes on images.

### Text alignment classes

Responsive text alignment utilities:

| Class | Behavior |
|-------|----------|
| `.text-left` | Left-align at all sizes |
| `.text-center` | Center at all sizes |
| `.text-right` | Right-align at all sizes |
| `.small-text-left` | Left-align on mobile only |
| `.small-text-center` | Center on mobile only |
| `.small-text-right` | Right-align on mobile only |

Combine them for responsive alignment:

```html
<column lg="12" class="text-center small-text-left">
  Centered on desktop, left-aligned on mobile
</column>
```

### Clients that ignore media queries
- **Gmail app on Android** (renders the desktop/large breakpoint)
- **Gmail IMAP** on some third-party clients
- **Outlook desktop**

For these clients, the desktop layout is what users see. Design your large breakpoint to be readable at narrow widths too.

### Progressive enhancement

Design for your most constrained client first (usually Outlook), then layer on enhancements for capable clients. This means:

1. Ensure the **table-based layout** looks correct without media queries
2. Add **responsive behavior** via `@media` queries for clients that support them
3. Add **visual enhancements** (border-radius, background images, web fonts) for modern clients

If you have analytics on your audience's email clients, prioritize testing for the clients your readers actually use.

---

## Alignment

### Text alignment
Inky supports alignment on most components via the `align` attribute or CSS `text-align`:

```html
<column lg="12" align="center">Centered content</column>
```

### Centering images
Use the `align` attribute on the `<image>` component:

```html
<image src="logo.png" alt="Logo" width="200" align="center" />
```

### Vertical alignment
Use the `valign` attribute on columns:

```html
<column lg="6" valign="middle">Vertically centered</column>
```

Values: `top` (default), `middle`, `bottom`.

---

## Dark Mode

Many email clients now support dark mode. Inky's SCSS framework includes dark mode overrides using `@media (prefers-color-scheme: dark)`.

Clients with dark mode support:
- Apple Mail / iOS Mail
- Outlook.com / Outlook apps
- Gmail (partial — adjusts colors automatically)
- Yahoo Mail

Key tips:
- Use transparent PNGs for logos so they work on both light and dark backgrounds.
- Test with both color schemes.
- Use the dark mode SCSS variables in `_settings.scss` to customize dark mode colors.

---

## File Size and Gmail Clipping

Gmail **clips** emails larger than **102KB** (HTML only, not including images). When an email is clipped, Gmail shows a "View entire message" link, and tracking pixels may not fire.

To keep emails small:
- Minimize inline CSS — Inky's inliner already deduplicates styles.
- Avoid large chunks of hidden content.
- Use `inky validate` to check for Gmail clipping risk.
- Use `inky build --no-framework-css` if you're providing your own CSS and don't need the full framework.

---

## Testing

### Recommended tools
- **[Litmus](https://litmus.com)** — renders your email across 90+ clients and devices.
- **[Email on Acid](https://emailonacid.com)** — similar rendering previews with accessibility checks.
- **`inky validate`** — catches structural issues, missing alt text, Outlook problems, and spam triggers before you send.
- **`inky spam-check`** — checks for common spam trigger words and patterns.

### Testing workflow
1. Build with `inky build` (or `inky watch` during development).
2. Run `inky validate` to catch structural issues.
3. Run `inky spam-check` for deliverability issues.
4. Preview locally with `inky serve`.
5. Send test emails through your ESP or use Litmus/Email on Acid for cross-client rendering.

### Key clients to test
At a minimum, test in:
- **Outlook desktop** (2016 or 2019) — the most restrictive renderer
- **Gmail** (web) — strips `<style>` tags, so inline CSS is critical
- **Apple Mail / iOS Mail** — generally the most capable, good baseline
- **Outlook.com** (web) — different rendering from Outlook desktop
- **Yahoo Mail** — unique quirks with `<style>` handling

---

## Accessibility

- **All layout tables** use `role="presentation"` — Inky adds this automatically so screen readers don't announce table structure.
- **Alt text on images** — always include it. `inky validate` flags missing alt text.
- **Semantic headings** — use `<h1>` through `<h6>` in order. Don't skip heading levels.
- **Sufficient color contrast** — 4.5:1 ratio minimum for body text.
- **Link text** — avoid "click here." Use descriptive link text: "View your order" instead.
- **Language attribute** — include `lang="en"` (or appropriate language) on the root `<html>` element.

---

## Design Tips

### Think above the fold
Put your most important content — the key message, primary CTA — near the top of the email. Many readers scan the preview pane without scrolling.

### Keep it simple
- Stick to **one or two fonts** throughout the email. Mixing many typefaces looks cluttered.
- Use a clear visual hierarchy: one primary call-to-action, with secondary actions clearly subordinate.
- Single-column layouts are more reliable across clients and easier to read on mobile.

### Footer best practices
Every email footer should include:
- Physical mailing address (required by CAN-SPAM)
- Unsubscribe link (required)
- A brief reminder of why the reader is receiving the email ("You're receiving this because you signed up at...")
- Contact information or support links
- Social media links (use the `<social>` component)

### External resources

- **[Campaign Monitor CSS Support](https://www.campaignmonitor.com/css/)** — comprehensive, regularly updated guide to which CSS properties work in which email clients
- **[Can I Email](https://www.caniemail.com/)** — "Can I Use" but for email clients
- **[Litmus](https://litmus.com)** — cross-client rendering tests
- **[Email on Acid](https://emailonacid.com)** — cross-client rendering with accessibility checks

---

## Sending with ESPs

If you use an Email Service Provider (Mailchimp, SendGrid, Postmark, etc.) that handles its own merge tags:

- **Do not** use Inky's `--data` flag — let the ESP handle merge tags.
- Inky auto-detects and preserves common merge tag syntax: `{{ handlebars }}`, `<%= erb %>`, `*|mailchimp|*`, `{%- jinja -%}`, etc.
- Build your templates with `inky build`, then upload the output HTML to your ESP.

---

## CAN-SPAM and Legal Requirements

Every commercial email must include:
- Your **physical mailing address**
- A working **unsubscribe link**
- Accurate **From** and **Subject** lines

Your ESP typically handles unsubscribe management. Include the address in your email footer.

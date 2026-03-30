---
raw: true
title: "Email Client Compatibility"
nav_group: "Guides"
nav_order: 4
---

# Email Client Compatibility

Inky generates HTML that works across all major email clients. This table shows the rendering engine used by each client and any notable limitations.

---

## Support Matrix

### Desktop Clients

| Client | Rendering Engine | Grid | Media Queries | `<style>` Block | Background Images | VML Buttons | Notes |
|--------|-----------------|------|---------------|-----------------|-------------------|-------------|-------|
| **Apple Mail** | WebKit | Yes | Yes | Yes | Yes | N/A | Best-in-class rendering |
| **Outlook 2007** | Word | Yes | No | Partial | No | Yes | Most restrictive. Use VML for advanced styling. |
| **Outlook 2010** | Word | Yes | No | Partial | No | Yes | Same as 2007 |
| **Outlook 2013** | Word | Yes | No | Partial | No | Yes | Same as 2007 |
| **Outlook 2016** | Word | Yes | No | Partial | No | Yes | Same as 2007 |
| **Outlook 2019** | Word | Yes | No | Partial | No | Yes | Same as 2007 |
| **Thunderbird** | Gecko | Yes | Yes | Yes | Yes | N/A | Generally reliable |

### Webmail

| Client | Grid | Media Queries | `<style>` Block | Background Images | Notes |
|--------|------|---------------|-----------------|-------------------|-------|
| **Gmail** (web) | Yes | Yes | Yes (since 2016) | Yes | Strips unsupported CSS properties. Prefixes class names. |
| **Outlook.com** | Yes | Yes | Yes | Yes | Different engine from Outlook desktop. Generally good. |
| **Yahoo Mail** | Yes | Yes | Yes | Yes | Good support. Some quirks with `<style>` specificity. |
| **Office 365** (web) | Yes | Yes | Yes | Yes | Similar to Outlook.com |
| **AOL Mail** | Yes | Yes | Yes | Yes | Similar to Yahoo Mail (same parent company) |
| **ProtonMail** | Yes | No | Partial | Yes | Strips some styles for security. Images blocked by default. |
| **Fastmail** | Yes | Yes | Yes | Yes | Good overall support |

### Mobile

| Client | Grid | Media Queries | `<style>` Block | Notes |
|--------|------|---------------|-----------------|-------|
| **iOS Mail** | Yes | Yes | Yes | WebKit-based. Excellent rendering. |
| **Gmail app (iOS)** | Yes | Yes | Yes | Better support than Android counterpart |
| **Gmail app (Android)** | Yes | **No** | **No** | Renders large/desktop breakpoint. Does not support media queries. |
| **Outlook app (iOS/Android)** | Yes | Yes | Yes | Good rendering, different from Outlook desktop |
| **Samsung Mail** | Yes | Yes | Yes | WebKit-based |
| **Yahoo app** | Yes | Yes | Yes | Good support |

---

## Rendering Engine Notes

### Word (Outlook Desktop 2007–2019)

The most restrictive rendering environment. Microsoft Outlook on Windows uses the Word HTML renderer, which means:

- **No `float`, `position`, flexbox, or grid** — tables only
- **No `background-image`** via CSS — use VML or the `<hero>` component
- **No `border-radius`** via CSS — Inky uses VML `<v:roundrect>` for bulletproof buttons
- **No `max-width`** — use fixed `width` on tables
- **No `rgba()` or `hsla()`** — use solid hex colors
- **`margin` is unreliable** — use `padding` on table cells or the `<spacer>` component
- **DPI scaling** affects images — always set explicit `width` and `height` on images
- **Conditional comments work** — `<!--[if mso]>` lets you target Outlook specifically

Inky handles all of these automatically in its generated output.

### WebKit (Apple Mail, iOS)

The most capable rendering engine for email. Supports:
- Media queries
- Web fonts (`@font-face`)
- CSS animations (limited use in email)
- `background-image` with `background-size`
- `border-radius`
- Most modern CSS

### Gmail

Gmail has improved significantly since 2016 when it added `<style>` block support. Current behavior:
- Supports `<style>` blocks in `<head>`
- Prefixes all class names (e.g., `.button` becomes `.m_button`) — avoid overly generic names
- Strips unsupported CSS properties
- Clips emails over **102KB** — use `inky validate` to check
- Gmail app on Android still does **not** support media queries

---

## What Inky Does Automatically

You don't need to memorize this table. Inky handles cross-client compatibility in its output:

| Problem | Inky's Solution |
|---------|----------------|
| Outlook ignores `<div>` layout | Generates `<table>` markup (or MSO fallbacks in `--hybrid` mode) |
| Gmail strips `<style>` | Inlines CSS by default |
| Outlook ignores `border-radius` | VML `<v:roundrect>` bulletproof buttons |
| Outlook ignores `background-image` | VML background in `<hero>` component |
| Images blocked by default | Validator flags missing `alt` text |
| Gmail clips large emails | Validator warns about 102KB threshold |
| Outlook spacing unreliable | `<spacer>` generates a fixed-height table row |
| Accessibility | `role="presentation"` on all layout tables |

---

## Breakpoint

Inky uses a single responsive breakpoint at **596px** (`$global-breakpoint` in SCSS).

- **Above 596px** — desktop/large layout (columns side by side)
- **Below 596px** — mobile/small layout (columns stack)

This single breakpoint is sufficient for email because:
- Email preview panes are typically 500–700px wide
- Mobile screens are below 596px
- More breakpoints add complexity with minimal benefit in the email context

---

## Testing Recommendations

Rendering varies across clients and changes over time. Always test before sending:

1. **`inky validate`** — catches structural issues and common problems
2. **`inky serve`** — local preview with auto-reload
3. **[Litmus](https://litmus.com)** or **[Email on Acid](https://emailonacid.com)** — cross-client rendering screenshots
4. **Send real test emails** — especially to Outlook desktop and Gmail

---
raw: true
title: "Troubleshooting"
nav_group: "Guides"
nav_order: 5
---

# Troubleshooting

Common issues and how to fix them.

---

## Build Issues

### "Unknown component" error

```sh
error: unknown component <columns> at line 5
  hint: did you mean <column>? (singular in v2)
  hint: run `inky migrate` to convert v1 syntax automatically
```

**Cause:** You're using v1 syntax (e.g., `<columns>`, `<h-line>`, `class="expand"`).

**Fix:** Run the migrator:

```bash
inky migrate src/ --in-place
```

See the [Migration Guide](migration.md) for the full list of v1 to v2 changes.

---

### CSS inlining makes the file too large

**Cause:** The full SCSS framework is compiled and injected into each email, then inlined onto every element.

**Fix:** Options to reduce size:

```bash
# Skip the framework CSS if you provide your own
inky build --no-framework-css

# Skip inlining if your ESP handles it
inky build --no-inline-css

# Both
inky build --no-framework-css --no-inline-css
```

You can also override SCSS variables to disable components you don't use, reducing the generated CSS.

---

### SCSS compilation fails

**Cause:** The `sass` CLI (Dart Sass) is required to compile the SCSS framework.

**Fix:** Install Dart Sass:

```bash
# npm
npm install -g sass

# Homebrew
brew install sass/sass/sass
```

Then verify: `sass --version` should show 1.x or later.

**Note:** `sassc` (LibSass) is deprecated and does not support Inky's SCSS, which uses the `@use` module system.

---

## Rendering Issues

### Columns not stacking on mobile

**Possible causes:**
1. **Gmail app on Android** — does not support media queries. Columns will always render at the large breakpoint. This is a Gmail limitation, not an Inky bug.
2. **Missing responsive attributes** — if you set `lg` but not `sm`, columns default to `sm="12"` (full width), which is usually correct. Check your column attributes.
3. **CSS inlining disabled** — if you pass `--no-inline-css` and your ESP doesn't inline styles, the responsive styles may not apply.

---

### Buttons look wrong in Outlook

**Possible causes:**
1. **Missing `href`** — Outlook.com requires `href` on buttons. Always include it.
2. **Custom CSS overriding VML** — Inky generates VML `<v:roundrect>` fallbacks for Outlook. If you add custom button CSS, test in Outlook to make sure it doesn't conflict.
3. **Very long button text** — Outlook VML buttons have a fixed width. Very long text may overflow. Keep button text concise.

---

### Background images not showing in Outlook

**Expected behavior.** Outlook desktop (Word engine) ignores CSS `background-image`.

**Fix:** Use Inky's `<hero>` component, which generates VML background fallbacks for Outlook:

```html
<hero background="https://example.com/bg.jpg">
  <row>
    <column>Content over the image</column>
  </row>
</hero>
```

---

### Email looks different in dark mode

Dark mode behavior varies by client:

- **Apple Mail / iOS** — applies `prefers-color-scheme: dark` and inverts colors
- **Gmail** — auto-darkens colors (no media query control)
- **Outlook app** — supports `prefers-color-scheme`
- **Outlook desktop** — no dark mode in the Word engine

**Tips:**
- Use transparent PNGs for logos
- Customize dark mode colors via the SCSS variables in `_settings.scss`
- Test in Apple Mail dark mode — it's the most aggressive at color inversion
- Add light borders or outlines to images that might disappear on dark backgrounds

---

### Spacing is inconsistent in Outlook

**Cause:** Outlook ignores `margin` on many elements and renders `padding` inconsistently on some.

**Fix:** Use the `<spacer>` component for reliable vertical spacing:

```html
<spacer height="20"></spacer>
```

For responsive spacing:

```html
<spacer sm="10" lg="20"></spacer>
```

Spacer generates a table row with a fixed height, which works in every client.

---

### Images are scaled/blurry in Outlook

**Cause:** Outlook on high-DPI displays scales images based on Windows display scaling.

**Fix:** Always set explicit `width` and `height` attributes:

```html
<image src="photo.jpg" alt="Description" width="300" height="200" />
```

For retina images, use a 2x source image with the display dimensions in the attributes.

---

## Validation Issues

### "Gmail clipping risk"

**Cause:** Your HTML output exceeds or approaches 102KB. Gmail will clip the email and show a "View entire message" link.

**Fix:**
- Remove unnecessary content or hidden elements
- Use `--no-framework-css` if you don't need the full SCSS framework
- Simplify complex layouts
- Check if large data merges are bloating the HTML

---

### "Missing alt text"

**Cause:** An `<img>` or `<image>` tag is missing the `alt` attribute.

**Fix:** Add descriptive alt text to every image:

```html
<image src="product.jpg" alt="Blue running shoes, side view" width="300" />
```

For decorative images that convey no information, use an empty alt: `alt=""`.

---

### "Spam trigger detected"

**Cause:** `inky spam-check` found words or patterns commonly flagged by spam filters.

**Common triggers:**
- ALL CAPS text ("FREE", "BUY NOW", "ACT NOW")
- Excessive exclamation marks ("Amazing deal!!!")
- Spammy phrases ("click here", "limited time", "no obligation")
- Large image-to-text ratio
- Missing unsubscribe link

**Fix:** Reword flagged content. The spam checker output shows the specific triggers and their locations.

---

## Data Merging Issues

### Merge tags not replaced

**Cause:** Variables in the template don't match keys in the JSON data file.

**Fix:** Check that:
1. Template uses `{{ variable_name }}` (Jinja2 syntax)
2. JSON data file has matching keys: `{"variable_name": "value"}`
3. You're passing `--data data.json` to the build command
4. The data file is valid JSON

Inky uses lenient mode by default — missing keys render as empty strings rather than causing errors.

---

### Merge tags rendered literally in output

**Cause:** If you see `{{ name }}` in the output HTML, data merging may not be enabled.

**Fix:** Make sure you're passing the data flag:

```bash
inky build --data data.json
```

Or set it in `inky.config.json`:

```json
{
  "data": "data.json"
}
```

**Note:** If you're using an ESP that handles its own merge tags (Mailchimp, SendGrid, etc.), do **not** use `--data`. Inky auto-preserves ESP merge tag syntax in the output.

---

## CLI Issues

### `inky` command not found

**Fix by install method:**

```bash
# Homebrew
brew tap foundation/inky && brew install inky

# Cargo
cargo install inky-cli

# npm
npm install -g inky

# Direct download — make sure the binary is on your PATH
export PATH="$PATH:/path/to/inky"
```

Verify: `inky --version`

---

### `inky watch` not detecting changes

**Possible causes:**
1. **Wrong directory** — make sure your source files are in the `src/` directory (or wherever your config points)
2. **File extension** — the watcher looks for `.inky` and `.html` files by default
3. **OS file watch limits** — on Linux, you may need to increase `inotify` limits:
   ```bash
   echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
   sudo sysctl -p
   ```

---

## Still Stuck?

- Run `inky validate` for automated diagnostics
- Check the [Component Reference](components.md) for correct syntax
- Open an issue: https://github.com/foundation/inky/issues

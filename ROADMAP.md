# Inky v2 — Remaining Work

All design, component reference, migration guide, CLI usage, bindings docs, and API reference have moved to `docs/`.

## Ship It

| Task | Status |
|------|--------|
| Publish v2.0.0 to npm, crates.io, Packagist, PyPI, RubyGems, Homebrew | TODO |
| Archive `foundation/inky-rb` with pointer to new gem | TODO |
| Archive `foundation/foundation-emails` with pointer to `inky` | TODO |
| Submit formula to `homebrew-core` (once project is established) | TODO |
| Re-enable Dependabot | TODO |

## Future (v2.x / v3.0)

| Feature | Description |
|---------|-------------|
| Email previews | Validation warnings for common client rendering quirks (expand beyond Outlook/Gmail) |
| i18n / RTL support | Auto `dir="rtl"` and mirrored layouts for right-to-left locales |
| ESP integration | Push templates to SendGrid, Mailchimp, Postmark APIs (`inky upload`) |
| S3/CDN image upload | `inky build --upload-images s3://bucket/` to host images and rewrite URLs |

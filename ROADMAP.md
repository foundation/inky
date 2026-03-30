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
| MCP server | Expose inky as an MCP tool server (`inky mcp`) so AI agents can discover and call transform, validate, spam-check, and migrate directly without shelling out. Blocked on MCP spec improvements — current protocol has context bloat and stateful data flow issues. Revisit after mid-2026 spec release. See below for details. |

## MCP Server (Future)

Expose inky capabilities as MCP tools via `inky mcp`, using JSON-RPC over stdio:

| Tool | Input | Output |
|------|-------|--------|
| `inky_transform` | HTML string + options (columns, inline_css, hybrid, etc.) | Transformed email HTML |
| `inky_validate` | HTML string | Array of diagnostics (severity, rule, message) |
| `inky_spam_check` | HTML string | Array of spam diagnostics |
| `inky_migrate` | v1 HTML string | v2 HTML string + list of changes |
| `inky_list_components` | — | All available components with attributes and examples |

**Why wait:** MCP currently consumes significant context tokens just for tool schemas (50K+ tokens for a moderately-sized server). The `--json` + stdin CLI approach shipped in v2 is more token-efficient for now. Anthropic and the MCP community are working on fixes (lazy loading, stateful sessions) targeting mid-2026.

**Implementation plan:** Add as a subcommand (`inky mcp`) in the existing CLI binary rather than a separate crate, so there's zero additional install for users. All transform/validate logic already exists in `inky-core`.

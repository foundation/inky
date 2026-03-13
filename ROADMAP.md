# Inky v2 — Remaining Work

All design, component reference, migration guide, CLI usage, bindings docs, and API reference have moved to `docs/`.

## Stage 5: Ship It

| Step | Task | Status |
|------|------|--------|
| 40 | Set up CI (`.github/workflows/ci.yml`) | Done |
| 41 | Set up release pipeline (`.github/workflows/release.yml`) | Done |
| 42 | Create `foundation/homebrew-inky` tap repo | Done |
| 43 | Automate Homebrew formula updates on release | Done (in release workflow) |
| 44 | Write documentation and migration guide (`docs/`) | Done |
| 44a | Create Go bindings (`foundation/inky-go` — 21 tests) | Done |
| 45 | Publish v2.0.0 to npm, crates.io, Packagist, PyPI, RubyGems, Homebrew | TODO |
| 46 | Archive `foundation/inky-rb` with pointer to new gem | TODO |
| 47 | Archive `foundation/foundation-emails` with pointer to `inky` | TODO |
| 48 | Submit formula to `homebrew-core` (once project is established) | TODO |
| 49 | Re-enable Dependabot | TODO |

## Future (v2.x / v3.0)

| Feature | Description |
|---------|-------------|
| Hybrid output mode | `<div>` layout with Outlook table fallbacks |
| Contrast checker | Validates color accessibility |
| Live preview | `inky serve` with browser preview |

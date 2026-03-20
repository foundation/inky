# Foundation for Emails Archive & Redirect Plan

This document contains everything needed to archive the `foundation/foundation-emails`
repo and redirect its users to Inky 2. Work through each section in order.

---

## Step 1: Ensure Inky 2 is ready to receive traffic

Before touching the old repo, confirm:

- [ ] Inky 2 README on `develop` (or `main`) is public-ready
- [ ] Docs site / GitHub Pages is live (if applicable)
- [ ] `inky-email` npm package is published (or ready to publish)
- [ ] `inky-cli` is on crates.io / Homebrew tap (or ready)

---

## Step 2: Update foundation-emails README

Replace the **entire** contents of `foundation-emails/README.md` with:

```markdown
# Foundation for Emails (Archived)

> **This project has been replaced by [Inky 2](https://github.com/foundation/inky).**
> Foundation for Emails is no longer maintained. All future development happens in the Inky repo.

## What is Inky 2?

Inky 2 is a ground-up rewrite of Foundation for Emails in Rust. It is faster,
more capable, and ships as a single binary with bindings for Node.js, PHP, Python,
Ruby, and Go.

Highlights:

- 10-100x faster builds
- 27 components (up from 10)
- Built-in CSS inlining, validation, live preview, and spam checking
- Cleaner syntax — attributes over classes
- Layouts, includes, and template variables
- Automatic migration tool: `inky migrate`

## Migrating

Inky 2 includes a CLI migrator that converts v1 syntax to v2 automatically:

    # Install Inky 2
    brew tap foundation/inky && brew install inky
    # — or —
    cargo install inky-cli
    # — or —
    npm install -g inky-email

    # Migrate your templates
    inky migrate src/ --in-place

For the full migration guide, see:
https://github.com/foundation/inky/blob/develop/docs/migration.md

## Links

- **Inky 2 repo:** https://github.com/foundation/inky
- **Migration guide:** https://github.com/foundation/inky/blob/develop/docs/migration.md
- **Documentation:** https://github.com/foundation/inky/tree/develop/docs

## Historical documentation

The original Foundation for Emails documentation remains in this repo's
git history for reference. Browse any prior commit or tag to access it.
```

---

## Step 3: Update repo metadata

Before archiving, update these fields in the GitHub repo settings:

- **Description:** `Archived — replaced by Inky 2: https://github.com/foundation/inky`
- **Website URL:** `https://github.com/foundation/inky`
- **Topics:** add `archived`, `deprecated`, `email`, `responsive-email`

---

## Step 4: Pin a final issue

Create and pin this issue in `foundation-emails`:

- **Title:** `Foundation for Emails has moved to Inky 2`
- **Body:**

```markdown
Foundation for Emails is now **Inky 2** — a complete rewrite in Rust with a
modern CLI, 27 components, built-in CSS inlining, validation, and more.

**New repo:** https://github.com/foundation/inky

### How to migrate

Inky 2 includes an automatic migration tool:

```bash
inky migrate src/ --in-place
```

Full migration guide: https://github.com/foundation/inky/blob/develop/docs/migration.md

### What happens to this repo?

This repo is now archived (read-only). All issues, PRs, and history remain
accessible but no new changes will be accepted. Please open new issues in the
[Inky repo](https://github.com/foundation/inky/issues).
```

---

## Step 5: Publish a final npm release (if applicable)

If `foundation-emails` is on npm, publish one last version that warns users:

### Option A: npm deprecate (no code change needed)

```bash
npm deprecate foundation-emails "This package has been replaced by inky-email. See https://github.com/foundation/inky"
```

### Option B: Publish a final version with a postinstall warning

Add to `package.json`:

```json
{
  "scripts": {
    "postinstall": "echo '\\n\\n  WARNING: foundation-emails is deprecated. Use inky-email instead.\\n  See https://github.com/foundation/inky\\n\\n'"
  }
}
```

Then bump the version and publish. Option A is simpler and preferred.

---

## Step 6: Archive the repo

In GitHub repo Settings > Danger Zone > Archive this repository.

This makes the repo read-only. The README, issues, and all history remain visible.

---

## Step 7: Post-archive announcements

### GitHub Discussion / Blog post (draft)

```
# Foundation for Emails is now Inky 2

After years of serving the email development community, Foundation for Emails
has been rebuilt from the ground up as **Inky 2**.

Inky 2 is written in Rust and ships as a single CLI binary, a WASM module, and
native bindings for Node.js, PHP, Python, Ruby, and Go. It's 10-100x faster,
supports 27 components, and includes built-in CSS inlining, validation, a live
preview server, and a spam checker.

### Getting started

    brew tap foundation/inky && brew install inky
    inky init my-email
    inky build

### Migrating from Foundation for Emails

    inky migrate src/ --in-place

The migrator automatically converts v1 syntax (plural <columns>, class-based
styles, <h-line>, etc.) to v2. See the full migration guide:
https://github.com/foundation/inky/blob/develop/docs/migration.md

### Links

- Repo: https://github.com/foundation/inky
- Docs: https://github.com/foundation/inky/tree/develop/docs
- npm: https://www.npmjs.com/package/inky-email
```

### Social media (short form)

```
Foundation for Emails is now Inky 2 — a ground-up rewrite in Rust.

Single binary CLI, 27 components, built-in CSS inlining, validation,
live preview, and bindings for Node/PHP/Python/Ruby/Go.

Automatic migration tool included: `inky migrate src/ --in-place`

https://github.com/foundation/inky
```

---

## Step 8: SEO / discoverability

The Inky 2 README already includes this line, which is good:

> Inky was formerly known as "Foundation for Emails." Starting with v2,
> everything is unified under the Inky brand.

Additional steps:

- [ ] If there's a Foundation for Emails docs site, add a redirect or banner to Inky 2
- [ ] Update the Stack Overflow `foundation-emails` tag wiki (if one exists)
- [ ] Update any ZURB/Foundation website references

---

## Checklist summary

- [ ] Inky 2 repo is public-ready
- [ ] foundation-emails README replaced with redirect
- [ ] Repo description and website URL updated
- [ ] Final issue pinned
- [ ] npm package deprecated
- [ ] Repo archived
- [ ] Announcement posted (blog / social / forums)
- [ ] Docs site redirected (if applicable)

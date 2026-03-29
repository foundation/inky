use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process;

#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_file as symlink;

pub fn cmd_init(name: Option<String>) {
    let project_dir = match &name {
        Some(n) => PathBuf::from(n),
        None => std::env::current_dir().unwrap_or_else(|e| {
            eprintln!(
                "{} Could not determine current directory: {}",
                "error:".red().bold(),
                e
            );
            process::exit(1);
        }),
    };

    // If a name was given, check the directory doesn't already exist with content
    if name.is_some() && project_dir.exists() {
        let has_content = fs::read_dir(&project_dir)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false);
        if has_content {
            eprintln!(
                "{} Directory '{}' already exists and is not empty",
                "error:".red().bold(),
                project_dir.display()
            );
            process::exit(1);
        }
    }

    // Create directory structure
    let dirs = [
        "src/layouts",
        "src/partials",
        "src/components",
        "src/styles",
        "src/emails",
        "data",
        "dist",
    ];

    for dir in &dirs {
        let full = project_dir.join(dir);
        fs::create_dir_all(&full).unwrap_or_else(|e| {
            eprintln!(
                "{} Failed to create {}: {}",
                "error:".red().bold(),
                full.display(),
                e
            );
            process::exit(1);
        });
    }

    // Write files
    let files: Vec<(&str, &str)> = vec![
        ("inky.config.json", CONFIG_JSON),
        ("AGENT.md", AGENT_MD),
        ("src/layouts/default.html", LAYOUT_DEFAULT),
        ("src/styles/theme.scss", STYLES_THEME),
        ("src/partials/header.inky", PARTIAL_HEADER),
        ("src/partials/footer.inky", PARTIAL_FOOTER),
        ("src/components/cta.inky", COMPONENT_CTA),
        ("src/emails/welcome.inky", EMAIL_WELCOME),
        ("data/welcome.json", DATA_WELCOME),
    ];

    for (rel_path, content) in &files {
        let full = project_dir.join(rel_path);
        fs::write(&full, content).unwrap_or_else(|e| {
            eprintln!(
                "{} Failed to write {}: {}",
                "error:".red().bold(),
                full.display(),
                e
            );
            process::exit(1);
        });
    }

    // Create agent.md symlinks for various AI tools
    let agent_symlinks: Vec<(&str, &str)> = vec![
        ("CLAUDE.md", "AGENT.md"),
        (".cursorrules", "AGENT.md"),
        (".github/copilot-instructions.md", "../AGENT.md"),
    ];

    // .github directory is needed for Copilot symlink
    let github_dir = project_dir.join(".github");
    fs::create_dir_all(&github_dir).unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to create {}: {}",
            "error:".red().bold(),
            github_dir.display(),
            e
        );
        process::exit(1);
    });

    for (link_path, target) in &agent_symlinks {
        let full = project_dir.join(link_path);
        symlink(target, &full).unwrap_or_else(|e| {
            eprintln!(
                "{} Failed to create symlink {}: {}",
                "error:".red().bold(),
                full.display(),
                e
            );
            process::exit(1);
        });
    }

    // Print summary
    let display_root = name.as_deref().unwrap_or(".");
    eprintln!();
    if name.is_some() {
        print_created(&format!("{}/", display_root));
    }
    for (rel_path, _) in &files {
        print_created(rel_path);
    }

    eprintln!();
    eprintln!("  {}:", "Get started".bold());
    if name.is_some() {
        eprintln!("    cd {}", display_root);
    }
    eprintln!("    inky build");
    eprintln!("    inky watch");
    eprintln!();
    eprintln!("  {}:", "Preview with sample data".bold());
    eprintln!("    inky build --data data");
    eprintln!();
}

fn print_created(path: &str) {
    eprintln!("  {} {}", "created".green().bold(), path);
}

const CONFIG_JSON: &str = r#"{
  "src": "src/emails",
  "dist": "dist",
  "columns": 12,
  "components": "src/components"
}
"#;

// Note: "data" is not included in the default config so merge tags
// pass through untouched. Users can add "data": "data" to merge
// per-template data, or use: inky build --data data

const LAYOUT_DEFAULT: &str = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
  <meta name="viewport" content="width=device-width">
  <title>$title|$</title>
  <link rel="stylesheet" href="../styles/theme.scss">
  <!-- You can also use inline SCSS overrides instead of a linked file:
  <style type="text/scss">
  $primary-color: #1a73b5;
  $global-font-family: Helvetica, Arial, sans-serif;
  $global-width: 580px;
  $body-background: #f3f3f3;
  $container-background: #fefefe;
  </style>
  -->
</head>
<body>
  <span class="preheader">$preheader|$</span>
  <table class="body" data-made-with-inky>
    <tr>
      <td class="center" align="center" valign="top">
        <center>
          <!-- email content goes here -->
          <yield>
        </center>
      </td>
    </tr>
  </table>
</body>
</html>
"#;

const STYLES_THEME: &str = r#"// Inky Theme
// Uncomment and edit variables to customize your email styles.
// See all available variables: https://github.com/foundation/inky/blob/develop/docs/styles.md

// Colors
// $primary-color: #1a73b5;
// $secondary-color: #777777;
// $success-color: #3adb76;
// $warning-color: #ffae00;
// $alert-color: #ec5840;

// Layout
// $global-width: 580px;
// $global-gutter: 16px;
// $body-background: #f3f3f3;
// $container-background: #fefefe;

// Typography
// $body-font-family: Helvetica, Arial, sans-serif;
// $global-font-size: 16px;
// $global-font-color: #0a0a0a;
// $header-font-family: $body-font-family;

// Buttons
// $button-background: $primary-color;
// $button-color: #fefefe;
// $button-font-weight: bold;
// $button-radius: 3px;

// Dark Mode
// $dark-body-background: #1a1a1a;
// $dark-container-background: #2d2d2d;
// $dark-font-color: #f0f0f0;
// $dark-link-color: #5ab5f7;
"#;

const PARTIAL_HEADER: &str = r#"<wrapper class="header">
  <container>
    <row>
      <column sm="12" lg="12">
        <img src="https://placehold.co/200x50?text=Logo" alt="Company logo" width="200">
      </column>
    </row>
  </container>
</wrapper>
"#;

const PARTIAL_FOOTER: &str = r##"<wrapper class="footer">
  <container>
    <row>
      <column sm="12" lg="12">
        <p class="text-center"><small>You're receiving this because you signed up. <a href="{{ unsubscribe_url }}">Unsubscribe</a></small></p>
      </column>
    </row>
  </container>
</wrapper>
"##;

const COMPONENT_CTA: &str = r#"<row>
  <column sm="12" lg="12">
    <center>
      <button href="$href$" class="$color|primary$">$text|Learn More$</button>
    </center>
    <yield>
  </column>
</row>
"#;

const EMAIL_WELCOME: &str = r#"<layout src="../layouts/default.html" title="Welcome!" preheader="Thanks for signing up — here's how to get started.">
<include src="../partials/header.inky">

<container>
  <row>
    <column sm="12" lg="12">
      <h1>Welcome, {{ user_name }}!</h1>
      <p>Thanks for signing up. We're excited to have you on board.</p>
    </column>
  </row>

  <!-- Custom component: resolves to src/components/cta.inky -->
  <ink-cta href="{{ cta_url }}" text="Get Started">
    <spacer height="16"></spacer>
    <p class="text-center"><small>Questions? Just reply to this email.</small></p>
  </ink-cta>
</container>

<include src="../partials/footer.inky">
"#;

const DATA_WELCOME: &str = r#"{
  "user_name": "Alice",
  "cta_url": "https://example.com/get-started",
  "unsubscribe_url": "https://example.com/unsubscribe"
}
"#;

const AGENT_MD: &str = r##"# Inky Email Project

This is an email project built with [Inky](https://inky.email), a framework that converts simple HTML components into responsive, email-safe table markup.

## Commands

```bash
inky build                # Build all templates in src/emails → dist/
inky build --data data    # Build with JSON data merging
inky validate             # Check templates for accessibility, rendering, and spam issues
inky spam-check           # Detect common spam triggers
inky watch                # Watch for changes and auto-rebuild
inky serve                # Live preview dev server (port 3000)
```

Use `--json` on build, validate, or spam-check for machine-readable output.

## Project Structure

```
src/
├── emails/       # Email templates (.inky files) — the main files you edit
├── layouts/      # HTML wrapper layouts (DOCTYPE, <head>, <body> boilerplate)
├── partials/     # Reusable snippets included via <include>
├── components/   # Custom components (referenced by tag name, e.g. ink-cta → cta.inky)
└── styles/       # SCSS theme overrides (variables only, not arbitrary CSS)
data/             # JSON files for template data merging (matched by filename)
dist/             # Build output (do not edit)
inky.config.json  # Project configuration
```

## Writing Templates

Inky uses semantic HTML components that compile to email-safe table markup. Use attributes, not classes, for component options.

### Layout & Grid

Every email needs a `<container>` with `<row>` and `<column>` inside. Columns use a 12-column grid.

```html
<container>
  <row>
    <column sm="12" lg="6">Left half</column>
    <column sm="12" lg="6">Right half</column>
  </row>
</container>
```

- `sm` = small screen column width, `lg` = large screen column width
- Columns in a row should add up to 12

### Common Components

```html
<button href="https://example.com">Click Me</button>
<button href="#" size="large" class="secondary">Big Button</button>

<callout type="success">This is a success message.</callout>
<alert type="warning">Watch out!</alert>

<hero background="https://example.com/bg.jpg">
  <h1>Big Hero Section</h1>
</hero>

<spacer height="16"></spacer>
<divider></divider>

<image src="photo.jpg" alt="Description" width="580" />
```

### Layouts and Includes

Templates reference a layout and include partials:

```html
<layout src="../layouts/default.html" title="Subject" preheader="Preview text">
<include src="../partials/header.inky">

<container>
  <!-- email content -->
</container>

<include src="../partials/footer.inky">
```

### Data Merging

Use Jinja2 syntax for dynamic content. Data comes from matching JSON files in the data/ directory.

```html
<h1>Hello, {{ user_name }}!</h1>
<button href="{{ cta_url }}">Get Started</button>
```

### Custom Components

Files in src/components/ become custom tags. A file named `cta.inky` is used as `<ink-cta>`. Parameters use `$name|default$` syntax and nested content goes where `<yield>` appears.

## Validation

`inky validate` checks for: missing alt text, images without width, Gmail clipping risk (>102KB), Outlook rendering issues, and accessibility problems. Fix all errors; warnings are recommendations.
"##;

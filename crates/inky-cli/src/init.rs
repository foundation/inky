use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process;

pub fn cmd_init(name: Option<String>) {
    let project_dir = match &name {
        Some(n) => PathBuf::from(n),
        None => std::env::current_dir().unwrap_or_else(|e| {
            eprintln!("{} Could not determine current directory: {}", "error:".red().bold(), e);
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
        "src/emails",
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
        ("src/layouts/default.html", LAYOUT_DEFAULT),
        ("src/partials/header.inky", PARTIAL_HEADER),
        ("src/partials/footer.inky", PARTIAL_FOOTER),
        ("src/emails/welcome.inky", EMAIL_WELCOME),
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
    eprintln!("    inky build src/emails -o dist");
    eprintln!("    inky watch src/emails -o dist");
    eprintln!();
}

fn print_created(path: &str) {
    eprintln!("  {} {}", "created".green().bold(), path);
}

const CONFIG_JSON: &str = r#"{
  "src": "src/emails",
  "dist": "dist",
  "columns": 12
}
"#;

const LAYOUT_DEFAULT: &str = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
  <meta name="viewport" content="width=device-width">
  <title></title>
</head>
<body>
  <table class="body" data-made-with-inky>
    <tr>
      <td class="center" align="center" valign="top">
        <center>
          <!-- email content goes here -->
        </center>
      </td>
    </tr>
  </table>
</body>
</html>
"#;

const PARTIAL_HEADER: &str = r#"<wrapper class="header">
  <container>
    <row>
      <column sm="12" lg="12">
        <img src="https://placehold.co/200x50?text=Logo" alt="Logo">
      </column>
    </row>
  </container>
</wrapper>
"#;

const PARTIAL_FOOTER: &str = r##"<wrapper class="footer">
  <container>
    <row>
      <column sm="12" lg="12">
        <p class="text-center"><small>You're receiving this because you signed up. <a href="#">Unsubscribe</a></small></p>
      </column>
    </row>
  </container>
</wrapper>
"##;

const EMAIL_WELCOME: &str = r#"<include src="../partials/header.inky">

<container>
  <row>
    <column sm="12" lg="12">
      <h1>Welcome!</h1>
      <p>Thanks for signing up. We're excited to have you on board.</p>
      <button href="https://example.com">Get Started</button>
    </column>
  </row>
  <row>
    <column sm="12" lg="12">
      <spacer height="16"></spacer>
      <p>If you have any questions, just reply to this email — we'd love to help.</p>
    </column>
  </row>
</container>

<include src="../partials/footer.inky">
"#;

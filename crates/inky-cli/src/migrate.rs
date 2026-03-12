use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process;

use inky_core::migrate;

pub fn cmd_migrate(input: PathBuf, output: Option<PathBuf>, in_place: bool) {
    if input.is_dir() {
        migrate_directory(&input, output.as_deref(), in_place);
    } else {
        migrate_file(&input, output.as_deref(), in_place);
    }
}

fn migrate_file(input: &std::path::Path, output: Option<&std::path::Path>, in_place: bool) {
    let html = fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("{} Failed to read {}: {}", "error:".red().bold(), input.display(), e);
        process::exit(1);
    });

    let result = migrate::migrate(&html);

    if result.changes.is_empty() {
        eprintln!("  {} {} (no changes needed)", "ok".green().bold(), input.display());
        return;
    }

    // Report changes
    for change in &result.changes {
        eprintln!("  {} {} {}", "migrated".cyan().bold(), input.display(), change.description);
    }

    // Write output
    if in_place {
        fs::write(input, &result.html).unwrap_or_else(|e| {
            eprintln!("{} Failed to write {}: {}", "error:".red().bold(), input.display(), e);
            process::exit(1);
        });
        eprintln!("  {} {}", "wrote".green().bold(), input.display());
    } else if let Some(out) = output {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(out, &result.html).unwrap_or_else(|e| {
            eprintln!("{} Failed to write {}: {}", "error:".red().bold(), out.display(), e);
            process::exit(1);
        });
        eprintln!("  {} {} → {}", "wrote".green().bold(), input.display(), out.display());
    } else {
        // stdout
        print!("{}", result.html);
    }
}

fn migrate_directory(input: &std::path::Path, output: Option<&std::path::Path>, in_place: bool) {
    let files = find_template_files(input);

    if files.is_empty() {
        eprintln!(
            "{} No .inky or .html files found in {}",
            "warning:".yellow().bold(),
            input.display()
        );
        return;
    }

    let mut total_changes = 0;

    for file in &files {
        let html = fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("{} Failed to read {}: {}", "error:".red().bold(), file.display(), e);
            return String::new();
        });

        if html.is_empty() {
            continue;
        }

        let result = migrate::migrate(&html);

        if result.changes.is_empty() {
            continue;
        }

        total_changes += result.changes.len();

        for change in &result.changes {
            eprintln!("  {} {} {}", "migrated".cyan().bold(), file.display(), change.description);
        }

        if in_place {
            fs::write(file, &result.html).unwrap_or_else(|e| {
                eprintln!("{} Failed to write {}: {}", "error:".red().bold(), file.display(), e);
            });
        } else if let Some(out_dir) = output {
            let relative = file.strip_prefix(input).unwrap_or(file);
            let dest = out_dir.join(relative);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&dest, &result.html).unwrap_or_else(|e| {
                eprintln!("{} Failed to write {}: {}", "error:".red().bold(), dest.display(), e);
            });
        } else {
            // stdout — separate files with a comment
            println!("<!-- {} -->\n{}\n", file.display(), result.html);
        }
    }

    if total_changes > 0 {
        eprintln!(
            "\n  {} Migrated {} change(s) across {} file(s)",
            "done".green().bold(),
            total_changes,
            files.len()
        );
    } else {
        eprintln!("\n  {} All files already use v2 syntax", "done".green().bold());
    }
}

fn find_template_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for ext in &["inky", "html"] {
        let pattern = format!("{}/**/*.{}", dir.display(), ext);
        if let Ok(paths) = glob::glob(&pattern) {
            files.extend(paths.filter_map(|entry| entry.ok()));
        }
    }
    files.sort();
    files
}

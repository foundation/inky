use std::collections::HashSet;
use std::path::{Path, PathBuf};

use colored::Colorize;
use regex::Regex;

/// Base template extensions used for builds, validation, and migration.
pub const TEMPLATE_EXTENSIONS: &[&str] = &["inky", "html"];

/// Extended extensions for file watching (includes stylesheets).
pub const WATCH_EXTENSIONS: &[&str] = &["inky", "html", "scss", "css"];

/// Find all template files matching the given extensions in a directory.
pub fn find_files(dir: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for ext in extensions {
        let pattern = format!("{}/**/*.{}", dir.display(), ext);
        if let Ok(paths) = glob::glob(&pattern) {
            files.extend(paths.filter_map(|entry| entry.ok()));
        }
    }
    files.sort();
    files
}

/// Convert an input path to an output path, changing .inky to .html.
pub fn to_output_path(input: &Path, input_dir: &Path, output_dir: &Path) -> PathBuf {
    let relative = input.strip_prefix(input_dir).unwrap_or(input);
    let dest = output_dir.join(relative);
    if dest.extension().and_then(|e| e.to_str()) == Some("inky") {
        dest.with_extension("html")
    } else {
        dest
    }
}

/// Check if a file has a watchable extension (templates + stylesheets).
pub fn is_watchable_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| WATCH_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

/// Load and parse a JSON data file, logging warnings on failure.
pub fn load_json_data(path: Option<&Path>) -> Option<serde_json::Value> {
    let path = path?;
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!(
                    "  {} Failed to parse data file {}: {}",
                    "warning:".yellow().bold(),
                    path.display(),
                    e
                );
                None
            }
        },
        Err(e) => {
            eprintln!(
                "  {} Failed to read data file {}: {}",
                "warning:".yellow().bold(),
                path.display(),
                e
            );
            None
        }
    }
}

/// Scan templates for include/layout/link references and return their directories.
pub fn find_include_dirs(input_dir: &Path) -> Vec<PathBuf> {
    let include_re = Regex::new(r#"<include\s+[^>]*?src\s*=\s*"([^"]+)"[^>]*/?\s*>"#).unwrap();
    let layout_re = Regex::new(r#"<layout\s+[^>]*?src\s*=\s*"([^"]+)"[^>]*>"#).unwrap();
    let link_re =
        Regex::new(r#"<link\s+[^>]*href\s*=\s*"([^"]+\.(?:scss|css))"[^>]*/?\s*>"#).unwrap();
    let mut dirs = HashSet::new();
    let mut referenced_files: Vec<PathBuf> = Vec::new();
    let files = find_files(input_dir, TEMPLATE_EXTENSIONS);

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let base = file.parent().unwrap_or(input_dir);
            for re in [&include_re, &layout_re, &link_re] {
                for cap in re.captures_iter(&content) {
                    let ref_path = base.join(&cap[1]);
                    if let Some(parent) = ref_path.parent() {
                        if let Ok(canonical) = std::fs::canonicalize(parent) {
                            dirs.insert(canonical);
                        }
                    }
                    if re.as_str().contains("layout") {
                        referenced_files.push(ref_path);
                    }
                }
            }
        }
    }

    for layout_file in &referenced_files {
        if let Ok(content) = std::fs::read_to_string(layout_file) {
            let base = layout_file.parent().unwrap_or(input_dir);
            for cap in link_re.captures_iter(&content) {
                let ref_path = base.join(&cap[1]);
                if let Some(parent) = ref_path.parent() {
                    if let Ok(canonical) = std::fs::canonicalize(parent) {
                        dirs.insert(canonical);
                    }
                }
            }
        }
    }

    dirs.into_iter().collect()
}

use std::path::{Path, PathBuf};

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

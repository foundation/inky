use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use colored::Colorize;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use regex::Regex;

use crate::scss;
use inky_core::{Config, Inky};

const INKY_EXTENSIONS: &[&str] = &["inky", "html"];

pub fn cmd_watch(
    input: PathBuf,
    output: PathBuf,
    columns: u32,
    inline_css: bool,
    framework_css: bool,
) {
    if !input.is_dir() {
        eprintln!(
            "{} Input path '{}' is not a directory",
            "error:".red().bold(),
            input.display()
        );
        std::process::exit(1);
    }

    // Canonicalize input so it matches notify's absolute event paths
    let input = std::fs::canonicalize(&input).unwrap_or(input);
    // Ensure output dir exists, then canonicalize
    std::fs::create_dir_all(&output).ok();
    let output = std::fs::canonicalize(&output).unwrap_or(output);

    let config = Config {
        column_count: columns,
        ..Config::default()
    };

    // Initial full build
    eprintln!(
        "  {} {} → {}",
        "watching".cyan().bold(),
        input.display(),
        output.display()
    );

    do_full_build(&input, &output, &config, inline_css, framework_css);

    eprintln!("  press {} to stop\n", "Ctrl+C".bold());

    // Set up file watcher with 300ms debounce
    let (tx, rx) = mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(300), tx).unwrap_or_else(|e| {
        eprintln!("{} Failed to create file watcher: {}", "error:".red().bold(), e);
        std::process::exit(1);
    });

    debouncer
        .watcher()
        .watch(&input, notify::RecursiveMode::Recursive)
        .unwrap_or_else(|e| {
            eprintln!(
                "{} Failed to watch directory '{}': {}",
                "error:".red().bold(),
                input.display(),
                e
            );
            std::process::exit(1);
        });

    // Also watch directories containing included partials
    let include_dirs = find_include_dirs(&input);
    for dir in &include_dirs {
        if dir != &input {
            eprintln!("  {} {}", "watching".cyan().bold(), dir.display());
            debouncer
                .watcher()
                .watch(dir, notify::RecursiveMode::Recursive)
                .unwrap_or_else(|e| {
                    eprintln!(
                        "  {} Failed to watch include directory '{}': {}",
                        "warning:".yellow().bold(),
                        dir.display(),
                        e
                    );
                });
        }
    }

    // Event loop
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Collect unique changed template files
                let mut changed_files: Vec<PathBuf> = Vec::new();
                let mut needs_full_rebuild = false;

                for event in &events {
                    let path = &event.path;

                    // Only care about template files, ignore output directory
                    if !is_template_file(path) || path.starts_with(&output) {
                        continue;
                    }

                    match event.kind {
                        DebouncedEventKind::Any => {
                            if !path.exists() {
                                // File deleted
                                needs_full_rebuild = true;
                            } else if !path.starts_with(&input) {
                                // Changed file is outside input dir (i.e. an include/partial)
                                needs_full_rebuild = true;
                            } else {
                                // File modified or created in input dir
                                if !changed_files.contains(path) {
                                    changed_files.push(path.clone());
                                }
                            }
                        }
                        _ => {
                            // Ongoing writes or other events, skip
                        }
                    }
                }

                if needs_full_rebuild {
                    let timestamp = current_time();
                    eprintln!("  [{}] include or file changed, rebuilding all...", timestamp);
                    do_full_build(&input, &output, &config, inline_css, framework_css);
                } else {
                    for file in &changed_files {
                        rebuild_single_file(file, &input, &output, &config, inline_css, framework_css);
                    }
                }
            }
            Ok(Err(error)) => {
                eprintln!("  {} watch error: {}", "error:".red().bold(), error);
            }
            Err(e) => {
                eprintln!("{} Watch channel closed: {}", "error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    }
}

fn is_template_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| INKY_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

fn current_time() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;

    // Adjust for local timezone offset (rough approach using libc)
    // For simplicity, just use UTC-based display with a note
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn do_full_build(input: &Path, output: &Path, config: &Config, inline_css: bool, framework_css: bool) {
    let inky = Inky::with_config(config.clone());
    let files = find_template_files(input);

    if files.is_empty() {
        eprintln!(
            "  {} no template files found in {}",
            "warning:".yellow().bold(),
            input.display()
        );
        return;
    }

    let mut built = 0;
    for file in &files {
        match build_file(&inky, file, input, output, config, inline_css, framework_css) {
            Ok(dest) => {
                let timestamp = current_time();
                eprintln!(
                    "  [{}] {} {} → {}",
                    timestamp,
                    "built".green().bold(),
                    file.display(),
                    dest.display()
                );
                built += 1;
            }
            Err(e) => {
                eprintln!("  {} {}: {}", "error:".red().bold(), file.display(), e);
            }
        }
    }

    eprintln!(
        "  {} built {} file(s)\n",
        "done".green().bold(),
        built
    );
}

fn rebuild_single_file(
    file: &Path,
    input_dir: &Path,
    output_dir: &Path,
    config: &Config,
    inline_css: bool,
    framework_css: bool,
) {
    let inky = Inky::with_config(config.clone());
    let timestamp = current_time();

    match build_file(&inky, file, input_dir, output_dir, config, inline_css, framework_css) {
        Ok(dest) => {
            eprintln!(
                "  [{}] {} {} → {}",
                timestamp,
                "rebuilt".green().bold(),
                file.display(),
                dest.display()
            );
        }
        Err(e) => {
            eprintln!(
                "  [{}] {} {}: {}",
                timestamp,
                "error:".red().bold(),
                file.display(),
                e
            );
        }
    }
}

fn build_file(
    inky: &Inky,
    file: &Path,
    input_dir: &Path,
    output_dir: &Path,
    config: &Config,
    inline_css: bool,
    framework_css: bool,
) -> Result<PathBuf, String> {
    let html = std::fs::read_to_string(file)
        .map_err(|e| format!("Failed to read: {}", e))?;

    // Run validation
    let diagnostics = inky_core::validate::validate(&html, config);
    for d in &diagnostics {
        let label = match d.severity {
            inky_core::validate::Severity::Warning => "warn".yellow().bold(),
            inky_core::validate::Severity::Error => "error".red().bold(),
        };
        eprintln!("  {} {} [{}] {}", label, file.display(), d.rule, d.message);
    }

    let result = process_template(inky, &html, inline_css, framework_css, file.parent());

    let dest = to_output_path(file, input_dir, output_dir);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    std::fs::write(&dest, &result)
        .map_err(|e| format!("Failed to write: {}", e))?;

    Ok(dest)
}

/// Full build pipeline: resolve includes → extract SCSS overrides → compile framework CSS → inject → transform → inline.
fn process_template(
    inky: &Inky,
    html: &str,
    inline_css: bool,
    framework_css: bool,
    base_path: Option<&Path>,
) -> String {
    // Resolve <layout> tag, then <include> tags before any other processing
    let mut html = if let Some(base) = base_path {
        let with_layout = inky_core::include::process_layout(html, base).unwrap_or_else(|e| {
            eprintln!("{} {}", "error:".red().bold(), e);
            String::new()
        });
        inky_core::include::process_includes(&with_layout, base).unwrap_or_else(|e| {
            eprintln!("{} {}", "error:".red().bold(), e);
            String::new()
        })
    } else {
        html.to_string()
    };

    if framework_css {
        let (cleaned, overrides) = scss::extract_scss_overrides(&html);
        html = cleaned;

        let css = scss::compile_framework_scss(&overrides).unwrap_or_else(|e| {
            eprintln!("{} SCSS compilation failed: {}", "error:".red().bold(), e);
            String::new()
        });

        html = scss::inject_css_into_html(&html, &css);
    } else {
        let (cleaned, _) = scss::extract_scss_overrides(&html);
        html = cleaned;
    }

    if inline_css {
        inky.transform_and_inline(&html, base_path)
            .unwrap_or_else(|e| {
                eprintln!("{} CSS inlining failed: {}", "error:".red().bold(), e);
                html.clone()
            })
    } else {
        inky.transform(&html)
    }
}

fn find_template_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for ext in INKY_EXTENSIONS {
        let pattern = format!("{}/**/*.{}", dir.display(), ext);
        if let Ok(paths) = glob::glob(&pattern) {
            files.extend(paths.filter_map(|entry| entry.ok()));
        }
    }
    files.sort();
    files
}

/// Scan all templates in a directory for <include> and <layout> tags and return
/// the unique canonicalized directories containing those referenced files.
fn find_include_dirs(input_dir: &Path) -> Vec<PathBuf> {
    let include_re = Regex::new(r#"<include\s+[^>]*?src\s*=\s*"([^"]+)"[^>]*/?\s*>"#).unwrap();
    let layout_re = Regex::new(r#"<layout\s+[^>]*?src\s*=\s*"([^"]+)"[^>]*>"#).unwrap();
    let mut dirs = HashSet::new();
    let files = find_template_files(input_dir);

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let base = file.parent().unwrap_or(input_dir);
            for re in [&include_re, &layout_re] {
                for cap in re.captures_iter(&content) {
                    let ref_path = base.join(&cap[1]);
                    if let Some(parent) = ref_path.parent() {
                        if let Ok(canonical) = std::fs::canonicalize(parent) {
                            dirs.insert(canonical);
                        }
                    }
                }
            }
        }
    }

    dirs.into_iter().collect()
}

fn to_output_path(input: &Path, input_dir: &Path, output_dir: &Path) -> PathBuf {
    let relative = input.strip_prefix(input_dir).unwrap_or(input);
    let dest = output_dir.join(relative);
    if dest.extension().and_then(|e| e.to_str()) == Some("inky") {
        dest.with_extension("html")
    } else {
        dest
    }
}

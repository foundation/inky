use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use colored::Colorize;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};

use inky_core::{Config, Inky, OutputMode};

#[allow(clippy::too_many_arguments)]
pub fn cmd_watch(
    input: PathBuf,
    output: PathBuf,
    columns: u32,
    inline_css: bool,
    framework_css: bool,
    components_dir: Option<String>,
    data_path: Option<PathBuf>,
    output_mode: OutputMode,
    _plain_text: bool,
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
        output_mode,
        ..Config::default()
    };

    // Load merge data if a data file was provided
    let merge_data = crate::util::load_json_data(data_path.as_deref());

    // Initial full build
    eprintln!(
        "  {} {} → {}",
        "watching".cyan().bold(),
        input.display(),
        output.display()
    );

    do_full_build(
        &input,
        &output,
        &config,
        inline_css,
        framework_css,
        components_dir.as_deref(),
        merge_data.as_ref(),
    );

    eprintln!("  press {} to stop\n", "Ctrl+C".bold());

    // Set up file watcher with 300ms debounce
    let (tx, rx) = mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(300), tx).unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to create file watcher: {}",
            "error:".red().bold(),
            e
        );
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

    // Watch the data file for changes
    if let Some(ref data_file) = data_path {
        if let Some(parent) = data_file.parent() {
            let canonical = std::fs::canonicalize(parent).unwrap_or(parent.to_path_buf());
            eprintln!(
                "  {} {} (data)",
                "watching".cyan().bold(),
                data_file.display()
            );
            debouncer
                .watcher()
                .watch(&canonical, notify::RecursiveMode::NonRecursive)
                .unwrap_or_else(|e| {
                    eprintln!(
                        "  {} Failed to watch data file directory '{}': {}",
                        "warning:".yellow().bold(),
                        canonical.display(),
                        e
                    );
                });
        }
    }

    // Also watch directories containing included partials
    let include_dirs = crate::util::find_include_dirs(&input);
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
    let mut merge_data = merge_data;
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Collect unique changed template files
                let mut changed_files: Vec<PathBuf> = Vec::new();
                let mut needs_full_rebuild = false;
                let mut data_changed = false;

                for event in &events {
                    let path = &event.path;

                    // Check if the data file changed
                    if let Some(ref data_file) = data_path {
                        let canonical_data =
                            std::fs::canonicalize(data_file).unwrap_or(data_file.clone());
                        let canonical_event = std::fs::canonicalize(path).unwrap_or(path.clone());
                        if canonical_event == canonical_data {
                            data_changed = true;
                            continue;
                        }
                    }

                    // Only care about template files, ignore output directory
                    if !crate::util::is_watchable_file(path) || path.starts_with(&output) {
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

                // Reload data file if it changed
                if data_changed {
                    let timestamp = current_time();
                    eprintln!("  [{}] data file changed, reloading...", timestamp);
                    merge_data = crate::util::load_json_data(data_path.as_deref());
                    needs_full_rebuild = true;
                }

                if needs_full_rebuild {
                    let timestamp = current_time();
                    eprintln!(
                        "  [{}] include or file changed, rebuilding all...",
                        timestamp
                    );
                    do_full_build(
                        &input,
                        &output,
                        &config,
                        inline_css,
                        framework_css,
                        components_dir.as_deref(),
                        merge_data.as_ref(),
                    );
                } else {
                    for file in &changed_files {
                        rebuild_single_file(
                            file,
                            &input,
                            &output,
                            &config,
                            inline_css,
                            framework_css,
                            components_dir.as_deref(),
                            merge_data.as_ref(),
                        );
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

fn do_full_build(
    input: &Path,
    output: &Path,
    config: &Config,
    inline_css: bool,
    framework_css: bool,
    components_dir: Option<&str>,
    merge_data: Option<&serde_json::Value>,
) {
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
        match build_file(
            &inky,
            file,
            input,
            output,
            config,
            inline_css,
            framework_css,
            components_dir,
            merge_data,
        ) {
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

    eprintln!("  {} built {} file(s)\n", "done".green().bold(), built);
}

#[allow(clippy::too_many_arguments)]
fn rebuild_single_file(
    file: &Path,
    input_dir: &Path,
    output_dir: &Path,
    config: &Config,
    inline_css: bool,
    framework_css: bool,
    components_dir: Option<&str>,
    merge_data: Option<&serde_json::Value>,
) {
    let inky = Inky::with_config(config.clone());
    let timestamp = current_time();

    match build_file(
        &inky,
        file,
        input_dir,
        output_dir,
        config,
        inline_css,
        framework_css,
        components_dir,
        merge_data,
    ) {
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

#[allow(clippy::too_many_arguments)]
fn build_file(
    inky: &Inky,
    file: &Path,
    input_dir: &Path,
    output_dir: &Path,
    config: &Config,
    inline_css: bool,
    framework_css: bool,
    components_dir: Option<&str>,
    merge_data: Option<&serde_json::Value>,
) -> Result<PathBuf, String> {
    let html = std::fs::read_to_string(file).map_err(|e| format!("Failed to read: {}", e))?;

    // Run validation
    let diagnostics = inky_core::validate::validate(&html, config);
    for d in &diagnostics {
        let label = match d.severity {
            inky_core::validate::Severity::Warning => "warn".yellow().bold(),
            inky_core::validate::Severity::Error => "error".red().bold(),
        };
        eprintln!("  {} {} [{}] {}", label, file.display(), d.rule, d.message);
    }

    let result = crate::build::process_template(
        inky,
        &html,
        inline_css,
        framework_css,
        file.parent(),
        components_dir,
        merge_data,
        crate::build::ErrorMode::Continue,
    );

    let dest = to_output_path(file, input_dir, output_dir);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    std::fs::write(&dest, &result).map_err(|e| format!("Failed to write: {}", e))?;

    Ok(dest)
}

fn find_template_files(dir: &Path) -> Vec<PathBuf> {
    crate::util::find_files(dir, crate::util::TEMPLATE_EXTENSIONS)
}

fn to_output_path(input: &Path, input_dir: &Path, output_dir: &Path) -> PathBuf {
    crate::util::to_output_path(input, input_dir, output_dir)
}

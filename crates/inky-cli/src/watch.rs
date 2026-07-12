use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use colored::Colorize;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};

use inky_core::Config;

pub fn cmd_watch(
    input: PathBuf,
    output: PathBuf,
    build_ctx: crate::build::BuildContext,
    data_path: Option<PathBuf>,
    data_source: crate::builder::DataSource,
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
        column_count: build_ctx.columns,
        output_mode: build_ctx.output_mode,
        bulletproof_buttons: build_ctx.bulletproof_buttons,
        ..Config::default()
    };

    let builder = crate::builder::Builder::new(
        config,
        build_ctx.pipeline_options(),
        build_ctx.plain_text,
    );

    // Initial full build
    eprintln!(
        "  {} {} → {}",
        "watching".cyan().bold(),
        input.display(),
        output.display()
    );

    do_full_build(&input, &output, &builder, &data_source);

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

    // Watch the data path (file or directory) for changes
    if let Some(ref data_file) = data_path {
        let watch_target = if data_file.is_dir() {
            data_file.clone()
        } else {
            data_file
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| data_file.clone())
        };
        let canonical = std::fs::canonicalize(&watch_target).unwrap_or(watch_target);
        eprintln!("  {} {} (data)", "watching".cyan().bold(), data_file.display());
        debouncer
            .watcher()
            .watch(&canonical, notify::RecursiveMode::Recursive)
            .unwrap_or_else(|e| {
                eprintln!(
                    "  {} Failed to watch data path '{}': {}",
                    "warning:".yellow().bold(),
                    canonical.display(),
                    e
                );
            });
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
    let mut data_source = data_source;
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Collect unique changed template files
                let mut changed_files: Vec<PathBuf> = Vec::new();
                let mut needs_full_rebuild = false;
                let mut data_changed = false;

                for event in &events {
                    let path = &event.path;

                    // Check if the data path (file or directory) changed
                    if let Some(ref data_file) = data_path {
                        let canonical_data =
                            std::fs::canonicalize(data_file).unwrap_or(data_file.clone());
                        let canonical_event = std::fs::canonicalize(path).unwrap_or(path.clone());
                        let is_data_event = canonical_event == canonical_data
                            || (canonical_data.is_dir() && canonical_event.starts_with(&canonical_data));
                        if is_data_event {
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

                // Reload data if it changed
                if data_changed {
                    let timestamp = current_time();
                    eprintln!("  [{}] data file changed, reloading...", timestamp);
                    data_source = match data_path.as_deref() {
                        Some(p) if p.is_dir() => {
                            crate::builder::DataSource::Directory(p.to_path_buf())
                        }
                        Some(p) => match crate::util::load_json_data(Some(p)) {
                            Some(v) => crate::builder::DataSource::File(v),
                            None => crate::builder::DataSource::None,
                        },
                        None => crate::builder::DataSource::None,
                    };
                    needs_full_rebuild = true;
                }

                if needs_full_rebuild {
                    let timestamp = current_time();
                    eprintln!(
                        "  [{}] include or file changed, rebuilding all...",
                        timestamp
                    );
                    do_full_build(&input, &output, &builder, &data_source);
                } else {
                    for file in &changed_files {
                        rebuild_single_file(file, &input, &output, &builder, &data_source);
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
    builder: &crate::builder::Builder,
    data_source: &crate::builder::DataSource,
) {
    let files = crate::builder::find_template_files(input, Some(output));

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
        match build_file(builder, file, input, output, data_source) {
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

fn rebuild_single_file(
    file: &Path,
    input_dir: &Path,
    output_dir: &Path,
    builder: &crate::builder::Builder,
    data_source: &crate::builder::DataSource,
) {
    let timestamp = current_time();

    match build_file(builder, file, input_dir, output_dir, data_source) {
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
    builder: &crate::builder::Builder,
    file: &Path,
    input_dir: &Path,
    output_dir: &Path,
    data_source: &crate::builder::DataSource,
) -> Result<PathBuf, String> {
    let built = builder.build_file(file, input_dir, data_source)?;

    for w in &built.warnings {
        eprintln!("  {} {}", "warning:".yellow().bold(), w);
    }
    for d in &built.diagnostics {
        let label = match d.severity {
            inky_core::validate::Severity::Warning => "warn".yellow().bold(),
            inky_core::validate::Severity::Error => "error".red().bold(),
        };
        eprintln!("  {} {} [{}] {}", label, file.display(), d.rule, d.message);
    }

    let dest = to_output_path(file, input_dir, output_dir);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    std::fs::write(&dest, &built.html).map_err(|e| format!("Failed to write: {}", e))?;
    if let Some(ref txt) = built.plain_text {
        let txt_path = dest.with_extension("txt");
        std::fs::write(&txt_path, txt)
            .map_err(|e| format!("Failed to write {}: {}", txt_path.display(), e))?;
    }

    Ok(dest)
}

fn to_output_path(input: &Path, input_dir: &Path, output_dir: &Path) -> PathBuf {
    crate::util::to_output_path(input, input_dir, output_dir)
}

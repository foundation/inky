mod build;
mod config;
mod init;
mod migrate;
mod scss;
mod serve;
pub mod util;
mod watch;

use clap::{Parser, Subcommand};
use colored::Colorize;
use inky_core::validate::{self, Severity};
use inky_core::{Config, Inky, OutputMode};
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(name = "inky")]
#[command(about = "Inky — transform email templates into email-safe HTML")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Transform Inky HTML into email-safe table markup
    Build {
        /// Input file or directory (reads from stdin if omitted)
        input: Option<PathBuf>,

        /// Output file or directory (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Number of columns in the grid (default: 12)
        #[arg(long)]
        columns: Option<u32>,

        /// Skip CSS inlining (inlining is on by default)
        #[arg(long)]
        no_inline_css: bool,

        /// Skip injecting framework CSS (Inky styles are included by default)
        #[arg(long)]
        no_framework_css: bool,

        /// Exit with non-zero code if validation finds any warnings or errors
        #[arg(long)]
        strict: bool,

        /// JSON file or directory for template merge data (file = global, directory = per-template)
        #[arg(long)]
        data: Option<PathBuf>,

        /// Use hybrid output mode (div + MSO ghost tables instead of pure tables)
        #[arg(long)]
        hybrid: bool,

        /// Also generate a plain text version (.txt) alongside each HTML file
        #[arg(long)]
        plain_text: bool,
    },

    /// Validate Inky templates for common issues
    Validate {
        /// Input file or directory
        input: PathBuf,
    },

    /// Migrate v1 Inky syntax to v2
    Migrate {
        /// Input file or directory
        input: PathBuf,

        /// Output file or directory (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Rewrite files in-place
        #[arg(long)]
        in_place: bool,
    },

    /// Scaffold a new Inky email project
    Init {
        /// Project directory name (creates in current directory if omitted)
        name: Option<String>,
    },

    /// Watch for changes and rebuild automatically
    Watch {
        /// Input directory to watch
        input: Option<PathBuf>,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Number of columns in the grid (default: 12)
        #[arg(long)]
        columns: Option<u32>,

        /// Skip CSS inlining
        #[arg(long)]
        no_inline_css: bool,

        /// Skip injecting framework CSS
        #[arg(long)]
        no_framework_css: bool,

        /// JSON file or directory for template merge data (file = global, directory = per-template)
        #[arg(long)]
        data: Option<PathBuf>,

        /// Use hybrid output mode (div + MSO ghost tables instead of pure tables)
        #[arg(long)]
        hybrid: bool,

        /// Also generate a plain text version (.txt) alongside each HTML file
        #[arg(long)]
        plain_text: bool,
    },

    /// Start a live preview dev server
    Serve {
        /// Input directory containing templates
        input: Option<PathBuf>,

        /// Port to serve on (default: 3000)
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Number of columns in the grid (default: 12)
        #[arg(long)]
        columns: Option<u32>,

        /// Skip CSS inlining
        #[arg(long)]
        no_inline_css: bool,

        /// Skip injecting framework CSS
        #[arg(long)]
        no_framework_css: bool,

        /// JSON file or directory for template merge data (file = global, directory = per-template)
        #[arg(long)]
        data: Option<PathBuf>,

        /// Use hybrid output mode (div + MSO ghost tables instead of pure tables)
        #[arg(long)]
        hybrid: bool,
    },

    /// Check templates for common spam triggers
    SpamCheck {
        /// Input file or directory
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            input,
            output,
            columns,
            no_inline_css,
            no_framework_css,
            strict,
            data,
            hybrid,
            plain_text,
        } => {
            let (input, output, columns, components, cfg_data, cfg_hybrid, cfg_plain_text) =
                resolve_config(input, output, columns);
            let data_path = data.or(cfg_data);
            let data_source = resolve_data_source(data_path.as_deref());
            let output_mode = if hybrid || cfg_hybrid {
                OutputMode::Hybrid
            } else {
                OutputMode::Table
            };
            let plain_text = plain_text || cfg_plain_text;
            cmd_build(
                input,
                output,
                columns,
                !no_inline_css,
                !no_framework_css,
                strict,
                components,
                &data_source,
                output_mode,
                plain_text,
            )
        }
        Commands::Validate { input } => cmd_validate(input),
        Commands::Migrate {
            input,
            output,
            in_place,
        } => migrate::cmd_migrate(input, output, in_place),
        Commands::Init { name } => init::cmd_init(name),
        Commands::Watch {
            input,
            output,
            columns,
            no_inline_css,
            no_framework_css,
            data,
            hybrid,
            plain_text,
        } => {
            let (input, output, columns, components, cfg_data, cfg_hybrid, cfg_plain_text) =
                resolve_config(input, output, columns);
            let data = data.or(cfg_data);
            let output_mode = if hybrid || cfg_hybrid {
                OutputMode::Hybrid
            } else {
                OutputMode::Table
            };
            let _plain_text = plain_text || cfg_plain_text;
            let input = input.unwrap_or_else(|| {
                eprintln!("{} No input directory specified. Use `inky watch <dir>` or set \"src\" in inky.config.json", "error:".red().bold());
                process::exit(1);
            });
            let output = output.unwrap_or_else(|| {
                eprintln!("{} No output directory specified. Use `-o <dir>` or set \"dist\" in inky.config.json", "error:".red().bold());
                process::exit(1);
            });
            watch::cmd_watch(
                input,
                output,
                columns,
                !no_inline_css,
                !no_framework_css,
                components,
                data,
                output_mode,
                _plain_text,
            )
        }
        Commands::Serve {
            input,
            port,
            columns,
            no_inline_css,
            no_framework_css,
            data,
            hybrid,
        } => {
            let (input, _output, columns, components, cfg_data, cfg_hybrid, _cfg_plain_text) =
                resolve_config(input, None, columns);
            let data = data.or(cfg_data);
            let output_mode = if hybrid || cfg_hybrid {
                OutputMode::Hybrid
            } else {
                OutputMode::Table
            };
            let input = input.unwrap_or_else(|| {
                eprintln!("{} No input directory specified. Use `inky serve <dir>` or set \"src\" in inky.config.json", "error:".red().bold());
                process::exit(1);
            });
            serve::cmd_serve(
                input,
                columns,
                !no_inline_css,
                !no_framework_css,
                components,
                data,
                port,
                output_mode,
            )
        }
        Commands::SpamCheck { input } => cmd_spam_check(input),
    }
}

/// Find the project config by checking (in order):
/// 1. The input path itself (if it's a directory with inky.config.json)
/// 2. Searching upward from the input path
/// 3. Searching upward from cwd
fn find_project_config(input: Option<&Path>) -> Option<(config::ProjectConfig, PathBuf)> {
    // If an input path was given, check it and its ancestors first
    if let Some(path) = input {
        let dir = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };
        if let Some(cfg) = config::load_config(&dir) {
            return Some(cfg);
        }
    }

    // Fall back to searching from cwd
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    config::load_config(&cwd)
}

/// Resolve CLI arguments with fallbacks from inky.config.json.
/// CLI flags take priority over config file values.
///
/// When the input points to a project directory (containing inky.config.json),
/// the config's `src` and `dist` are resolved relative to that directory.
#[allow(clippy::type_complexity)]
fn resolve_config(
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    columns: Option<u32>,
) -> (
    Option<PathBuf>,
    Option<PathBuf>,
    u32,
    Option<String>,
    Option<PathBuf>,
    bool,
    bool,
) {
    let project_config = find_project_config(input.as_deref());

    let (cfg_src, cfg_dist, cfg_columns, cfg_components, cfg_data, cfg_hybrid, cfg_plain_text) =
        match &project_config {
            Some((cfg, base_dir)) => (
                cfg.src.as_ref().map(|s| base_dir.join(s)),
                cfg.dist.as_ref().map(|d| base_dir.join(d)),
                cfg.columns,
                cfg.components.as_ref().map(|c| {
                    let p = base_dir.join(c);
                    std::fs::canonicalize(&p)
                        .unwrap_or(p)
                        .to_string_lossy()
                        .to_string()
                }),
                cfg.data.as_ref().map(|d| base_dir.join(d)),
                cfg.hybrid.unwrap_or(false),
                cfg.plain_text.unwrap_or(false),
            ),
            None => (None, None, None, None, None, false, false),
        };

    // If input points to a project root (has config), use config's src
    // If input points to a specific file/dir, use it directly
    let input = if let (Some(ref path), Some((_, ref base_dir))) = (&input, &project_config) {
        if path == base_dir.as_path() {
            // User passed the project root — use config's src
            cfg_src
        } else {
            // User passed a specific path — use it directly
            input
        }
    } else {
        input.or(cfg_src)
    };

    let output = output.or(cfg_dist);
    let columns = columns.or(cfg_columns).unwrap_or(12);

    (
        input,
        output,
        columns,
        cfg_components,
        cfg_data,
        cfg_hybrid,
        cfg_plain_text,
    )
}

/// Resolved data source for template merging.
enum DataSource {
    /// No data — merge tags pass through untouched.
    None,
    /// Single JSON file applied to all templates.
    File(serde_json::Value),
    /// Directory of per-template JSON files (e.g., data/welcome.json for welcome.inky).
    Directory(PathBuf),
}

/// Detect whether a data path is a file or directory and return the appropriate source.
fn resolve_data_source(path: Option<&Path>) -> DataSource {
    let Some(path) = path else {
        return DataSource::None;
    };
    if path.is_dir() {
        DataSource::Directory(path.to_path_buf())
    } else if path.is_file() {
        match load_merge_data(Some(path)) {
            Some(data) => DataSource::File(data),
            std::option::Option::None => DataSource::None,
        }
    } else {
        eprintln!(
            "{} Data path '{}' does not exist",
            "error:".red().bold(),
            path.display()
        );
        process::exit(1);
    }
}

/// Load and parse a JSON data file for template merging.
fn load_merge_data(path: Option<&Path>) -> Option<serde_json::Value> {
    let path = path?;
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to read data file {}: {}",
            "error:".red().bold(),
            path.display(),
            e
        );
        process::exit(1);
    });
    let data: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to parse data file {}: {}",
            "error:".red().bold(),
            path.display(),
            e
        );
        process::exit(1);
    });
    Some(data)
}

#[allow(clippy::too_many_arguments)]
fn cmd_build(
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    columns: u32,
    inline_css: bool,
    framework_css: bool,
    strict: bool,
    components_dir: Option<String>,
    data_source: &DataSource,
    output_mode: OutputMode,
    plain_text: bool,
) {
    let config = Config {
        column_count: columns,
        output_mode,
        ..Config::default()
    };
    let inky = Inky::with_config(config.clone());

    let has_warnings = match input {
        Some(path) => {
            if path.is_dir() {
                build_directory(
                    &inky,
                    &path,
                    output.as_deref(),
                    inline_css,
                    framework_css,
                    &config,
                    components_dir.as_deref(),
                    data_source,
                    plain_text,
                )
            } else {
                let base = path.parent().map(Path::to_path_buf);
                let html = read_file(&path);
                let file_data = resolve_data_for_file(
                    &path,
                    path.parent().unwrap_or(Path::new(".")),
                    data_source,
                );
                let result = build::process_template(
                    &inky,
                    &html,
                    inline_css,
                    framework_css,
                    base.as_deref(),
                    components_dir.as_deref(),
                    file_data.as_ref(),
                    build::ErrorMode::Exit,
                );
                let warnings = print_validation_warnings(&html, &result, &config, &path);
                // If no output specified and input is .inky, write to .html
                let out = output.clone().or_else(|| {
                    if path.extension().and_then(OsStr::to_str) == Some("inky") {
                        Some(path.with_extension("html"))
                    } else {
                        None
                    }
                });
                write_output(&result, out.as_deref());
                if plain_text {
                    if let Some(ref out_path) = out {
                        let txt = inky_core::plaintext::html_to_plain_text(&result);
                        let txt_path = out_path.with_extension("txt");
                        write_output(&txt, Some(&txt_path));
                    }
                }
                warnings
            }
        }
        None => {
            // Read from stdin — use cwd as base for resolving CSS files
            let mut html = String::new();
            io::stdin().read_to_string(&mut html).unwrap_or_else(|e| {
                eprintln!("{} Failed to read stdin: {}", "error:".red().bold(), e);
                process::exit(1);
            });
            let cwd = std::env::current_dir().ok();
            let global_data = match data_source {
                DataSource::File(ref d) => Some(d),
                _ => None,
            };
            let result = build::process_template(
                &inky,
                &html,
                inline_css,
                framework_css,
                cwd.as_deref(),
                components_dir.as_deref(),
                global_data,
                build::ErrorMode::Exit,
            );
            let warnings = print_validation_warnings(&html, &result, &config, Path::new("stdin"));
            write_output(&result, output.as_deref());
            warnings
        }
    };

    if strict && has_warnings {
        process::exit(1);
    }
}

/// Run validation on source and output HTML, print any warnings to stderr.
/// Returns true if any diagnostics were found.
fn print_validation_warnings(
    source_html: &str,
    output_html: &str,
    config: &Config,
    path: &Path,
) -> bool {
    let mut diagnostics = validate::validate_source(source_html, config);
    diagnostics.extend(validate::validate_output(output_html));
    for d in &diagnostics {
        let label = match d.severity {
            Severity::Warning => "warn".yellow().bold(),
            Severity::Error => "error".red().bold(),
        };
        eprintln!("  {} {} [{}] {}", label, path.display(), d.rule, d.message);
    }
    !diagnostics.is_empty()
}

#[allow(clippy::too_many_arguments)]
fn build_directory(
    inky: &Inky,
    input_dir: &Path,
    output_dir: Option<&Path>,
    inline_css: bool,
    framework_css: bool,
    config: &Config,
    components_dir: Option<&str>,
    data_source: &DataSource,
    plain_text: bool,
) -> bool {
    let files = find_template_files(input_dir);
    let mut has_warnings = false;

    if files.is_empty() {
        eprintln!(
            "{} No .inky or .html files found in {}",
            "warning:".yellow().bold(),
            input_dir.display()
        );
        return false;
    }

    for file in &files {
        let html = read_file(file);
        let file_data = resolve_data_for_file(file, input_dir, data_source);
        let base = file.parent().map(Path::to_path_buf);
        let result = build::process_template(
            inky,
            &html,
            inline_css,
            framework_css,
            base.as_deref(),
            components_dir,
            file_data.as_ref(),
            build::ErrorMode::Exit,
        );

        let out_path = match output_dir {
            Some(dir) => {
                let dest = to_output_path(file, input_dir, dir);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).unwrap_or_else(|e| {
                        eprintln!(
                            "{} Failed to create directory {}: {}",
                            "error:".red().bold(),
                            parent.display(),
                            e
                        );
                        process::exit(1);
                    });
                }
                Some(dest)
            }
            None => None,
        };

        match out_path {
            Some(dest) => {
                fs::write(&dest, &result).unwrap_or_else(|e| {
                    eprintln!(
                        "{} Failed to write {}: {}",
                        "error:".red().bold(),
                        dest.display(),
                        e
                    );
                    process::exit(1);
                });
                if print_validation_warnings(&html, &result, config, &dest) {
                    has_warnings = true;
                }
                eprintln!(
                    "  {} {} → {}",
                    "built".green().bold(),
                    file.display(),
                    dest.display()
                );
                if plain_text {
                    let txt = inky_core::plaintext::html_to_plain_text(&result);
                    let txt_path = dest.with_extension("txt");
                    fs::write(&txt_path, &txt).unwrap_or_else(|e| {
                        eprintln!(
                            "{} Failed to write {}: {}",
                            "error:".red().bold(),
                            txt_path.display(),
                            e
                        );
                    });
                }
            }
            None => {
                println!("<!-- {} -->\n{}\n", file.display(), result);
            }
        }
    }

    eprintln!(
        "\n{} Transformed {} file(s)",
        "done".green().bold(),
        files.len()
    );

    has_warnings
}

fn cmd_validate(input: PathBuf) {
    let (resolved_input, _output, columns, components, cfg_data, cfg_hybrid, _cfg_plain_text) =
        resolve_config(Some(input.clone()), None, None);
    let data_source = resolve_data_source(cfg_data.as_deref());
    let output_mode = if cfg_hybrid {
        OutputMode::Hybrid
    } else {
        OutputMode::Table
    };
    let config = Config {
        column_count: columns,
        output_mode,
        ..Config::default()
    };
    let inky = Inky::with_config(config.clone());
    let input_path = resolved_input.unwrap_or(input);

    let files = if input_path.is_dir() {
        find_template_files(&input_path)
    } else {
        vec![input_path.clone()]
    };

    if files.is_empty() {
        eprintln!(
            "{} No .inky or .html files found in {}",
            "warning:".yellow().bold(),
            input_path.display()
        );
        return;
    }

    let input_dir = if input_path.is_dir() {
        &input_path
    } else {
        input_path.parent().unwrap_or(Path::new("."))
    };

    let mut has_errors = false;
    for file in &files {
        let source_html = read_file(file);
        let file_data = resolve_data_for_file(file, input_dir, &data_source);
        let base = file.parent().map(Path::to_path_buf);
        let output_html = build::process_template(
            &inky,
            &source_html,
            true,
            true,
            base.as_deref(),
            components.as_deref(),
            file_data.as_ref(),
            build::ErrorMode::Continue,
        );
        if print_validation_warnings(&source_html, &output_html, &config, file) {
            has_errors = true;
        } else {
            eprintln!("  {} {}", "ok".green().bold(), file.display());
        }
    }

    eprintln!("\n  Validated {} file(s)", files.len());

    if has_errors {
        process::exit(1);
    }
}

fn find_template_files(dir: &Path) -> Vec<PathBuf> {
    util::find_files(dir, util::TEMPLATE_EXTENSIONS)
}

fn to_output_path(input: &Path, input_dir: &Path, output_dir: &Path) -> PathBuf {
    util::to_output_path(input, input_dir, output_dir)
}

fn read_file(path: &std::path::Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!(
            "{} Failed to read {}: {}",
            "error:".red().bold(),
            path.display(),
            e
        );
        process::exit(1);
    })
}

/// Resolve merge data for a specific template file based on the data source.
fn resolve_data_for_file(
    file: &Path,
    input_dir: &Path,
    data_source: &DataSource,
) -> Option<serde_json::Value> {
    match data_source {
        DataSource::None => None,
        DataSource::File(data) => Some(data.clone()),
        DataSource::Directory(dir) => {
            let relative = file.strip_prefix(input_dir).ok()?;
            let json_path = dir.join(relative).with_extension("json");
            if json_path.is_file() {
                let content = fs::read_to_string(&json_path).ok()?;
                serde_json::from_str(&content).ok()
            } else {
                None
            }
        }
    }
}

fn cmd_spam_check(input: PathBuf) {
    let config = Config::default();
    let inky = Inky::with_config(config.clone());
    let mut has_issues = false;

    let files = if input.is_dir() {
        find_template_files(&input)
    } else {
        vec![input]
    };

    if files.is_empty() {
        eprintln!("{} No template files found", "warning:".yellow().bold());
        return;
    }

    for file in &files {
        let html = read_file(file);
        let base = file.parent().map(Path::to_path_buf);
        let result = build::process_template(
            &inky,
            &html,
            true,
            true,
            base.as_deref(),
            None,
            None,
            build::ErrorMode::Continue,
        );
        let diagnostics = validate::validate_spam(&result);

        if diagnostics.is_empty() {
            eprintln!("  {} {}", "ok".green().bold(), file.display());
        } else {
            has_issues = true;
            for d in &diagnostics {
                let label = match d.severity {
                    Severity::Warning => "warn".yellow().bold(),
                    Severity::Error => "error".red().bold(),
                };
                eprintln!("  {} {} [{}] {}", label, file.display(), d.rule, d.message);
            }
        }
    }

    if has_issues {
        process::exit(1);
    }
}

fn write_output(content: &str, path: Option<&std::path::Path>) {
    match path {
        Some(p) => {
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|e| {
                    eprintln!(
                        "{} Failed to create directory {}: {}",
                        "error:".red().bold(),
                        parent.display(),
                        e
                    );
                    process::exit(1);
                });
            }
            fs::write(p, content).unwrap_or_else(|e| {
                eprintln!(
                    "{} Failed to write {}: {}",
                    "error:".red().bold(),
                    p.display(),
                    e
                );
                process::exit(1);
            });
        }
        None => print!("{}", content),
    }
}

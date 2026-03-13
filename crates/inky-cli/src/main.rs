mod build;
mod config;
mod init;
mod migrate;
mod scss;
pub mod util;
mod watch;

use clap::{Parser, Subcommand};
use colored::Colorize;
use inky_core::validate::{self, Severity};
use inky_core::{Config, Inky};
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
        } => {
            let (input, output, columns) = resolve_config(input, output, columns);
            cmd_build(
                input,
                output,
                columns,
                !no_inline_css,
                !no_framework_css,
                strict,
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
        } => {
            let (input, output, columns) = resolve_config(input, output, columns);
            let input = input.unwrap_or_else(|| {
                eprintln!("{} No input directory specified. Use `inky watch <dir>` or set \"src\" in inky.config.json", "error:".red().bold());
                process::exit(1);
            });
            let output = output.unwrap_or_else(|| {
                eprintln!("{} No output directory specified. Use `-o <dir>` or set \"dist\" in inky.config.json", "error:".red().bold());
                process::exit(1);
            });
            watch::cmd_watch(input, output, columns, !no_inline_css, !no_framework_css)
        }
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
fn resolve_config(
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    columns: Option<u32>,
) -> (Option<PathBuf>, Option<PathBuf>, u32) {
    let project_config = find_project_config(input.as_deref());

    let (cfg_src, cfg_dist, cfg_columns) = match &project_config {
        Some((cfg, base_dir)) => (
            cfg.src.as_ref().map(|s| base_dir.join(s)),
            cfg.dist.as_ref().map(|d| base_dir.join(d)),
            cfg.columns,
        ),
        None => (None, None, None),
    };

    // If input points to a project root (has config), use config's src
    // If input points to a specific file/dir, use it directly
    let input = if input.is_some() && project_config.is_some() {
        let path = input.as_ref().unwrap();
        // Check if the input path is the project root (where config was found)
        let (_, base_dir) = project_config.as_ref().unwrap();
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

    (input, output, columns)
}

fn cmd_build(
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    columns: u32,
    inline_css: bool,
    framework_css: bool,
    strict: bool,
) {
    let config = Config {
        column_count: columns,
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
                )
            } else {
                let base = path.parent().map(Path::to_path_buf);
                let html = read_file(&path);
                let warnings = print_validation_warnings(&html, &config, &path);
                let result = build::process_template(
                    &inky,
                    &html,
                    inline_css,
                    framework_css,
                    base.as_deref(),
                    build::ErrorMode::Exit,
                );
                // If no output specified and input is .inky, write to .html
                let out = output.clone().or_else(|| {
                    if path.extension().and_then(OsStr::to_str) == Some("inky") {
                        Some(path.with_extension("html"))
                    } else {
                        None
                    }
                });
                write_output(&result, out.as_deref());
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
            let warnings = print_validation_warnings(&html, &config, Path::new("stdin"));
            let cwd = std::env::current_dir().ok();
            let result = build::process_template(
                &inky,
                &html,
                inline_css,
                framework_css,
                cwd.as_deref(),
                build::ErrorMode::Exit,
            );
            write_output(&result, output.as_deref());
            warnings
        }
    };

    if strict && has_warnings {
        process::exit(1);
    }
}

/// Run validation and print any warnings to stderr. Returns true if any diagnostics were found.
fn print_validation_warnings(html: &str, config: &Config, path: &Path) -> bool {
    let diagnostics = validate::validate(html, config);
    for d in &diagnostics {
        let label = match d.severity {
            Severity::Warning => "warn".yellow().bold(),
            Severity::Error => "error".red().bold(),
        };
        eprintln!("  {} {} [{}] {}", label, path.display(), d.rule, d.message);
    }
    !diagnostics.is_empty()
}

fn build_directory(
    inky: &Inky,
    input_dir: &Path,
    output_dir: Option<&Path>,
    inline_css: bool,
    framework_css: bool,
    config: &Config,
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
        if print_validation_warnings(&html, config, file) {
            has_warnings = true;
        }
        let base = file.parent().map(Path::to_path_buf);
        let result = build::process_template(
            inky,
            &html,
            inline_css,
            framework_css,
            base.as_deref(),
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
                eprintln!(
                    "  {} {} → {}",
                    "built".green().bold(),
                    file.display(),
                    dest.display()
                );
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
    let config = Config::default();
    let mut has_errors = false;

    if input.is_dir() {
        let files = find_template_files(&input);

        if files.is_empty() {
            eprintln!(
                "{} No .inky or .html files found in {}",
                "warning:".yellow().bold(),
                input.display()
            );
            return;
        }

        for file in &files {
            if validate_file(file, &config) {
                has_errors = true;
            }
        }

        let total = files.len();
        eprintln!("\n  Validated {} file(s)", total);
    } else {
        has_errors = validate_file(&input, &config);
    }

    if has_errors {
        process::exit(1);
    }
}

fn validate_file(path: &std::path::Path, config: &Config) -> bool {
    let html = read_file(path);
    let diagnostics = validate::validate(&html, config);

    if diagnostics.is_empty() {
        eprintln!("  {} {}", "ok".green().bold(), path.display());
        return false;
    }

    for d in &diagnostics {
        let label = match d.severity {
            Severity::Warning => "warn".yellow().bold(),
            Severity::Error => "error".red().bold(),
        };
        eprintln!("  {} {} [{}] {}", label, path.display(), d.rule, d.message);
    }

    true
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

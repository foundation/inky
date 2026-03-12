use clap::{Parser, Subcommand};
use colored::Colorize;
use inky_core::validate::{self, Severity};
use inky_core::{Config, Inky};
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

const INKY_EXTENSIONS: &[&str] = &["inky", "html"];

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
        #[arg(long, default_value = "12")]
        columns: u32,

        /// Skip CSS inlining (inlining is on by default)
        #[arg(long)]
        no_inline_css: bool,

        /// Exit with non-zero code if validation finds any warnings or errors
        #[arg(long)]
        strict: bool,
    },

    /// Validate Inky templates for common issues
    Validate {
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
            strict,
        } => cmd_build(input, output, columns, !no_inline_css, strict),
        Commands::Validate { input } => cmd_validate(input),
    }
}

fn cmd_build(input: Option<PathBuf>, output: Option<PathBuf>, columns: u32, inline_css: bool, strict: bool) {
    let config = Config {
        column_count: columns,
        ..Config::default()
    };
    let inky = Inky::with_config(config.clone());

    let has_warnings = match input {
        Some(path) => {
            if path.is_dir() {
                build_directory(&inky, &path, output.as_deref(), inline_css, &config)
            } else {
                let base = path.parent().map(Path::to_path_buf);
                let html = read_file(&path);
                let warnings = print_validation_warnings(&html, &config, &path);
                let result = process_html(&inky, &html, inline_css, base.as_deref());
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
            io::stdin()
                .read_to_string(&mut html)
                .unwrap_or_else(|e| {
                    eprintln!("{} Failed to read stdin: {}", "error:".red().bold(), e);
                    process::exit(1);
                });
            let warnings = print_validation_warnings(&html, &config, Path::new("stdin"));
            let cwd = std::env::current_dir().ok();
            let result = process_html(&inky, &html, inline_css, cwd.as_deref());
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

fn process_html(inky: &Inky, html: &str, inline_css: bool, base_path: Option<&Path>) -> String {
    if inline_css {
        inky.transform_and_inline(html, base_path)
            .unwrap_or_else(|e| {
                eprintln!("{} CSS inlining failed: {}", "error:".red().bold(), e);
                process::exit(1);
            })
    } else {
        inky.transform(html)
    }
}

fn build_directory(inky: &Inky, input_dir: &Path, output_dir: Option<&Path>, inline_css: bool, config: &Config) -> bool {
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
        let result = process_html(inky, &html, inline_css, base.as_deref());

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

/// Find all template files (.inky and .html) in a directory.
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

/// Convert an input path to an output path, changing .inky to .html.
fn to_output_path(input: &Path, input_dir: &Path, output_dir: &Path) -> PathBuf {
    let relative = input.strip_prefix(input_dir).unwrap();
    let dest = output_dir.join(relative);
    if dest.extension().and_then(OsStr::to_str) == Some("inky") {
        dest.with_extension("html")
    } else {
        dest
    }
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

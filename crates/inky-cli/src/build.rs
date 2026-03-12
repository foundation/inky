use colored::Colorize;
use regex::Regex;
use std::path::Path;

use inky_core::Inky;

use crate::scss;

/// How to handle errors during the build pipeline.
#[derive(Clone, Copy)]
pub enum ErrorMode {
    /// Exit the process on error (for `inky build`)
    Exit,
    /// Log the error and continue with empty/fallback output (for `inky watch`)
    Continue,
}

fn handle_error(mode: ErrorMode, msg: &str) -> String {
    eprintln!("{} {}", "error:".red().bold(), msg);
    match mode {
        ErrorMode::Exit => std::process::exit(1),
        ErrorMode::Continue => String::new(),
    }
}

/// Full build pipeline: layout → includes → extract SCSS overrides → compile framework CSS → inject → transform → inline → cleanup.
pub fn process_template(
    inky: &Inky,
    html: &str,
    inline_css: bool,
    framework_css: bool,
    base_path: Option<&Path>,
    error_mode: ErrorMode,
) -> String {
    // Resolve <layout> tag, then <include> tags before any other processing
    let mut html = if let Some(base) = base_path {
        let with_layout = inky_core::include::process_layout(html, base)
            .unwrap_or_else(|e| handle_error(error_mode, &e));
        inky_core::include::process_includes(&with_layout, base)
            .unwrap_or_else(|e| handle_error(error_mode, &e))
    } else {
        html.to_string()
    };

    if framework_css {
        let (cleaned, overrides) = scss::extract_scss_overrides(&html, base_path);
        html = cleaned;

        let css = scss::compile_framework_scss(&overrides).unwrap_or_else(|e| {
            handle_error(error_mode, &format!("SCSS compilation failed: {}", e))
        });

        html = scss::inject_css_into_html(&html, &css);
    } else {
        let (cleaned, _) = scss::extract_scss_overrides(&html, base_path);
        html = cleaned;
    }

    let result = if inline_css {
        inky.transform_and_inline(&html, base_path)
            .unwrap_or_else(|e| {
                eprintln!("{} CSS inlining failed: {}", "error:".red().bold(), e);
                match error_mode {
                    ErrorMode::Exit => std::process::exit(1),
                    ErrorMode::Continue => html.clone(),
                }
            })
    } else {
        inky.transform(&html)
    };

    collapse_blank_lines(&result)
}

/// Remove consecutive blank lines, preserving content inside <pre> blocks.
fn collapse_blank_lines(html: &str) -> String {
    let pre_re = Regex::new(r"(?si)<pre[^>]*>.*?</pre>").unwrap();
    let mut result = String::with_capacity(html.len());
    let mut last_end = 0;

    for m in pre_re.find_iter(html) {
        result.push_str(&do_collapse(&html[last_end..m.start()]));
        result.push_str(m.as_str());
        last_end = m.end();
    }

    result.push_str(&do_collapse(&html[last_end..]));
    result
}

fn do_collapse(s: &str) -> String {
    let blank_re = Regex::new(r"\n\s*\n(\s*\n)*").unwrap();
    blank_re.replace_all(s, "\n").to_string()
}

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

/// Full build pipeline: layout → custom components → includes → extract SCSS overrides → compile framework CSS → inject → transform → inline → cleanup.
pub fn process_template(
    inky: &Inky,
    html: &str,
    inline_css: bool,
    framework_css: bool,
    base_path: Option<&Path>,
    components_dir: Option<&str>,
    error_mode: ErrorMode,
) -> String {
    // Resolve <layout> tag, then custom components, then <include> tags
    let mut html = if let Some(base) = base_path {
        let with_layout = inky_core::include::process_layout(html, base)
            .unwrap_or_else(|e| handle_error(error_mode, &e));
        let with_components = inky_core::include::process_custom_components(
            &with_layout,
            base,
            components_dir.unwrap_or("components"),
        )
        .unwrap_or_else(|e| handle_error(error_mode, &e));
        inky_core::include::process_includes(&with_components, base)
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

        // Inject color-scheme meta tags for dark mode support
        html = inject_color_scheme_meta(&html);
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

/// Inject `<meta name="color-scheme">` and `<meta name="supported-color-schemes">`
/// into `<head>` if dark mode styles are present and the meta tags aren't already there.
fn inject_color_scheme_meta(html: &str) -> String {
    // Only inject if dark mode styles exist in the output
    if !html.contains("prefers-color-scheme") {
        return html.to_string();
    }

    // Don't inject if the user already has them
    if html.contains("color-scheme") {
        return html.to_string();
    }

    let meta_tags = r#"<meta name="color-scheme" content="light dark">
<meta name="supported-color-schemes" content="light dark">"#;

    // Insert after opening <head> tag
    let head_re = Regex::new(r"(?i)<head[^>]*>").unwrap();
    if let Some(m) = head_re.find(html) {
        let mut result = String::with_capacity(html.len() + meta_tags.len() + 2);
        result.push_str(&html[..m.end()]);
        result.push('\n');
        result.push_str(meta_tags);
        result.push_str(&html[m.end()..]);
        return result;
    }

    html.to_string()
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

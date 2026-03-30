use std::path::Path;
use std::sync::LazyLock;

use colored::Colorize;
use regex::Regex;

static RE_TABLE_TAGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(</?(table|tbody|tr|td|th)[\s>])").unwrap());
static RE_CLOSING_TAGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(^</(?:table|tbody|tr|td|th)>\n){2,}").unwrap());
static RE_LEADING_WS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^\s+<").unwrap());
static RE_HEAD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<head[^>]*>").unwrap());
static RE_PRE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?si)<pre[^>]*>.*?</pre>").unwrap());
static RE_BLANK_LINES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n\s*\n(\s*\n)*").unwrap());

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

/// Common build parameters shared across build, watch, and serve commands.
#[derive(Clone)]
pub struct BuildContext {
    pub inline_css: bool,
    pub framework_css: bool,
    pub components_dir: Option<String>,
    pub error_mode: ErrorMode,
    pub output_mode: inky_core::OutputMode,
    pub columns: u32,
    pub bulletproof_buttons: bool,
    pub plain_text: bool,
    pub json: bool,
}

fn handle_error(mode: ErrorMode, msg: &str) -> String {
    eprintln!("{} {}", "error:".red().bold(), msg);
    match mode {
        ErrorMode::Exit => std::process::exit(1),
        ErrorMode::Continue => String::new(),
    }
}

/// Full build pipeline: layout → custom components → includes → merge data → extract SCSS overrides → compile framework CSS → inject → transform → inline → cleanup.
pub fn process_template(
    inky: &Inky,
    html: &str,
    ctx: &BuildContext,
    base_path: Option<&Path>,
    merge_data: Option<&serde_json::Value>,
) -> String {
    // Resolve <layout> tag, then custom components, then <include> tags
    let mut html = if let Some(base) = base_path {
        let with_layout = inky_core::include::process_layout(html, base)
            .unwrap_or_else(|e| handle_error(ctx.error_mode, &e));
        let with_components = inky_core::include::process_custom_components(
            &with_layout,
            base,
            ctx.components_dir.as_deref().unwrap_or("components"),
        )
        .unwrap_or_else(|e| handle_error(ctx.error_mode, &e));
        inky_core::include::process_includes(&with_components, base)
            .unwrap_or_else(|e| handle_error(ctx.error_mode, &e))
    } else {
        html.to_string()
    };

    // MiniJinja template merge (after includes, before transform)
    if let Some(data) = merge_data {
        html = inky_core::templating::render_template(&html, data, false).unwrap_or_else(|e| {
            handle_error(ctx.error_mode, &format!("Template merge failed: {}", e))
        });
    }

    if ctx.framework_css {
        let (cleaned, overrides) = scss::extract_scss_overrides(&html, base_path);
        html = cleaned;

        let css = scss::compile_framework_scss(&overrides).unwrap_or_else(|e| {
            handle_error(ctx.error_mode, &format!("SCSS compilation failed: {}", e))
        });

        html = scss::inject_css_into_html(&html, &css);

        // Inject color-scheme meta tags for dark mode support
        html = inject_color_scheme_meta(&html);
    } else {
        let (cleaned, _) = scss::extract_scss_overrides(&html, base_path);
        html = cleaned;
    }

    let result = if ctx.inline_css {
        inky.transform_and_inline(&html, base_path)
            .unwrap_or_else(|e| {
                eprintln!("{} CSS inlining failed: {}", "error:".red().bold(), e);
                match ctx.error_mode {
                    ErrorMode::Exit => std::process::exit(1),
                    ErrorMode::Continue => html.clone(),
                }
            })
    } else {
        inky.transform(&html)
    };

    let result = break_long_lines(&result);
    let result = strip_leading_whitespace(&result);
    let result = collapse_closing_tags(&result);
    collapse_blank_lines(&result)
}

/// Insert newlines before and after table structure tags to prevent lines
/// exceeding RFC 2822's 998-character limit. Whitespace between table
/// elements (<table>, <tbody>, <tr>, <td>, <th>) is ignored by email clients,
/// so this is safe and does not affect rendering.
fn break_long_lines(html: &str) -> String {
    RE_TABLE_TAGS
        .replace_all(html, |caps: &regex::Captures| {
            let tag = &caps[0];
            if tag.starts_with("</") {
                // Closing tag: newline before it
                format!("\n{}", tag)
            } else {
                // Opening tag: newline before it
                format!("\n{}", tag)
            }
        })
        .to_string()
}

/// Collapse consecutive lines that contain only closing table tags into a single line.
/// e.g., `</th>\n</tr>\n</tbody>\n</table>\n` becomes `</th></tr></tbody></table>\n`
fn collapse_closing_tags(html: &str) -> String {
    RE_CLOSING_TAGS
        .replace_all(html, |caps: &regex::Captures| {
            let s = &caps[0];
            // Join all closing tags, keep one trailing newline
            let joined: String = s.lines().collect::<Vec<_>>().join("");
            format!("{}\n", joined)
        })
        .to_string()
}

/// Strip leading whitespace from lines that start with an HTML tag.
fn strip_leading_whitespace(html: &str) -> String {
    RE_LEADING_WS.replace_all(html, "<").to_string()
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
    if let Some(m) = RE_HEAD.find(html) {
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
    let mut result = String::with_capacity(html.len());
    let mut last_end = 0;

    for m in RE_PRE.find_iter(html) {
        result.push_str(&do_collapse(&html[last_end..m.start()]));
        result.push_str(m.as_str());
        last_end = m.end();
    }

    result.push_str(&do_collapse(&html[last_end..]));
    result
}

fn do_collapse(s: &str) -> String {
    RE_BLANK_LINES.replace_all(s, "\n").to_string()
}

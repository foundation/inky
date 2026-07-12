//! The full template-processing pipeline: layout/includes → data merge →
//! SCSS compile/inject → component transform (+ CSS inlining) → output cleanup.
//!
//! Library-safe: no printing, no process exits. Non-fatal notes are returned
//! in [`Processed::warnings`].

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::{scss, Config, Inky};

pub struct PipelineOptions {
    pub inline_css: bool,
    pub framework_css: bool,
    pub components_dir: String,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            inline_css: true,
            framework_css: true,
            components_dir: "components".to_string(),
        }
    }
}

pub struct Processed {
    pub html: String,
    pub warnings: Vec<String>,
}

pub struct Pipeline {
    inky: Inky,
    config: Config,
    options: PipelineOptions,
}

impl Pipeline {
    pub fn new(config: Config, options: PipelineOptions) -> Self {
        Self {
            inky: Inky::with_config(config.clone()),
            config,
            options,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Run the full pipeline on one template.
    ///
    /// `base_path` is the directory used to resolve layouts, includes,
    /// custom components, linked SCSS, and linked CSS. `None` disables
    /// filesystem resolution (stdin-style input).
    pub fn process(
        &self,
        html: &str,
        base_path: Option<&Path>,
        data: Option<&serde_json::Value>,
    ) -> Result<Processed, String> {
        let mut warnings = Vec::new();

        // Layout → custom components → includes
        let mut html = if let Some(base) = base_path {
            let with_layout = crate::include::process_layout(html, base)?;
            let with_components = crate::include::process_custom_components(
                &with_layout,
                base,
                &self.options.components_dir,
            )?;
            crate::include::process_includes(&with_components, base)?
        } else {
            html.to_string()
        };

        // MiniJinja template merge (after includes, before transform)
        if let Some(data) = data {
            html = crate::templating::render_template(&html, data, false)
                .map_err(|e| format!("Template merge failed: {}", e))?;
        }

        if self.options.framework_css {
            let (cleaned, user_scss, scss_warnings) =
                scss::extract_scss_sources(&html, base_path);
            warnings.extend(scss_warnings);
            html = cleaned;

            let css = scss::compile_framework_scss(&user_scss)
                .map_err(|e| format!("SCSS compilation failed: {}", e))?;

            html = scss::inject_css_into_html(&html, &css);
            html = inject_color_scheme_meta(&html);
        } else {
            let (cleaned, _, scss_warnings) = scss::extract_scss_sources(&html, base_path);
            warnings.extend(scss_warnings);
            html = cleaned;
        }

        let result = if self.options.inline_css {
            self.inky
                .transform_and_inline(&html, base_path)
                .map_err(|e| format!("CSS inlining failed: {}", e))?
        } else {
            self.inky.transform(&html)
        };

        let result = strip_comments(&result);
        let result = break_at_rules(&result);
        let result = break_long_lines(&result);
        let result = strip_leading_whitespace(&result);
        let result = collapse_closing_tags(&result);
        let html = collapse_blank_lines(&result);

        Ok(Processed { html, warnings })
    }
}

static RE_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<!--.*?-->").unwrap());
static RE_TABLE_TAGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(</?(table|tbody|tr|td|th)[\s>])").unwrap());
static RE_CLOSING_TAGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(^</(?:table|tbody|tr|td|th)>\n){2,}").unwrap());
static RE_LEADING_WS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^\s+<").unwrap());
static RE_HEAD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<head[^>]*>").unwrap());
static RE_PRE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?si)<pre[^>]*>.*?</pre>").unwrap());
static RE_BLANK_LINES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n\s*\n(\s*\n)*").unwrap());
static RE_STYLE_CONTENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)(<style>)(.*?)(</style>)").unwrap());

/// Break long lines inside <style> blocks by inserting newlines
/// before @media rules and between CSS rule groups.
fn break_at_rules(html: &str) -> String {
    RE_STYLE_CONTENT
        .replace_all(html, |caps: &regex::Captures| {
            let css = &caps[2];
            let css = css.replace(" @media", "\n@media");
            // Break between CSS rules (}selector{) but keep }} together
            let css = css.replace("}", "}\n");
            let css = css.replace("}\n}", "}}");
            format!("{}{}{}", &caps[1], css.trim(), &caps[3])
        })
        .to_string()
}

/// Strip HTML comments, preserving MSO conditional comments.
fn strip_comments(html: &str) -> String {
    RE_COMMENT
        .replace_all(html, |caps: &regex::Captures| {
            let comment = &caps[0];
            if comment.starts_with("<!--[if ") || comment.contains("<![endif]") {
                comment.to_string()
            } else {
                String::new()
            }
        })
        .to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    fn pipe() -> Pipeline {
        Pipeline::new(Config::default(), PipelineOptions::default())
    }

    #[test]
    fn transforms_components_through_pipeline() {
        let out = pipe()
            .process(r#"<button href="https://x.dev">Go</button>"#, None, None)
            .unwrap();
        assert!(out.html.contains(r#"class="button""#));
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn framework_css_injected_into_head() {
        let input = "<html><head><title>T</title></head><body><row><column>x</column></row></body></html>";
        let out = pipe().process(input, None, None).unwrap();
        // Framework CSS is inlined into elements AND the media-query residue
        // stays in a <style> block in head.
        assert!(out.html.contains("<style"));
        assert!(out.html.contains("@media"));
    }

    #[test]
    fn no_framework_css_option_skips_injection() {
        let p = Pipeline::new(
            Config::default(),
            PipelineOptions { framework_css: false, inline_css: false, ..Default::default() },
        );
        let out = p.process("<p>hi</p>", None, None).unwrap();
        assert!(!out.html.contains("<style"));
    }

    #[test]
    fn scss_style_blocks_are_extracted_not_leaked() {
        let p = Pipeline::new(
            Config::default(),
            PipelineOptions { framework_css: false, inline_css: false, ..Default::default() },
        );
        let input = r#"<style type="text/scss">$x: 1;</style><p>hi</p>"#;
        let out = p.process(input, None, None).unwrap();
        assert!(!out.html.contains("text/scss"));
        assert!(out.html.contains("<p>hi</p>"));
    }

    #[test]
    fn merge_data_applied() {
        let data = serde_json::json!({"name": "Joe"});
        let p = Pipeline::new(
            Config::default(),
            PipelineOptions { framework_css: false, inline_css: false, ..Default::default() },
        );
        let out = p.process("<p>Hello {{ name }}</p>", None, Some(&data)).unwrap();
        assert!(out.html.contains("Hello Joe"));
    }

    #[test]
    fn missing_layout_is_an_error_not_empty_output() {
        let dir = std::env::temp_dir().join("inky-pipeline-missing-layout");
        std::fs::create_dir_all(&dir).unwrap();
        let res = pipe().process(
            r#"<layout src="nope.html"><p>x</p></layout>"#,
            Some(&dir),
            None,
        );
        assert!(res.is_err(), "missing layout must be Err, got {:?}", res.map(|p| p.html));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn comments_stripped_but_mso_preserved() {
        let p = Pipeline::new(
            Config::default(),
            PipelineOptions { framework_css: false, inline_css: false, ..Default::default() },
        );
        let input = "<!-- note --><!--[if mso]><table></table><![endif]--><p>x</p>";
        let out = p.process(input, None, None).unwrap();
        assert!(!out.html.contains("note"));
        assert!(out.html.contains("<!--[if mso]>"));
    }

    #[test]
    fn unreadable_linked_scss_is_warning_not_error() {
        let dir = std::env::temp_dir().join("inky-pipeline-scss-warn");
        std::fs::create_dir_all(&dir).unwrap();
        let input = r#"<html><head><link rel="stylesheet" href="nope.scss"></head><body><p>x</p></body></html>"#;
        let out = pipe().process(input, Some(&dir), None).unwrap();
        assert_eq!(out.warnings.len(), 1);
        assert!(out.warnings[0].contains("nope.scss"));
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- post-pass functions, moved verbatim from inky-cli's build.rs ---

    #[test]
    fn strip_comments_removes_regular_comments() {
        let html = "<p>a</p><!-- note --><p>b</p>";
        assert_eq!(strip_comments(html), "<p>a</p><p>b</p>");
    }

    #[test]
    fn strip_comments_preserves_mso_conditionals() {
        let html = "<!--[if mso]><table></table><![endif]-->";
        assert_eq!(strip_comments(html), html);
    }

    #[test]
    fn strip_comments_preserves_downlevel_revealed_pair() {
        // Emitted by inky-core's <not-outlook> and bulletproof buttons.
        let html = r##"<!--[if !mso]><!--><a href="#">btn</a><!--<![endif]-->"##;
        assert_eq!(strip_comments(html), html);
    }
}

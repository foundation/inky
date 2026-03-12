use regex::Regex;
use scraper::{Html, Selector};

use crate::config::Config;
use crate::Inky;

/// Severity of a validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

/// A single validation finding.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub rule: &'static str,
    pub message: String,
}

/// Run all validation checks: source-level on the input, output-level on the transformed result.
pub fn validate(html: &str, config: &Config) -> Vec<Diagnostic> {
    let mut diags = validate_source(html, config);
    let transformed = Inky::with_config(config.clone()).transform(html);
    diags.extend(validate_output(&transformed));
    diags
}

/// Validate an Inky source template (pre-transform).
pub fn validate_source(html: &str, config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_v1_syntax(html));
    diags.extend(check_missing_container(html, config));
    diags.extend(check_button_no_href(html, config));
    diags.extend(check_missing_alt(html));
    diags.extend(check_missing_preheader(html));
    diags
}

/// Validate transformed/final HTML (post-transform).
pub fn validate_output(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_email_too_large(html));
    diags.extend(check_style_block_too_large(html));
    diags.extend(check_img_no_width(html));
    diags.extend(check_deep_nesting(html));
    diags
}

// --- v1 syntax detection ---

fn check_v1_syntax(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // <columns> (plural) → should be <column> (singular)
    if Regex::new(r"<columns[\s>]").unwrap().is_match(html) {
        diags.push(Diagnostic {
            severity: Severity::Warning,
            rule: "v1-syntax",
            message: "<columns> is v1 syntax — use <column> instead, or run `inky migrate`".to_string(),
        });
    }

    // <h-line> → should be <divider>
    if Regex::new(r"<h-line[\s>]").unwrap().is_match(html) {
        diags.push(Diagnostic {
            severity: Severity::Warning,
            rule: "v1-syntax",
            message: "<h-line> is v1 syntax — use <divider> instead, or run `inky migrate`".to_string(),
        });
    }

    // large="..." on <column> → should be lg="..."
    if Regex::new(r#"<column[^>]+\blarge\s*="#).unwrap().is_match(html) {
        diags.push(Diagnostic {
            severity: Severity::Warning,
            rule: "v1-syntax",
            message: r#"large="..." is v1 syntax — use lg="..." instead, or run `inky migrate`"#.to_string(),
        });
    }

    // small="..." on <column> → should be sm="..."
    if Regex::new(r#"<column[^>]+\bsmall\s*="#).unwrap().is_match(html) {
        diags.push(Diagnostic {
            severity: Severity::Warning,
            rule: "v1-syntax",
            message: r#"small="..." is v1 syntax — use sm="..." instead, or run `inky migrate`"#.to_string(),
        });
    }

    // <spacer size="..."> → should be <spacer height="...">
    if Regex::new(r#"<spacer[^>]+\bsize\s*="#).unwrap().is_match(html) {
        diags.push(Diagnostic {
            severity: Severity::Warning,
            rule: "v1-syntax",
            message: r#"<spacer size="..."> is v1 syntax — use height="..." instead, or run `inky migrate`"#.to_string(),
        });
    }

    diags
}

// --- Source-level checks ---

fn check_missing_container(html: &str, config: &Config) -> Vec<Diagnostic> {
    let tag = &config.components.container;
    let doc = Html::parse_fragment(html);
    if let Ok(sel) = Selector::parse(tag) {
        if doc.select(&sel).next().is_none() {
            return vec![Diagnostic {
                severity: Severity::Warning,
                rule: "missing-container",
                message: format!("No <{}> element found — email content won't be centered", tag),
            }];
        }
    }
    vec![]
}

fn check_button_no_href(html: &str, config: &Config) -> Vec<Diagnostic> {
    let tag = &config.components.button;
    let doc = Html::parse_fragment(html);
    let mut diags = Vec::new();
    if let Ok(sel) = Selector::parse(tag) {
        for (i, el) in doc.select(&sel).enumerate() {
            if el.value().attr("href").is_none() {
                let text = el.text().collect::<String>();
                let snippet = if text.len() > 40 {
                    format!("{}...", &text[..40])
                } else {
                    text
                };
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    rule: "button-no-href",
                    message: format!(
                        "Button #{} missing href attribute: \"{}\"",
                        i + 1,
                        snippet.trim()
                    ),
                });
            }
        }
    }
    diags
}

fn check_missing_alt(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("img").unwrap();
    let mut count = 0;
    for el in doc.select(&sel) {
        if el.value().attr("alt").is_none() {
            count += 1;
        }
    }
    if count > 0 {
        vec![Diagnostic {
            severity: Severity::Warning,
            rule: "missing-alt",
            message: format!(
                "{} image(s) missing alt text — hurts accessibility and shows broken icon when images are blocked",
                count
            ),
        }]
    } else {
        vec![]
    }
}

fn check_missing_preheader(html: &str) -> Vec<Diagnostic> {
    let lower = html.to_lowercase();
    let has_preheader = lower.contains("preheader")
        || lower.contains("preview-text")
        || lower.contains("previewtext");

    if !has_preheader {
        vec![Diagnostic {
            severity: Severity::Warning,
            rule: "missing-preheader",
            message: "No preheader text found — inbox preview will show first visible content instead".to_string(),
        }]
    } else {
        vec![]
    }
}

// --- Output-level checks ---

const SIZE_WARNING_BYTES: usize = 90 * 1024;

fn check_email_too_large(html: &str) -> Vec<Diagnostic> {
    let size = html.len();
    if size > SIZE_WARNING_BYTES {
        let kb = size / 1024;
        vec![Diagnostic {
            severity: Severity::Warning,
            rule: "email-too-large",
            message: format!(
                "Email is {}KB — Gmail clips emails over 102KB. Consider reducing content",
                kb
            ),
        }]
    } else {
        vec![]
    }
}

const STYLE_BLOCK_LIMIT: usize = 8 * 1024;

fn check_style_block_too_large(html: &str) -> Vec<Diagnostic> {
    let re = Regex::new(r"(?si)<style[^>]*>(.*?)</style>").unwrap();
    let mut diags = Vec::new();
    for (i, caps) in re.captures_iter(html).enumerate() {
        let content = caps.get(1).unwrap().as_str();
        if content.len() > STYLE_BLOCK_LIMIT {
            let kb = content.len() / 1024;
            diags.push(Diagnostic {
                severity: Severity::Warning,
                rule: "style-block-too-large",
                message: format!(
                    "Style block #{} is {}KB — Gmail strips entire <style> blocks over 8KB",
                    i + 1,
                    kb
                ),
            });
        }
    }
    diags
}

fn check_img_no_width(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("img").unwrap();
    let mut count = 0;
    for el in doc.select(&sel) {
        let has_width = el.value().attr("width").is_some();
        let has_style_width = el
            .value()
            .attr("style")
            .map(|s| s.contains("width"))
            .unwrap_or(false);
        if !has_width && !has_style_width {
            count += 1;
        }
    }
    if count > 0 {
        vec![Diagnostic {
            severity: Severity::Warning,
            rule: "img-no-width",
            message: format!(
                "{} image(s) missing width attribute — may break layout in Outlook",
                count
            ),
        }]
    } else {
        vec![]
    }
}

const MAX_TABLE_DEPTH: usize = 4;

fn check_deep_nesting(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let max_depth = find_max_table_depth(&doc.root_element(), 0);
    if max_depth > MAX_TABLE_DEPTH {
        vec![Diagnostic {
            severity: Severity::Warning,
            rule: "deep-nesting",
            message: format!(
                "Tables nested {} levels deep — some email clients struggle beyond {} levels",
                max_depth, MAX_TABLE_DEPTH
            ),
        }]
    } else {
        vec![]
    }
}

fn find_max_table_depth(element: &scraper::ElementRef, current_depth: usize) -> usize {
    let mut max = current_depth;
    for child in element.children() {
        if let Some(child_el) = scraper::ElementRef::wrap(child) {
            let depth = if child_el.value().name() == "table" {
                current_depth + 1
            } else {
                current_depth
            };
            let child_max = find_max_table_depth(&child_el, depth);
            if child_max > max {
                max = child_max;
            }
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_missing_alt() {
        let html = r#"<container><img src="photo.jpg"></container>"#;
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "missing-alt"));
    }

    #[test]
    fn test_alt_present() {
        let html = r#"<container><img src="photo.jpg" alt="A photo"></container>"#;
        let diags = validate_source(html, &default_config());
        assert!(!diags.iter().any(|d| d.rule == "missing-alt"));
    }

    #[test]
    fn test_button_no_href() {
        let html = r#"<container><button>Click</button></container>"#;
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "button-no-href"));
    }

    #[test]
    fn test_button_with_href() {
        let html = r#"<container><button href="http://example.com">Click</button></container>"#;
        let diags = validate_source(html, &default_config());
        assert!(!diags.iter().any(|d| d.rule == "button-no-href"));
    }

    #[test]
    fn test_missing_container() {
        let html = "<row><columns>Hello</columns></row>";
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "missing-container"));
    }

    #[test]
    fn test_has_container() {
        let html = "<container><row><columns>Hello</columns></row></container>";
        let diags = validate_source(html, &default_config());
        assert!(!diags.iter().any(|d| d.rule == "missing-container"));
    }

    #[test]
    fn test_missing_preheader() {
        let html = "<container><row><columns>Hello</columns></row></container>";
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "missing-preheader"));
    }

    #[test]
    fn test_has_preheader() {
        let html = r#"<span class="preheader">Preview text</span><container></container>"#;
        let diags = validate_source(html, &default_config());
        assert!(!diags.iter().any(|d| d.rule == "missing-preheader"));
    }

    #[test]
    fn test_email_size_ok() {
        let html = "<table><tr><td>Small email</td></tr></table>";
        let diags = validate_output(html);
        assert!(!diags.iter().any(|d| d.rule == "email-too-large"));
    }

    #[test]
    fn test_email_too_large() {
        let html = "x".repeat(100 * 1024);
        let diags = validate_output(&html);
        assert!(diags.iter().any(|d| d.rule == "email-too-large"));
    }

    #[test]
    fn test_style_block_ok() {
        let html = "<style>.red { color: red; }</style>";
        let diags = validate_output(html);
        assert!(!diags.iter().any(|d| d.rule == "style-block-too-large"));
    }

    #[test]
    fn test_style_block_too_large() {
        let css = "a".repeat(9 * 1024);
        let html = format!("<style>{}</style>", css);
        let diags = validate_output(&html);
        assert!(diags.iter().any(|d| d.rule == "style-block-too-large"));
    }

    #[test]
    fn test_img_with_width() {
        let html = r#"<img src="photo.jpg" width="600" alt="test">"#;
        let diags = validate_output(html);
        assert!(!diags.iter().any(|d| d.rule == "img-no-width"));
    }

    #[test]
    fn test_img_no_width() {
        let html = r#"<img src="photo.jpg" alt="test">"#;
        let diags = validate_output(html);
        assert!(diags.iter().any(|d| d.rule == "img-no-width"));
    }

    #[test]
    fn test_shallow_nesting() {
        let html = "<table><tr><td><table><tr><td>ok</td></tr></table></td></tr></table>";
        let diags = validate_output(html);
        assert!(!diags.iter().any(|d| d.rule == "deep-nesting"));
    }

    #[test]
    fn test_deep_nesting() {
        let html = "<table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td>deep</td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table>";
        let diags = validate_output(html);
        assert!(diags.iter().any(|d| d.rule == "deep-nesting"));
    }
}

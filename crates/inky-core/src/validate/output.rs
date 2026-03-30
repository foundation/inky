use regex::Regex;
use scraper::{Html, Selector};

use super::Diagnostic;
use crate::color::{self, Color};

const GMAIL_CLIP_LIMIT: usize = 102 * 1024;
const STYLE_BLOCK_LIMIT: usize = 8 * 1024;
const MAX_TABLE_DEPTH: usize = 8;

pub(crate) fn check_gmail_clipping(html: &str) -> Vec<Diagnostic> {
    let size = html.len();
    let pct = (size as f64 / GMAIL_CLIP_LIMIT as f64 * 100.0) as usize;

    if size > GMAIL_CLIP_LIMIT {
        vec![Diagnostic::error(
            "gmail-clipping",
            format!(
                "Email is {}KB ({}% of Gmail's 102KB limit) — Gmail WILL clip this email. Reduce content immediately",
                size / 1024,
                pct
            ),
        )]
    } else if size > 90 * 1024 {
        vec![Diagnostic::warning(
            "gmail-clipping",
            format!(
                "Email is {}KB ({}% of Gmail's 102KB limit) — very close to being clipped. Reduce content urgently",
                size / 1024,
                pct
            ),
        )]
    } else if size > 70 * 1024 {
        vec![Diagnostic::warning(
            "gmail-clipping",
            format!(
                "Email is {}KB ({}% of Gmail's 102KB limit) — consider reducing content to avoid clipping",
                size / 1024,
                pct
            ),
        )]
    } else {
        vec![]
    }
}

pub(crate) fn check_style_block_too_large(html: &str) -> Vec<Diagnostic> {
    let re = Regex::new(r"(?si)<style[^>]*>(.*?)</style>").unwrap();
    let mut diags = Vec::new();
    for (i, caps) in re.captures_iter(html).enumerate() {
        let content = caps.get(1).unwrap().as_str();
        if content.len() > STYLE_BLOCK_LIMIT {
            let kb = content.len() / 1024;
            diags.push(Diagnostic::warning(
                "style-block-too-large",
                format!(
                    "Style block #{} is {}KB — Gmail strips entire <style> blocks over 8KB",
                    i + 1,
                    kb
                ),
            ));
        }
    }
    diags
}

pub(crate) fn check_deep_nesting(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let max_depth = find_max_table_depth(&doc.root_element(), 0);
    if max_depth > MAX_TABLE_DEPTH {
        vec![Diagnostic::warning(
            "deep-nesting",
            format!(
                "Tables nested {} levels deep — some email clients struggle beyond {} levels",
                max_depth, MAX_TABLE_DEPTH
            ),
        )]
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

/// Check for low color contrast between text and background in inline styles.
pub fn check_low_contrast(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("[style]").unwrap();
    let mut diags = Vec::new();

    for el in doc.select(&sel) {
        let style = match el.value().attr("style") {
            Some(s) => s,
            None => continue,
        };

        let fg = color::extract_css_property(style, "color").and_then(|v| Color::parse(&v));
        let bg = color::extract_css_property(style, "background-color")
            .or_else(|| color::extract_css_property(style, "background"))
            .and_then(|v| Color::parse(&v));

        let (fg, bg) = match (fg, bg) {
            (Some(f), Some(b)) => (f, b),
            _ => continue,
        };

        let ratio = color::contrast_ratio(&fg, &bg);

        let is_large = is_large_text(style);
        let threshold = if is_large { 3.0 } else { 4.5 };

        if ratio < threshold {
            let text: String = el.text().collect();
            let snippet = text.trim();
            let snippet = if snippet.len() > 30 {
                format!("{}...", &snippet[..30])
            } else {
                snippet.to_string()
            };
            diags.push(Diagnostic::warning(
                "low-contrast",
                format!(
                    "Low contrast ratio {:.1}:1 (needs {:.1}:1) on text \"{}\": color vs background-color",
                    ratio, threshold, snippet
                ),
            ));
        }
    }

    diags
}

fn is_large_text(style: &str) -> bool {
    let font_size =
        color::extract_css_property(style, "font-size").and_then(|v| parse_px_value(&v));

    let font_size = match font_size {
        Some(s) => s,
        None => return false,
    };

    if font_size >= 18.0 {
        return true;
    }

    if font_size >= 14.0 {
        if let Some(weight) = color::extract_css_property(style, "font-weight") {
            let w = weight.trim().to_lowercase();
            if w == "bold" || w == "bolder" || w.parse::<u32>().is_ok_and(|n| n >= 700) {
                return true;
            }
        }
    }

    false
}

fn parse_px_value(s: &str) -> Option<f64> {
    let s = s.trim().to_lowercase();
    if s.ends_with("px") {
        s[..s.len() - 2].trim().parse().ok()
    } else {
        s.parse().ok()
    }
}

pub(crate) fn check_outlook_unsupported_css(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("[style]").unwrap();

    let patterns: &[(&str, Regex)] = &[
        ("display: grid", Regex::new(r"(?i)display:\s*grid").unwrap()),
        ("grid-template", Regex::new(r"(?i)grid-template").unwrap()),
        ("display: flex", Regex::new(r"(?i)display:\s*flex").unwrap()),
        ("flex-direction", Regex::new(r"(?i)flex-direction").unwrap()),
        ("flex-wrap", Regex::new(r"(?i)flex-wrap").unwrap()),
        (
            "justify-content",
            Regex::new(r"(?i)justify-content").unwrap(),
        ),
        ("align-items", Regex::new(r"(?i)align-items").unwrap()),
        ("border-radius", Regex::new(r"(?i)border-radius").unwrap()),
    ];

    let mut found_props: Vec<&str> = Vec::new();

    for el in doc.select(&sel) {
        if let Some(style) = el.value().attr("style") {
            for (name, re) in patterns {
                if re.is_match(style) && !found_props.contains(name) {
                    found_props.push(name);
                }
            }
        }
    }

    if found_props.is_empty() {
        vec![]
    } else {
        vec![Diagnostic::warning(
            "outlook-unsupported-css",
            format!(
                "Outlook does not support these CSS properties: {}",
                found_props.join(", ")
            ),
        )]
    }
}

pub(crate) fn check_gmail_strips_class(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("[class]").unwrap();
    let mut offending: Vec<String> = Vec::new();

    for el in doc.select(&sel) {
        if let Some(class_attr) = el.value().attr("class") {
            for class_name in class_attr.split_whitespace() {
                if (class_name.contains('.') || class_name.contains(':'))
                    && !offending.contains(&class_name.to_string())
                {
                    offending.push(class_name.to_string());
                }
            }
        }
    }

    if offending.is_empty() {
        vec![]
    } else {
        vec![Diagnostic::warning(
            "gmail-strips-class",
            format!(
                "Gmail strips class names containing '.' or ':': {}",
                offending.join(", ")
            ),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::{validate_output, Severity};

    #[test]
    fn test_gmail_clipping_under_limit() {
        let html = "<table><tr><td>Small email</td></tr></table>";
        let diags = validate_output(html);
        assert!(!diags.iter().any(|d| d.rule == "gmail-clipping"));
    }

    #[test]
    fn test_gmail_clipping_over_limit() {
        let html = "x".repeat(110 * 1024);
        let diags = validate_output(&html);
        assert!(diags.iter().any(|d| d.rule == "gmail-clipping"));
        let d = diags.iter().find(|d| d.rule == "gmail-clipping").unwrap();
        assert_eq!(d.severity, Severity::Error);
    }

    #[test]
    fn test_gmail_clipping_warning_zone() {
        let html = "x".repeat(80 * 1024);
        let diags = validate_output(&html);
        assert!(diags.iter().any(|d| d.rule == "gmail-clipping"));
        let d = diags.iter().find(|d| d.rule == "gmail-clipping").unwrap();
        assert_eq!(d.severity, Severity::Warning);
    }

    #[test]
    fn test_gmail_clipping_urgent_zone() {
        let html = "x".repeat(95 * 1024);
        let diags = validate_output(&html);
        assert!(diags.iter().any(|d| d.rule == "gmail-clipping"));
        let d = diags.iter().find(|d| d.rule == "gmail-clipping").unwrap();
        assert_eq!(d.severity, Severity::Warning);
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
    fn test_shallow_nesting() {
        let html = "<table><tr><td><table><tr><td>ok</td></tr></table></td></tr></table>";
        let diags = validate_output(html);
        assert!(!diags.iter().any(|d| d.rule == "deep-nesting"));
    }

    #[test]
    fn test_deep_nesting() {
        let html = "<table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td>deep</td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table>";
        let diags = validate_output(html);
        assert!(diags.iter().any(|d| d.rule == "deep-nesting"));
    }

    #[test]
    fn test_low_contrast_detected() {
        let html = r#"<p style="color: #ffffff; background-color: #ffffff;">invisible text</p>"#;
        let diags = check_low_contrast(html);
        assert!(diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_good_contrast_passes() {
        let html = r#"<p style="color: black; background-color: white;">readable text</p>"#;
        let diags = check_low_contrast(html);
        assert!(!diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_low_contrast_light_gray_on_white() {
        let html = r#"<span style="color: #cccccc; background-color: #ffffff;">faint</span>"#;
        let diags = check_low_contrast(html);
        assert!(diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_no_contrast_check_without_both_colors() {
        let html = r#"<p style="color: red;">text</p>"#;
        let diags = check_low_contrast(html);
        assert!(!diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_large_text_lower_threshold() {
        let html = r#"<p style="color: #767676; background-color: white; font-size: 18px;">large text</p>"#;
        let diags = check_low_contrast(html);
        assert!(
            !diags.iter().any(|d| d.rule == "low-contrast"),
            "Large text at 18px with ~4.5:1 ratio should pass the 3.0 threshold"
        );
    }

    #[test]
    fn test_bold_14px_is_large_text() {
        let html = r#"<p style="color: #767676; background-color: white; font-size: 14px; font-weight: bold;">bold text</p>"#;
        let diags = check_low_contrast(html);
        assert!(
            !diags.iter().any(|d| d.rule == "low-contrast"),
            "14px bold text with ~4.5:1 ratio should pass the 3.0 large-text threshold"
        );
    }

    #[test]
    fn test_background_shorthand() {
        let html = r#"<p style="color: #ffffff; background: #ffffff;">invisible</p>"#;
        let diags = check_low_contrast(html);
        assert!(diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_low_contrast_in_validate_output() {
        let html = r#"<p style="color: #ffffff; background-color: #fefefe;">barely visible</p>"#;
        let diags = validate_output(html);
        assert!(diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_outlook_css_grid() {
        let html = r#"<div style="display: grid; grid-template-columns: 1fr 1fr;">content</div>"#;
        let diags = check_outlook_unsupported_css(html);
        assert!(diags.iter().any(|d| d.rule == "outlook-unsupported-css"));
    }

    #[test]
    fn test_outlook_css_flex() {
        let html = r#"<div style="display: flex; justify-content: center;">content</div>"#;
        let diags = check_outlook_unsupported_css(html);
        assert!(diags.iter().any(|d| d.rule == "outlook-unsupported-css"));
    }

    #[test]
    fn test_outlook_css_border_radius() {
        let html = r#"<div style="border-radius: 5px;">content</div>"#;
        let diags = check_outlook_unsupported_css(html);
        assert!(diags.iter().any(|d| d.rule == "outlook-unsupported-css"));
    }

    #[test]
    fn test_normal_css_ok() {
        let html = r#"<div style="color: red; padding: 10px;">content</div>"#;
        let diags = check_outlook_unsupported_css(html);
        assert!(!diags.iter().any(|d| d.rule == "outlook-unsupported-css"));
    }

    #[test]
    fn test_gmail_strips_class_dot() {
        let html = r#"<div class="sm.hidden">content</div>"#;
        let diags = check_gmail_strips_class(html);
        assert!(diags.iter().any(|d| d.rule == "gmail-strips-class"));
    }

    #[test]
    fn test_gmail_strips_class_colon() {
        let html = r#"<div class="hover:underline">content</div>"#;
        let diags = check_gmail_strips_class(html);
        assert!(diags.iter().any(|d| d.rule == "gmail-strips-class"));
    }

    #[test]
    fn test_gmail_normal_class_ok() {
        let html = r#"<div class="container main-content">content</div>"#;
        let diags = check_gmail_strips_class(html);
        assert!(!diags.iter().any(|d| d.rule == "gmail-strips-class"));
    }
}

use regex::Regex;
use scraper::{Html, Selector};

use crate::color::{self, Color};
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

impl Diagnostic {
    pub fn warning(rule: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            rule,
            message: message.into(),
        }
    }

    pub fn error(rule: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            rule,
            message: message.into(),
        }
    }
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
    diags.extend(check_video_no_src(html, config));
    diags.extend(check_hero_no_background(html, config));
    diags.extend(check_social_link_no_platform(html, config));
    diags.extend(check_generic_link_text(html));
    diags
}

/// Validate transformed/final HTML (post-transform).
pub fn validate_output(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_email_too_large(html));
    diags.extend(check_style_block_too_large(html));
    diags.extend(check_img_no_width(html));
    diags.extend(check_deep_nesting(html));
    diags.extend(check_low_contrast(html));
    diags
}

// --- v1 syntax detection ---

fn check_v1_syntax(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // <columns> (plural) → should be <column> (singular)
    if Regex::new(r"<columns[\s>]").unwrap().is_match(html) {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            "<columns> is v1 syntax — use <column> instead, or run `inky migrate`",
        ));
    }

    // <h-line> → should be <divider>
    if Regex::new(r"<h-line[\s>]").unwrap().is_match(html) {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            "<h-line> is v1 syntax — use <divider> instead, or run `inky migrate`",
        ));
    }

    // large="..." on <column> → should be lg="..."
    if Regex::new(r#"<column[^>]+\blarge\s*="#)
        .unwrap()
        .is_match(html)
    {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            r#"large="..." is v1 syntax — use lg="..." instead, or run `inky migrate`"#,
        ));
    }

    // small="..." on <column> → should be sm="..."
    if Regex::new(r#"<column[^>]+\bsmall\s*="#)
        .unwrap()
        .is_match(html)
    {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            r#"small="..." is v1 syntax — use sm="..." instead, or run `inky migrate`"#,
        ));
    }

    // <spacer size="..."> → should be <spacer height="...">
    if Regex::new(r#"<spacer[^>]+\bsize\s*="#)
        .unwrap()
        .is_match(html)
    {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            r#"<spacer size="..."> is v1 syntax — use height="..." instead, or run `inky migrate`"#,
        ));
    }

    diags
}

// --- Source-level checks ---

fn check_missing_container(html: &str, config: &Config) -> Vec<Diagnostic> {
    let tag = &config.components.container;
    let doc = Html::parse_fragment(html);
    if let Ok(sel) = Selector::parse(tag) {
        if doc.select(&sel).next().is_none() {
            return vec![Diagnostic::warning(
                "missing-container",
                format!(
                    "No <{}> element found — email content won't be centered",
                    tag
                ),
            )];
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
                diags.push(Diagnostic::error(
                    "button-no-href",
                    format!(
                        "Button #{} missing href attribute: \"{}\"",
                        i + 1,
                        snippet.trim()
                    ),
                ));
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
        vec![Diagnostic::warning("missing-alt", format!(
            "{} image(s) missing alt text — hurts accessibility and shows broken icon when images are blocked",
            count
        ))]
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
        vec![Diagnostic::warning(
            "missing-preheader",
            "No preheader text found — inbox preview will show first visible content instead",
        )]
    } else {
        vec![]
    }
}

fn check_video_no_src(html: &str, config: &Config) -> Vec<Diagnostic> {
    let tag = &config.components.video;
    let doc = Html::parse_fragment(html);
    let mut diags = Vec::new();
    if let Ok(sel) = Selector::parse(tag) {
        for (i, el) in doc.select(&sel).enumerate() {
            if el.value().attr("src").is_none() {
                diags.push(Diagnostic::error(
                    "video-no-src",
                    format!(
                        "<video> #{} missing src attribute — no video source to play",
                        i + 1
                    ),
                ));
            }
        }
    }
    diags
}

fn check_hero_no_background(html: &str, config: &Config) -> Vec<Diagnostic> {
    let tag = &config.components.hero;
    let doc = Html::parse_fragment(html);
    let mut diags = Vec::new();
    if let Ok(sel) = Selector::parse(tag) {
        for (i, el) in doc.select(&sel).enumerate() {
            if el.value().attr("background").is_none_or(|v| v.is_empty()) {
                diags.push(Diagnostic::warning("hero-no-background", format!(
                    "<hero> #{} missing background attribute — section will have no background image",
                    i + 1
                )));
            }
        }
    }
    diags
}

fn check_social_link_no_platform(html: &str, config: &Config) -> Vec<Diagnostic> {
    let tag = &config.components.social_link;
    let doc = Html::parse_fragment(html);
    let mut diags = Vec::new();
    if let Ok(sel) = Selector::parse(tag) {
        for (i, el) in doc.select(&sel).enumerate() {
            if el.value().attr("platform").is_none_or(|v| v.is_empty()) {
                diags.push(Diagnostic::warning("social-link-no-platform", format!(
                    "<social-link> #{} missing platform attribute — no icon or color will be applied",
                    i + 1
                )));
            }
        }
    }
    diags
}

fn check_generic_link_text(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("a").unwrap();
    let generic_phrases = ["click here", "learn more", "read more", "here", "link"];
    let mut count = 0;
    for el in doc.select(&sel) {
        let text: String = el.text().collect();
        let trimmed = text.trim().to_lowercase();
        if generic_phrases.contains(&trimmed.as_str()) {
            count += 1;
        }
    }
    if count > 0 {
        vec![Diagnostic::warning("generic-link-text", format!(
            "{} link(s) use generic text like \"Click Here\" — use descriptive text for accessibility and spam filtering",
            count
        ))]
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
        vec![Diagnostic::warning(
            "email-too-large",
            format!(
                "Email is {}KB — Gmail clips emails over 102KB. Consider reducing content",
                kb
            ),
        )]
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
        vec![Diagnostic::warning(
            "img-no-width",
            format!(
                "{} image(s) missing width attribute — may break layout in Outlook",
                count
            ),
        )]
    } else {
        vec![]
    }
}

const MAX_TABLE_DEPTH: usize = 5;

fn check_deep_nesting(html: &str) -> Vec<Diagnostic> {
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

        // Determine if text is "large" per WCAG: >= 18px, or >= 14px and bold
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

/// Determine if text is "large" per WCAG criteria based on inline style.
/// Large text is >= 18px (any weight) or >= 14px and bold (font-weight >= 700 or "bold").
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

/// Parse a px value from a CSS string like "18px" or "14.5px".
fn parse_px_value(s: &str) -> Option<f64> {
    let s = s.trim().to_lowercase();
    if s.ends_with("px") {
        s[..s.len() - 2].trim().parse().ok()
    } else {
        // Try parsing as bare number (some inline styles omit units)
        s.parse().ok()
    }
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
    fn test_video_no_src() {
        let html = r#"<container><video poster="poster.jpg"></video></container>"#;
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "video-no-src"));
    }

    #[test]
    fn test_video_with_src() {
        let html = r#"<container><video src="movie.mp4" poster="poster.jpg"></video></container>"#;
        let diags = validate_source(html, &default_config());
        assert!(!diags.iter().any(|d| d.rule == "video-no-src"));
    }

    #[test]
    fn test_hero_no_background() {
        let html = "<container><hero><p>Content</p></hero></container>";
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "hero-no-background"));
    }

    #[test]
    fn test_hero_with_background() {
        let html = r#"<container><hero background="bg.jpg"><p>Content</p></hero></container>"#;
        let diags = validate_source(html, &default_config());
        assert!(!diags.iter().any(|d| d.rule == "hero-no-background"));
    }

    #[test]
    fn test_social_link_no_platform() {
        let html = "<container><social-link href=\"#\">Link</social-link></container>";
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "social-link-no-platform"));
    }

    #[test]
    fn test_social_link_with_platform() {
        let html =
            "<container><social-link platform=\"facebook\" href=\"#\">FB</social-link></container>";
        let diags = validate_source(html, &default_config());
        assert!(!diags.iter().any(|d| d.rule == "social-link-no-platform"));
    }

    #[test]
    fn test_generic_link_text() {
        let html = "<container><a href=\"https://example.com\">Click Here</a></container>";
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "generic-link-text"));
    }

    #[test]
    fn test_generic_link_text_learn_more() {
        let html = "<container><a href=\"https://example.com\">Learn More</a></container>";
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "generic-link-text"));
    }

    #[test]
    fn test_descriptive_link_text() {
        let html =
            "<container><a href=\"https://example.com\">View your order details</a></container>";
        let diags = validate_source(html, &default_config());
        assert!(!diags.iter().any(|d| d.rule == "generic-link-text"));
    }

    #[test]
    fn test_deep_nesting() {
        let html = "<table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td>deep</td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table>";
        let diags = validate_output(html);
        assert!(diags.iter().any(|d| d.rule == "deep-nesting"));
    }

    // --- Low contrast tests ---

    #[test]
    fn test_low_contrast_detected() {
        // White text on white background — ratio 1:1, definitely low contrast
        let html = r#"<p style="color: #ffffff; background-color: #ffffff;">invisible text</p>"#;
        let diags = check_low_contrast(html);
        assert!(diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_good_contrast_passes() {
        // Black text on white background — ratio 21:1
        let html = r#"<p style="color: black; background-color: white;">readable text</p>"#;
        let diags = check_low_contrast(html);
        assert!(!diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_low_contrast_light_gray_on_white() {
        // Light gray (#ccc = 204,204,204) on white — ratio ~1.6:1
        let html = r#"<span style="color: #cccccc; background-color: #ffffff;">faint</span>"#;
        let diags = check_low_contrast(html);
        assert!(diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_no_contrast_check_without_both_colors() {
        // Only foreground color, no background — should not flag
        let html = r#"<p style="color: red;">text</p>"#;
        let diags = check_low_contrast(html);
        assert!(!diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_large_text_lower_threshold() {
        // Large text (18px) has a threshold of 3.0 instead of 4.5
        // Gray #777 on white has ratio ~4.48:1 — fails normal but passes large
        let html = r#"<p style="color: #767676; background-color: white; font-size: 18px;">large text</p>"#;
        let diags = check_low_contrast(html);
        assert!(
            !diags.iter().any(|d| d.rule == "low-contrast"),
            "Large text at 18px with ~4.5:1 ratio should pass the 3.0 threshold"
        );
    }

    #[test]
    fn test_bold_14px_is_large_text() {
        // 14px bold text counts as "large" per WCAG
        let html = r#"<p style="color: #767676; background-color: white; font-size: 14px; font-weight: bold;">bold text</p>"#;
        let diags = check_low_contrast(html);
        assert!(
            !diags.iter().any(|d| d.rule == "low-contrast"),
            "14px bold text with ~4.5:1 ratio should pass the 3.0 large-text threshold"
        );
    }

    #[test]
    fn test_background_shorthand() {
        // Use "background" instead of "background-color"
        let html = r#"<p style="color: #ffffff; background: #ffffff;">invisible</p>"#;
        let diags = check_low_contrast(html);
        assert!(diags.iter().any(|d| d.rule == "low-contrast"));
    }

    #[test]
    fn test_low_contrast_in_validate_output() {
        // Verify it's wired into validate_output
        let html = r#"<p style="color: #ffffff; background-color: #fefefe;">barely visible</p>"#;
        let diags = validate_output(html);
        assert!(diags.iter().any(|d| d.rule == "low-contrast"));
    }
}

use regex::Regex;
use scraper::{Html, Selector};

use crate::color::{self, Color};
use crate::config::Config;
use crate::Inky;

/// Severity of a validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Severity {
    Warning,
    Error,
}

/// A single validation finding.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
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
/// For best results, pass the fully assembled HTML (after layout/include/component resolution).
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
    diags.extend(check_insecure_link(html));
    diags.extend(check_empty_link(html));
    diags.extend(check_bad_shortlink(html));
    diags.extend(check_mailto_in_button(html, config));
    diags.extend(check_generic_alt(html));
    diags.extend(check_img_no_width(html));
    diags
}

/// Validate transformed/final HTML (post-transform).
pub fn validate_output(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_gmail_clipping(html));
    diags.extend(check_style_block_too_large(html));
    diags.extend(check_deep_nesting(html));
    diags.extend(check_low_contrast(html));
    diags.extend(check_outlook_unsupported_css(html));
    diags.extend(check_gmail_strips_class(html));
    diags.extend(validate_spam(html));
    diags
}

/// Run only spam-related checks on the given HTML.
pub fn validate_spam(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_spam_all_caps(html));
    diags.extend(check_spam_exclamation(html));
    diags.extend(check_spam_image_heavy(html));
    diags.extend(check_spam_missing_unsubscribe(html));
    diags.extend(check_spam_suspicious_phrases(html));
    diags
}

// --- v1 syntax detection ---

fn check_v1_syntax(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // <columns> (plural) -> should be <column> (singular)
    if Regex::new(r"<columns[\s>]").unwrap().is_match(html) {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            "<columns> is v1 syntax — use <column> instead, or run `inky migrate`",
        ));
    }

    // <h-line> -> should be <divider>
    if Regex::new(r"<h-line[\s>]").unwrap().is_match(html) {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            "<h-line> is v1 syntax — use <divider> instead, or run `inky migrate`",
        ));
    }

    // large="..." on <column> -> should be lg="..."
    if Regex::new(r#"<column[^>]+\blarge\s*="#)
        .unwrap()
        .is_match(html)
    {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            r#"large="..." is v1 syntax — use lg="..." instead, or run `inky migrate`"#,
        ));
    }

    // small="..." on <column> -> should be sm="..."
    if Regex::new(r#"<column[^>]+\bsmall\s*="#)
        .unwrap()
        .is_match(html)
    {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            r#"small="..." is v1 syntax — use sm="..." instead, or run `inky migrate`"#,
        ));
    }

    // <spacer size="..."> -> should be <spacer height="...">
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

// --- Group 1: Link Validation ---

fn check_insecure_link(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("[href]").unwrap();
    let mut count = 0;
    for el in doc.select(&sel) {
        if let Some(href) = el.value().attr("href") {
            let lower = href.to_lowercase();
            if lower.starts_with("http://")
                && !lower.starts_with("http://localhost")
                && !lower.starts_with("http://127.0.0.1")
            {
                count += 1;
            }
        }
    }
    if count > 0 {
        vec![Diagnostic::warning(
            "insecure-link",
            format!(
                "{} link(s) use http:// — use https:// for security and deliverability",
                count
            ),
        )]
    } else {
        vec![]
    }
}

fn check_empty_link(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("[href]").unwrap();
    let mut diags = Vec::new();
    for (i, el) in doc.select(&sel).enumerate() {
        if let Some(href) = el.value().attr("href") {
            let trimmed = href.trim();
            if trimmed.is_empty() {
                diags.push(Diagnostic::error(
                    "empty-link",
                    format!("Link #{} has an empty href", i + 1),
                ));
            } else if trimmed == "#" {
                diags.push(Diagnostic::warning(
                    "empty-link",
                    format!("Link #{} has a placeholder href (#)", i + 1),
                ));
            }
        }
    }
    diags
}

fn check_bad_shortlink(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("[href]").unwrap();
    let re = Regex::new(
        r"(?i)https?://(www\.)?(youtu\.be|bit\.ly|t\.co|goo\.gl|tinyurl\.com|ow\.ly|is\.gd|buff\.ly)",
    )
    .unwrap();
    let mut diags = Vec::new();
    for (i, el) in doc.select(&sel).enumerate() {
        if let Some(href) = el.value().attr("href") {
            if let Some(caps) = re.captures(href) {
                let domain = caps.get(2).unwrap().as_str();
                diags.push(Diagnostic::warning(
                    "bad-shortlink",
                    format!(
                        "Link #{} uses URL shortener {} — short links hurt deliverability and are often flagged as spam",
                        i + 1,
                        domain
                    ),
                ));
            }
        }
    }
    diags
}

fn check_mailto_in_button(html: &str, config: &Config) -> Vec<Diagnostic> {
    let tag = &config.components.button;
    let doc = Html::parse_fragment(html);
    let mut diags = Vec::new();
    if let Ok(sel) = Selector::parse(tag) {
        for (i, el) in doc.select(&sel).enumerate() {
            if let Some(href) = el.value().attr("href") {
                if href.trim().to_lowercase().starts_with("mailto:") {
                    diags.push(Diagnostic::warning(
                        "mailto-in-button",
                        format!(
                            "Button #{} uses a mailto: link — this may not work well as a button in all email clients",
                            i + 1
                        ),
                    ));
                }
            }
        }
    }
    diags
}

// --- Group 2: Alt Text Quality ---

fn check_generic_alt(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("img[alt]").unwrap();
    let generic_terms = [
        "image",
        "img",
        "logo",
        "banner",
        "photo",
        "picture",
        "icon",
        "graphic",
        "screenshot",
    ];
    let mut count = 0;
    for el in doc.select(&sel) {
        if let Some(alt) = el.value().attr("alt") {
            let trimmed = alt.trim();
            let lower = trimmed.to_lowercase();
            if generic_terms.contains(&lower.as_str()) || trimmed.len() == 1 {
                count += 1;
            }
        }
    }
    if count > 0 {
        vec![Diagnostic::warning(
            "generic-alt",
            format!(
                "{} image(s) have generic or single-character alt text — use descriptive alt text for accessibility",
                count
            ),
        )]
    } else {
        vec![]
    }
}

// --- Output-level checks ---

const GMAIL_CLIP_LIMIT: usize = 102 * 1024;

fn check_gmail_clipping(html: &str) -> Vec<Diagnostic> {
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
        if el.value().attr("width").is_none() {
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

const MAX_TABLE_DEPTH: usize = 8;

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

// --- Group 3: Rendering Quirk Warnings ---

fn check_outlook_unsupported_css(html: &str) -> Vec<Diagnostic> {
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

fn check_gmail_strips_class(html: &str) -> Vec<Diagnostic> {
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

// --- Group 5: Spam Score ---

/// Extract visible text from HTML, skipping style/script/head elements.
fn extract_visible_text(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let mut text = String::new();
    extract_text_recursive(&doc.root_element(), &mut text);
    text
}

fn extract_text_recursive(element: &scraper::ElementRef, text: &mut String) {
    for child in element.children() {
        if let Some(child_el) = scraper::ElementRef::wrap(child) {
            let tag = child_el.value().name().to_lowercase();
            if tag == "style" || tag == "script" || tag == "head" {
                continue;
            }
            extract_text_recursive(&child_el, text);
        } else if let Some(t) = child.value().as_text() {
            text.push_str(t);
        }
    }
}

fn check_spam_all_caps(html: &str) -> Vec<Diagnostic> {
    let text = extract_visible_text(html);
    let total_alpha: usize = text.chars().filter(|c| c.is_alphabetic()).count();
    if total_alpha <= 50 {
        return vec![];
    }
    let upper_alpha: usize = text.chars().filter(|c| c.is_uppercase()).count();
    let pct = (upper_alpha as f64 / total_alpha as f64) * 100.0;
    if pct > 20.0 {
        vec![Diagnostic::warning(
            "spam-all-caps",
            format!(
                "{:.0}% of text is uppercase — excessive caps can trigger spam filters",
                pct
            ),
        )]
    } else {
        vec![]
    }
}

fn check_spam_exclamation(html: &str) -> Vec<Diagnostic> {
    let text = extract_visible_text(html);
    let re = Regex::new(r"!{3,}").unwrap();
    let count = re.find_iter(&text).count();
    if count > 0 {
        vec![Diagnostic::warning(
            "spam-exclamation",
            format!(
                "{} occurrence(s) of 3+ consecutive exclamation marks — this can trigger spam filters",
                count
            ),
        )]
    } else {
        vec![]
    }
}

fn check_spam_image_heavy(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("img").unwrap();
    let img_count = doc.select(&sel).count();
    if img_count == 0 {
        return vec![];
    }
    let text = extract_visible_text(html);
    let text_len = text.trim().len();
    let ratio = text_len as f64 / img_count as f64;
    if ratio < 100.0 {
        vec![Diagnostic::warning(
            "spam-image-heavy",
            format!(
                "Only {} chars of text for {} image(s) — image-heavy emails are often flagged as spam",
                text_len, img_count
            ),
        )]
    } else {
        vec![]
    }
}

fn check_spam_missing_unsubscribe(html: &str) -> Vec<Diagnostic> {
    let doc = Html::parse_fragment(html);
    let sel = Selector::parse("a").unwrap();
    for el in doc.select(&sel) {
        let text: String = el.text().collect();
        if text.to_lowercase().contains("unsubscribe") {
            return vec![];
        }
        if let Some(href) = el.value().attr("href") {
            if href.to_lowercase().contains("unsubscribe") {
                return vec![];
            }
        }
    }
    vec![Diagnostic::warning(
        "spam-missing-unsubscribe",
        "No unsubscribe link found — most spam filters expect one and many jurisdictions require it",
    )]
}

fn check_spam_suspicious_phrases(html: &str) -> Vec<Diagnostic> {
    let text = extract_visible_text(html).to_lowercase();
    let phrases = [
        "act now",
        "limited time",
        "click here",
        "free",
        "winner",
        "congratulations",
        "no obligation",
        "risk free",
        "buy now",
        "order now",
        "don't delete",
        "urgent",
    ];
    let mut found: Vec<&str> = Vec::new();
    for phrase in &phrases {
        if text.contains(phrase) {
            found.push(phrase);
        }
    }
    if found.len() >= 3 {
        vec![Diagnostic::warning(
            "spam-suspicious-phrases",
            format!(
                "{} suspicious phrases found: {} — this combination may trigger spam filters",
                found.len(),
                found.join(", ")
            ),
        )]
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> Config {
        Config::default()
    }

    // --- v1 syntax detection ---

    #[test]
    fn test_v1_syntax_columns_plural() {
        let html = "<columns>content</columns>";
        let diags = check_v1_syntax(html);
        assert!(diags.iter().any(|d| d.rule == "v1-syntax"));
    }

    #[test]
    fn test_v1_syntax_h_line() {
        let html = "<h-line></h-line>";
        let diags = check_v1_syntax(html);
        assert!(diags
            .iter()
            .any(|d| d.rule == "v1-syntax" && d.message.contains("h-line")));
    }

    #[test]
    fn test_v1_syntax_large_attr() {
        let html = r#"<column large="8">content</column>"#;
        let diags = check_v1_syntax(html);
        assert!(diags
            .iter()
            .any(|d| d.rule == "v1-syntax" && d.message.contains("large")));
    }

    #[test]
    fn test_v1_syntax_small_attr() {
        let html = r#"<column small="6">content</column>"#;
        let diags = check_v1_syntax(html);
        assert!(diags
            .iter()
            .any(|d| d.rule == "v1-syntax" && d.message.contains("small")));
    }

    #[test]
    fn test_v1_syntax_spacer_size() {
        let html = r#"<spacer size="20"></spacer>"#;
        let diags = check_v1_syntax(html);
        assert!(diags
            .iter()
            .any(|d| d.rule == "v1-syntax" && d.message.contains("size")));
    }

    #[test]
    fn test_v2_syntax_no_warnings() {
        let html = r#"<column sm="6" lg="8">content</column>"#;
        let diags = check_v1_syntax(html);
        assert!(diags.is_empty());
    }

    // --- Source-level checks ---

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
    fn test_img_with_width() {
        let html = r#"<img src="photo.jpg" width="600" alt="test">"#;
        let diags = validate_output(html);
        assert!(!diags.iter().any(|d| d.rule == "img-no-width"));
    }

    #[test]
    fn test_img_no_width() {
        let html = r#"<img src="photo.jpg" alt="test">"#;
        let diags = validate_source(html, &default_config());
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
        // 9 levels of nesting (threshold is 8)
        let html = "<table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td><table><tr><td>deep</td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table></td></tr></table>";
        let diags = validate_output(html);
        assert!(diags.iter().any(|d| d.rule == "deep-nesting"));
    }

    // --- Low contrast tests ---

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

    // --- Group 1: Link Validation tests ---

    #[test]
    fn test_insecure_link() {
        let html = r#"<a href="http://example.com">Link</a>"#;
        let diags = check_insecure_link(html);
        assert!(diags.iter().any(|d| d.rule == "insecure-link"));
    }

    #[test]
    fn test_secure_link_ok() {
        let html = r#"<a href="https://example.com">Link</a>"#;
        let diags = check_insecure_link(html);
        assert!(!diags.iter().any(|d| d.rule == "insecure-link"));
    }

    #[test]
    fn test_insecure_link_localhost_excluded() {
        let html = r#"<a href="http://localhost:3000">Dev</a>"#;
        let diags = check_insecure_link(html);
        assert!(!diags.iter().any(|d| d.rule == "insecure-link"));
    }

    #[test]
    fn test_insecure_link_127_excluded() {
        let html = r#"<a href="http://127.0.0.1:8080">Dev</a>"#;
        let diags = check_insecure_link(html);
        assert!(!diags.iter().any(|d| d.rule == "insecure-link"));
    }

    #[test]
    fn test_empty_link_hash() {
        let html = r##"<a href="#">Link</a>"##;
        let diags = check_empty_link(html);
        assert!(diags.iter().any(|d| d.rule == "empty-link"));
    }

    #[test]
    fn test_empty_link_empty_string() {
        let html = r#"<a href="">Link</a>"#;
        let diags = check_empty_link(html);
        assert!(diags.iter().any(|d| d.rule == "empty-link"));
    }

    #[test]
    fn test_normal_link_ok() {
        let html = r#"<a href="https://example.com">Link</a>"#;
        let diags = check_empty_link(html);
        assert!(!diags.iter().any(|d| d.rule == "empty-link"));
    }

    #[test]
    fn test_bad_shortlink_youtube() {
        let html = r#"<a href="https://youtu.be/abc123">Video</a>"#;
        let diags = check_bad_shortlink(html);
        assert!(diags.iter().any(|d| d.rule == "bad-shortlink"));
    }

    #[test]
    fn test_bad_shortlink_bitly() {
        let html = r#"<a href="https://bit.ly/abc123">Link</a>"#;
        let diags = check_bad_shortlink(html);
        assert!(diags.iter().any(|d| d.rule == "bad-shortlink"));
    }

    #[test]
    fn test_normal_link_not_short() {
        let html = r#"<a href="https://www.youtube.com/watch?v=abc123">Video</a>"#;
        let diags = check_bad_shortlink(html);
        assert!(!diags.iter().any(|d| d.rule == "bad-shortlink"));
    }

    #[test]
    fn test_mailto_in_button() {
        let html =
            r#"<container><button href="mailto:test@example.com">Email Us</button></container>"#;
        let diags = check_mailto_in_button(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "mailto-in-button"));
    }

    #[test]
    fn test_button_normal_href_no_mailto() {
        let html = r#"<container><button href="https://example.com">Click</button></container>"#;
        let diags = check_mailto_in_button(html, &default_config());
        assert!(!diags.iter().any(|d| d.rule == "mailto-in-button"));
    }

    // --- Group 2: Alt Text Quality tests ---

    #[test]
    fn test_generic_alt_image() {
        let html = r#"<img src="photo.jpg" alt="image">"#;
        let diags = check_generic_alt(html);
        assert!(diags.iter().any(|d| d.rule == "generic-alt"));
    }

    #[test]
    fn test_generic_alt_logo() {
        let html = r#"<img src="logo.png" alt="Logo">"#;
        let diags = check_generic_alt(html);
        assert!(diags.iter().any(|d| d.rule == "generic-alt"));
    }

    #[test]
    fn test_generic_alt_single_char() {
        let html = r#"<img src="photo.jpg" alt="x">"#;
        let diags = check_generic_alt(html);
        assert!(diags.iter().any(|d| d.rule == "generic-alt"));
    }

    #[test]
    fn test_descriptive_alt_ok() {
        let html = r#"<img src="photo.jpg" alt="A golden retriever playing in the park">"#;
        let diags = check_generic_alt(html);
        assert!(!diags.iter().any(|d| d.rule == "generic-alt"));
    }

    // --- Group 3: Rendering Quirk tests ---

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

    // --- Group 5: Spam Score tests ---

    #[test]
    fn test_spam_all_caps() {
        let html =
            r#"<p>THIS IS ALL UPPERCASE TEXT AND IT KEEPS GOING ON AND ON TO BE LONG ENOUGH</p>"#;
        let diags = check_spam_all_caps(html);
        assert!(diags.iter().any(|d| d.rule == "spam-all-caps"));
    }

    #[test]
    fn test_spam_normal_case() {
        let html = r#"<p>This is a normal sentence with proper capitalization and enough text to pass the threshold easily.</p>"#;
        let diags = check_spam_all_caps(html);
        assert!(!diags.iter().any(|d| d.rule == "spam-all-caps"));
    }

    #[test]
    fn test_spam_all_caps_short_text_ok() {
        // Short text should not trigger even if all caps
        let html = "<p>ABC</p>";
        let diags = check_spam_all_caps(html);
        assert!(!diags.iter().any(|d| d.rule == "spam-all-caps"));
    }

    #[test]
    fn test_spam_exclamation() {
        let html = "<p>Buy now!!! Amazing deal!!!</p>";
        let diags = check_spam_exclamation(html);
        assert!(diags.iter().any(|d| d.rule == "spam-exclamation"));
    }

    #[test]
    fn test_spam_exclamation_ok() {
        let html = "<p>Great product! Love it!</p>";
        let diags = check_spam_exclamation(html);
        assert!(!diags.iter().any(|d| d.rule == "spam-exclamation"));
    }

    #[test]
    fn test_spam_image_heavy() {
        let html = r#"<img src="a.jpg"><img src="b.jpg"><img src="c.jpg"><p>Hi</p>"#;
        let diags = check_spam_image_heavy(html);
        assert!(diags.iter().any(|d| d.rule == "spam-image-heavy"));
    }

    #[test]
    fn test_spam_image_with_text_ok() {
        let long_text = "a".repeat(500);
        let html = format!(r#"<img src="a.jpg"><p>{}</p>"#, long_text);
        let diags = check_spam_image_heavy(&html);
        assert!(!diags.iter().any(|d| d.rule == "spam-image-heavy"));
    }

    #[test]
    fn test_spam_missing_unsubscribe() {
        let html = "<p>Hello world</p><a href=\"https://example.com\">Visit us</a>";
        let diags = check_spam_missing_unsubscribe(html);
        assert!(diags.iter().any(|d| d.rule == "spam-missing-unsubscribe"));
    }

    #[test]
    fn test_spam_has_unsubscribe() {
        let html = "<p>Hello world</p><a href=\"https://example.com/unsubscribe\">Unsubscribe</a>";
        let diags = check_spam_missing_unsubscribe(html);
        assert!(!diags.iter().any(|d| d.rule == "spam-missing-unsubscribe"));
    }

    #[test]
    fn test_spam_has_unsubscribe_in_href() {
        let html = "<p>Hello world</p><a href=\"https://example.com/unsubscribe\">Click here</a>";
        let diags = check_spam_missing_unsubscribe(html);
        assert!(!diags.iter().any(|d| d.rule == "spam-missing-unsubscribe"));
    }

    #[test]
    fn test_spam_suspicious_phrases() {
        let html = "<p>Act now! This is a limited time offer! Click here to get your free gift! You are a winner!</p>";
        let diags = check_spam_suspicious_phrases(html);
        assert!(diags.iter().any(|d| d.rule == "spam-suspicious-phrases"));
    }

    #[test]
    fn test_spam_clean_copy() {
        let html =
            "<p>We wanted to share our latest product updates with you. Check out our new features.</p>";
        let diags = check_spam_suspicious_phrases(html);
        assert!(!diags.iter().any(|d| d.rule == "spam-suspicious-phrases"));
    }

    #[test]
    fn test_spam_two_phrases_ok() {
        // Only 2 phrases should not trigger (need 3+)
        let html = "<p>Act now and get it free.</p>";
        let diags = check_spam_suspicious_phrases(html);
        assert!(!diags.iter().any(|d| d.rule == "spam-suspicious-phrases"));
    }

    #[test]
    fn test_validate_spam_function() {
        let html = "<p>ACT NOW!!! LIMITED TIME OFFER!!! CLICK HERE TO GET YOUR FREE GIFT!!! YOU ARE A WINNER!!! CONGRATULATIONS!!!</p>";
        let diags = validate_spam(html);
        // Should have multiple findings
        assert!(!diags.is_empty());
        // Should include at least exclamation and suspicious phrases
        assert!(diags.iter().any(|d| d.rule == "spam-exclamation"));
        assert!(diags.iter().any(|d| d.rule == "spam-suspicious-phrases"));
    }
}

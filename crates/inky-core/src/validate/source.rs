use std::sync::LazyLock;

use regex::Regex;
use scraper::{Html, Selector};

use super::Diagnostic;
use crate::config::Config;

// --- v1 syntax detection ---

static RE_V1_COLUMNS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<columns[\s>]").unwrap());
static RE_V1_HLINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<h-line[\s>]").unwrap());
static RE_V1_LARGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<column[^>]+\blarge\s*="#).unwrap());
static RE_V1_SMALL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<column[^>]+\bsmall\s*="#).unwrap());
static RE_V1_SPACER_SIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<spacer[^>]+\bsize\s*="#).unwrap());

pub(crate) fn check_v1_syntax(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    if RE_V1_COLUMNS.is_match(html) {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            "<columns> is v1 syntax — use <column> instead, or run `inky migrate`",
        ));
    }

    if RE_V1_HLINE.is_match(html) {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            "<h-line> is v1 syntax — use <divider> instead, or run `inky migrate`",
        ));
    }

    if RE_V1_LARGE.is_match(html) {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            r#"large="..." is v1 syntax — use lg="..." instead, or run `inky migrate`"#,
        ));
    }

    if RE_V1_SMALL.is_match(html) {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            r#"small="..." is v1 syntax — use sm="..." instead, or run `inky migrate`"#,
        ));
    }

    if RE_V1_SPACER_SIZE.is_match(html) {
        diags.push(Diagnostic::warning(
            "v1-syntax",
            r#"<spacer size="..."> is v1 syntax — use height="..." instead, or run `inky migrate`"#,
        ));
    }

    diags
}

// --- Source-level checks ---

pub(crate) fn check_missing_container(html: &str, config: &Config) -> Vec<Diagnostic> {
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

pub(crate) fn check_button_no_href(html: &str, config: &Config) -> Vec<Diagnostic> {
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

pub(crate) fn check_missing_alt(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_missing_preheader(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_video_no_src(html: &str, config: &Config) -> Vec<Diagnostic> {
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

pub(crate) fn check_hero_no_background(html: &str, config: &Config) -> Vec<Diagnostic> {
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

pub(crate) fn check_social_link_no_platform(html: &str, config: &Config) -> Vec<Diagnostic> {
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

pub(crate) fn check_generic_link_text(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_insecure_link(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_empty_link(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_bad_shortlink(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_mailto_in_button(html: &str, config: &Config) -> Vec<Diagnostic> {
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

pub(crate) fn check_generic_alt(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_img_no_width(html: &str) -> Vec<Diagnostic> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::validate::{validate_output, validate_source};

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
    fn test_img_no_width() {
        let html = r#"<img src="photo.jpg" alt="test">"#;
        let diags = validate_source(html, &default_config());
        assert!(diags.iter().any(|d| d.rule == "img-no-width"));
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

    #[test]
    fn test_img_with_width() {
        let html = r#"<img src="photo.jpg" width="600" alt="test">"#;
        let diags = validate_output(html);
        assert!(!diags.iter().any(|d| d.rule == "img-no-width"));
    }
}

use std::sync::LazyLock;

use regex::Regex;
use scraper::{Html, Selector};

use super::Diagnostic;

static RE_EXCLAMATION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!{3,}").unwrap());

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

pub(crate) fn check_spam_all_caps(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_spam_exclamation(html: &str) -> Vec<Diagnostic> {
    let text = extract_visible_text(html);
    let count = RE_EXCLAMATION.find_iter(&text).count();
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

pub(crate) fn check_spam_image_heavy(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_spam_missing_unsubscribe(html: &str) -> Vec<Diagnostic> {
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

pub(crate) fn check_spam_suspicious_phrases(html: &str) -> Vec<Diagnostic> {
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
    use crate::validate::validate_spam;

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
        let html = "<p>Act now and get it free.</p>";
        let diags = check_spam_suspicious_phrases(html);
        assert!(!diags.iter().any(|d| d.rule == "spam-suspicious-phrases"));
    }

    #[test]
    fn test_validate_spam_function() {
        let html = "<p>ACT NOW!!! LIMITED TIME OFFER!!! CLICK HERE TO GET YOUR FREE GIFT!!! YOU ARE A WINNER!!! CONGRATULATIONS!!!</p>";
        let diags = validate_spam(html);
        assert!(!diags.is_empty());
        assert!(diags.iter().any(|d| d.rule == "spam-exclamation"));
        assert!(diags.iter().any(|d| d.rule == "spam-suspicious-phrases"));
    }
}

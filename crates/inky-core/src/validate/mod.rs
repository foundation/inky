mod output;
mod source;
mod spam;

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
    diags.extend(source::check_v1_syntax(html));
    diags.extend(source::check_missing_container(html, config));
    diags.extend(source::check_button_no_href(html, config));
    diags.extend(source::check_missing_alt(html));
    diags.extend(source::check_missing_preheader(html));
    diags.extend(source::check_video_no_src(html, config));
    diags.extend(source::check_hero_no_background(html, config));
    diags.extend(source::check_social_link_no_platform(html, config));
    diags.extend(source::check_generic_link_text(html));
    diags.extend(source::check_insecure_link(html));
    diags.extend(source::check_empty_link(html));
    diags.extend(source::check_bad_shortlink(html));
    diags.extend(source::check_mailto_in_button(html, config));
    diags.extend(source::check_generic_alt(html));
    diags.extend(source::check_img_no_width(html));
    diags
}

/// Validate transformed/final HTML (post-transform).
pub fn validate_output(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(output::check_gmail_clipping(html));
    diags.extend(output::check_style_block_too_large(html));
    diags.extend(output::check_deep_nesting(html));
    diags.extend(output::check_low_contrast(html));
    diags.extend(output::check_outlook_unsupported_css(html));
    diags.extend(output::check_gmail_strips_class(html));
    diags.extend(validate_spam(html));
    diags
}

/// Run only spam-related checks on the given HTML.
pub fn validate_spam(html: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(spam::check_spam_all_caps(html));
    diags.extend(spam::check_spam_exclamation(html));
    diags.extend(spam::check_spam_image_heavy(html));
    diags.extend(spam::check_spam_missing_unsubscribe(html));
    diags.extend(spam::check_spam_suspicious_phrases(html));
    diags
}

// Re-export check_low_contrast for direct use
pub use output::check_low_contrast;

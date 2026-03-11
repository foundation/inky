pub mod attrs;
pub mod components;
pub mod config;
#[cfg(feature = "css-inlining")]
pub mod inline;

use regex::Regex;
use scraper::{Html, Selector};

use components::transform_component;
pub use config::{ComponentNames, Config};

/// The Inky parser. Converts simple HTML tags into email-safe table markup.
pub struct Inky {
    config: Config,
}

impl Inky {
    /// Create a new Inky parser with default configuration.
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Create a new Inky parser with custom configuration.
    pub fn with_config(config: Config) -> Self {
        Self { config }
    }

    /// Transform Inky HTML into email-safe table HTML.
    pub fn transform(&self, html: &str) -> String {
        // Step 1: Extract <raw> blocks and replace with placeholders
        let (raws, working_html) = extract_raws(html);

        // Step 2: Iteratively transform custom components
        let mut current = working_html;

        loop {
            // Parse HTML fresh each iteration since the DOM changes
            let doc = Html::parse_fragment(&current);
            let tags = self.config.components.all_tags();

            // Build a selector for all custom component tags
            // <center> needs special handling to avoid infinite loops
            let selector_str = tags
                .iter()
                .map(|tag| {
                    if *tag == self.config.components.center {
                        format!("{}:not([data-parsed])", tag)
                    } else {
                        tag.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            let selector = match Selector::parse(&selector_str) {
                Ok(s) => s,
                Err(_) => break,
            };

            // Find the first matching component
            let first_match = doc.select(&selector).next();
            let Some(element) = first_match else {
                break;
            };

            let tag_name = element.value().name().to_string();

            // Columns need special handling: process all sibling columns at once
            // because html5ever restructures <th> output into tables, breaking sibling detection
            if tag_name == self.config.components.columns {
                let replaced = transform_all_columns(&current, &self.config);
                if replaced == current {
                    break;
                }
                current = replaced;
                continue;
            }

            // Transform the component
            let new_html = match transform_component(&element, &self.config) {
                Some(html) => html,
                None => break,
            };

            // Use regex to find the original tag in the source string.
            // html5ever reorders attributes, so we can't match the serialized output directly.
            let replaced = replace_first_tag(&current, &tag_name, &new_html);
            if replaced == current {
                // No replacement made — avoid infinite loop
                break;
            }
            current = replaced;
        }

        // Step 3: Remove data-parsed attributes
        current = current.replace(" data-parsed=\"\"", "");

        // Step 4: Re-inject raw blocks
        re_inject_raws(&current, &raws)
    }

    /// Transform Inky HTML and then inline CSS.
    ///
    /// This is the typical email production pipeline: transform components
    /// into tables first, then move CSS declarations into inline `style`
    /// attributes for maximum email client compatibility.
    ///
    /// Resolves both `<style>` blocks and `<link rel="stylesheet" href="...">` tags.
    /// `base_path` is the directory used to resolve relative CSS file paths.
    /// Pass `None` if the HTML only uses inline `<style>` blocks.
    #[cfg(feature = "css-inlining")]
    pub fn transform_and_inline(
        &self,
        html: &str,
        base_path: Option<&std::path::Path>,
    ) -> Result<String, String> {
        let transformed = self.transform(html);
        inline::inline_css(&transformed, base_path)
    }
}

impl Default for Inky {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform all adjacent <columns> tags in a group, handling first/last/sibling-count correctly.
/// This is needed because html5ever restructures <th> elements into table structures,
/// breaking sibling detection when processing columns one at a time.
fn transform_all_columns(html: &str, config: &Config) -> String {
    let tag = &config.components.columns;
    let escaped = regex::escape(tag);

    // Pattern to match a single <columns>...</columns> tag
    let col_pattern = format!(r"(?s)<{e}(?:\s[^>]*)?>.*?</{e}>", e = escaped);
    let col_re = Regex::new(&col_pattern).unwrap();

    // Pattern to match a group of adjacent columns (with optional whitespace between)
    let group_pattern = format!(
        r"(?s)(?:<{e}(?:\s[^>]*)?>.*?</{e}>\s*)+",
        e = escaped
    );
    let group_re = Regex::new(&group_pattern).unwrap();

    let group_match = match group_re.find(html) {
        Some(m) => m,
        None => return html.to_string(),
    };

    let group_html = group_match.as_str();

    // Find individual columns within the group
    let individual_matches: Vec<regex::Match> = col_re.find_iter(group_html).collect();
    let col_count = individual_matches.len() as u32;

    // Transform each column with position info
    let mut transformed_parts = Vec::new();
    for (i, m) in individual_matches.iter().enumerate() {
        let col_html = m.as_str();
        let is_first = i == 0;
        let is_last = i == individual_matches.len() - 1;

        let doc = Html::parse_fragment(col_html);
        let sel = Selector::parse(tag).unwrap();
        if let Some(element) = doc.select(&sel).next() {
            let transformed = components::transform_column_with_position(
                &element, config, col_count, is_first, is_last,
            );
            transformed_parts.push(transformed);
        } else {
            transformed_parts.push(col_html.to_string());
        }
    }

    // Reconstruct: replace columns but preserve whitespace between them
    let mut result = String::new();
    let mut pos = 0;
    for (i, m) in individual_matches.iter().enumerate() {
        if m.start() > pos {
            result.push_str(&group_html[pos..m.start()]);
        }
        result.push_str(&transformed_parts[i]);
        pos = m.end();
    }
    if pos < group_html.len() {
        result.push_str(&group_html[pos..]);
    }

    format!(
        "{}{}{}",
        &html[..group_match.start()],
        result,
        &html[group_match.end()..]
    )
}

/// Replace the first occurrence of a custom tag (with its content) in the source HTML.
/// Handles nested tags correctly by tracking depth.
fn replace_first_tag(html: &str, tag_name: &str, replacement: &str) -> String {
    let escaped = regex::escape(tag_name);

    // Find opening tags
    let open_pattern = format!(r"<{}(?:\s[^>]*)?>", escaped);
    let close_pattern = format!(r"</{}>", escaped);
    let self_close_pattern = format!(r"<{}(?:\s[^>]*)?\s*/>", escaped);

    let open_re = Regex::new(&open_pattern).unwrap();
    let close_re = Regex::new(&close_pattern).unwrap();
    let self_close_re = Regex::new(&self_close_pattern).unwrap();

    // Try self-closing first
    if let Some(m) = self_close_re.find(html) {
        // Make sure this isn't also matched as an opening tag with content
        let has_close_after = close_re.find(&html[m.start()..]).is_some();
        if !has_close_after {
            return format!("{}{}{}", &html[..m.start()], replacement, &html[m.end()..]);
        }
    }

    // Find all opening tag positions
    let opens: Vec<(usize, usize)> = open_re
        .find_iter(html)
        .map(|m| (m.start(), m.end()))
        .collect();

    if opens.is_empty() {
        return html.to_string();
    }

    // For each opening tag, find its matching closing tag (tracking nesting depth)
    for &(open_start, open_end) in &opens {
        let open_str = &html[open_start..open_end];

        // For <center>, skip tags that already have data-parsed
        if tag_name == "center" && open_str.contains("data-parsed") {
            continue;
        }

        // Track nesting depth to find the matching close tag
        let mut depth = 1;
        let mut pos = open_end;

        loop {
            // Find next opening or closing tag after current position
            let next_open = open_re.find(&html[pos..]).map(|m| (pos + m.start(), pos + m.end()));
            let next_close = close_re.find(&html[pos..]).map(|m| (pos + m.start(), pos + m.end()));

            match (next_open, next_close) {
                (Some((os, oe)), Some((cs, ce))) => {
                    if cs < os {
                        // Close tag comes first
                        depth -= 1;
                        if depth == 0 {
                            return format!("{}{}{}", &html[..open_start], replacement, &html[ce..]);
                        }
                        pos = ce;
                    } else {
                        // Open tag comes first
                        depth += 1;
                        pos = oe;
                    }
                }
                (None, Some((_cs, ce))) => {
                    depth -= 1;
                    if depth == 0 {
                        return format!("{}{}{}", &html[..open_start], replacement, &html[ce..]);
                    }
                    pos = ce;
                }
                _ => break, // No more tags
            }
        }

        // If we get here, there's no matching close tag — treat the open tag alone
        return format!("{}{}{}", &html[..open_start], replacement, &html[open_end..]);
    }

    html.to_string()
}

/// Extract `<raw>` blocks from HTML, replacing them with placeholders.
fn extract_raws(html: &str) -> (Vec<String>, String) {
    let re = Regex::new(r"(?s)(?:\n *)?< *raw *>(.*?)</ *raw *>(?: *\n)?").unwrap();
    let mut raws = Vec::new();
    let mut result = html.to_string();
    let mut i = 0;

    while let Some(caps) = re.captures(&result) {
        let full_match = caps.get(0).unwrap();
        let content = caps.get(1).unwrap().as_str().to_string();
        raws.push(content);
        result = format!(
            "{}###RAW{}###{}",
            &result[..full_match.start()],
            i,
            &result[full_match.end()..]
        );
        i += 1;
    }

    (raws, result)
}

/// Re-inject raw block content back into placeholders.
fn re_inject_raws(html: &str, raws: &[String]) -> String {
    let mut result = html.to_string();
    for (i, raw) in raws.iter().enumerate() {
        let placeholder = format!("###RAW{}###", i);
        result = result.replace(&placeholder, raw);
    }
    result
}

/// Convenience function to transform HTML with default settings.
pub fn transform(html: &str) -> String {
    Inky::new().transform(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_raws() {
        let input = "before<raw>keep me</raw>after";
        let (raws, result) = extract_raws(input);
        assert_eq!(raws, vec!["keep me"]);
        assert_eq!(result, "before###RAW0###after");
    }

    #[test]
    fn test_re_inject_raws() {
        let html = "before###RAW0###after";
        let raws = vec!["keep me".to_string()];
        assert_eq!(re_inject_raws(html, &raws), "beforekeep meafter");
    }

    #[test]
    fn test_transform_button() {
        let input = r#"<button href="http://example.com">Click</button>"#;
        let result = transform(input);
        assert!(result.contains("table class=\"button\""));
        assert!(result.contains("http://example.com"));
        assert!(result.contains("Click"));
    }

    #[test]
    fn test_transform_row() {
        let input = "<row>content</row>";
        let result = transform(input);
        assert!(result.contains("table"));
        assert!(result.contains("class=\"row\""));
        assert!(result.contains("content"));
    }

    #[test]
    fn test_transform_container() {
        let input = "<container>content</container>";
        let result = transform(input);
        assert!(result.contains("class=\"container\""));
        assert!(result.contains("align=\"center\""));
    }

    #[test]
    fn test_transform_h_line() {
        let input = "<h-line></h-line>";
        let result = transform(input);
        assert!(result.contains("class=\"h-line\""));
        assert!(result.contains("<tbody>"));
    }

    #[test]
    fn test_transform_spacer() {
        let input = "<spacer size=\"10\"></spacer>";
        let result = transform(input);
        assert!(result.contains("height=\"10\""));
        assert!(result.contains("font-size:10px"));
    }

    #[test]
    fn test_raw_passthrough() {
        let input = "before<raw><button>not transformed</button></raw>after";
        let result = transform(input);
        assert!(result.contains("<button>not transformed</button>"));
        assert!(!result.contains("###RAW"));
    }
}

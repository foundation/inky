pub mod attrs;
pub mod color;
pub mod components;
pub mod config;
pub mod include;
#[cfg(feature = "css-inlining")]
pub mod inline;
pub mod migrate;
pub mod plaintext;
#[cfg(feature = "templating")]
pub mod templating;
pub mod validate;

use regex::Regex;
use scraper::{Html, Selector};

use components::transform_component;
pub use config::{ComponentNames, Config, OutputMode};

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
        // Step 0a: Protect template merge tags from html5ever mangling
        let (merge_tags, html) = protect_merge_tags(html);

        // Step 0b: Pre-process <image> tags (html5ever converts <image> to <img>)
        let html = preprocess_image_tags(&html);

        // Step 1: Extract <raw> blocks and replace with placeholders
        let (raws, working_html) = extract_raws(&html);

        // Step 1b: Preserve <td> content inside <block-grid> from html5ever stripping
        let working_html =
            preserve_block_grid_tds(&working_html, &self.config.components.block_grid);

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
                    if *tag == self.config.components.center || *tag == self.config.components.video
                    {
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
            if tag_name == self.config.components.columns || tag_name == "columns" {
                let replaced = transform_all_columns(&current, &self.config, &tag_name);
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

        // Step 3: Add float-center to .menu-item elements inside <center> tags
        current = add_float_center_to_centered_menu_items(&current);

        // Step 4: Restore protected block-grid <td> tags
        current = restore_block_grid_tds(&current);

        // Step 5: Remove data-parsed attributes (both forms: with and without ="")
        current = current.replace(" data-parsed=\"\"", "");
        current = current.replace(" data-parsed", "");

        // Step 6: Re-inject raw blocks
        let current = re_inject_raws(&current, &raws);

        // Step 7: Restore protected merge tags
        restore_merge_tags(&current, &merge_tags)
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

    /// Pre-process includes, then transform.
    pub fn transform_with_includes(
        &self,
        html: &str,
        base_path: &std::path::Path,
    ) -> Result<String, String> {
        let resolved = include::process_includes(html, base_path)?;
        Ok(self.transform(&resolved))
    }

    /// Pre-process includes, transform, then inline CSS.
    #[cfg(feature = "css-inlining")]
    pub fn transform_and_inline_with_includes(
        &self,
        html: &str,
        include_base: &std::path::Path,
        css_base: Option<&std::path::Path>,
    ) -> Result<String, String> {
        let resolved = include::process_includes(html, include_base)?;
        self.transform_and_inline(&resolved, css_base)
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
fn transform_all_columns(html: &str, config: &Config, actual_tag: &str) -> String {
    let tag = actual_tag;
    let escaped = regex::escape(tag);

    let open_re = Regex::new(&format!(r"<{}(?:\s[^>]*)?>", escaped)).unwrap();
    let close_re = Regex::new(&format!(r"</{}>", escaped)).unwrap();

    // Find the first top-level <columns> opening tag
    let first_open = match open_re.find(html) {
        Some(m) => m,
        None => return html.to_string(),
    };

    // Find all top-level column spans (with depth tracking for nested columns)
    let mut columns: Vec<(usize, usize)> = Vec::new(); // (start, end) of each column
    let mut search_start = first_open.start();

    while let Some(open_match) = open_re.find(&html[search_start..]) {
        let col_start = search_start + open_match.start();
        let mut pos = search_start + open_match.end();
        let mut depth = 1;

        // Track depth to find the matching close tag
        loop {
            let next_open = open_re
                .find(&html[pos..])
                .map(|m| (pos + m.start(), pos + m.end()));
            let next_close = close_re
                .find(&html[pos..])
                .map(|m| (pos + m.start(), pos + m.end()));

            match (next_open, next_close) {
                (Some((os, oe)), Some((cs, ce))) => {
                    if cs < os {
                        depth -= 1;
                        if depth == 0 {
                            columns.push((col_start, ce));
                            search_start = ce;
                            break;
                        }
                        pos = ce;
                    } else {
                        depth += 1;
                        pos = oe;
                    }
                }
                (None, Some((_cs, ce))) => {
                    depth -= 1;
                    if depth == 0 {
                        columns.push((col_start, ce));
                        search_start = ce;
                        break;
                    }
                    pos = ce;
                }
                _ => {
                    // No matching close tag
                    search_start = pos;
                    break;
                }
            }
        }

        // Check if the next non-whitespace content after this column is another <columns>
        let after = &html[search_start..];
        let trimmed = after.trim_start();
        if !trimmed.starts_with(&format!("<{}", tag)) {
            break; // No more adjacent columns
        }
    }

    if columns.is_empty() {
        return html.to_string();
    }

    let col_count = columns.len() as u32;
    let group_start = columns[0].0;
    let group_end = columns[columns.len() - 1].1;

    // Transform each column with position info
    let mut result = String::new();
    let mut prev_end = group_start;

    for (i, &(start, end)) in columns.iter().enumerate() {
        // Preserve whitespace between columns
        if start > prev_end {
            result.push_str(&html[prev_end..start]);
        }

        let col_html = &html[start..end];
        let is_first = i == 0;
        let is_last = i == columns.len() - 1;

        let doc = Html::parse_fragment(col_html);
        let sel = Selector::parse(tag).unwrap();
        if let Some(element) = doc.select(&sel).next() {
            let transformed = components::transform_column_with_position(
                &element, config, col_count, is_first, is_last,
            );
            result.push_str(&transformed);
        } else {
            result.push_str(col_html);
        }

        prev_end = end;
    }

    format!("{}{}{}", &html[..group_start], result, &html[group_end..])
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

        // For <center> and <video>, skip tags that already have data-parsed
        if (tag_name == "center" || tag_name == "video") && open_str.contains("data-parsed") {
            continue;
        }

        // Track nesting depth to find the matching close tag
        let mut depth = 1;
        let mut pos = open_end;

        loop {
            // Find next opening or closing tag after current position
            let next_open = open_re
                .find(&html[pos..])
                .map(|m| (pos + m.start(), pos + m.end()));
            let next_close = close_re
                .find(&html[pos..])
                .map(|m| (pos + m.start(), pos + m.end()));

            match (next_open, next_close) {
                (Some((os, oe)), Some((cs, ce))) => {
                    if cs < os {
                        // Close tag comes first
                        depth -= 1;
                        if depth == 0 {
                            return format!(
                                "{}{}{}",
                                &html[..open_start],
                                replacement,
                                &html[ce..]
                            );
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
        return format!(
            "{}{}{}",
            &html[..open_start],
            replacement,
            &html[open_end..]
        );
    }

    html.to_string()
}

/// Add float-center class to .menu-item elements inside <center> tags.
/// This matches the JS behavior: element.find('item, .menu-item').addClass('float-center')
/// We do this as a post-processing step because <center> is transformed before <item>,
/// so at center-transform time, the menu items haven't been converted to .menu-item yet.
fn add_float_center_to_centered_menu_items(html: &str) -> String {
    let center_re = Regex::new(r"(?s)<center[^>]*>(.*?)</center>").unwrap();
    let menu_item_re = Regex::new(r#"(<th\s[^>]*class=")menu-item(")"#).unwrap();

    center_re
        .replace_all(html, |caps: &regex::Captures| {
            let inner = &caps[1];
            let updated = menu_item_re.replace_all(inner, |mcaps: &regex::Captures| {
                format!("{}menu-item float-center{}", &mcaps[1], &mcaps[2])
            });
            format!(
                "<center{}>{}</center>",
                // Preserve any attributes on the center tag
                &caps[0][7..caps[0].find('>').unwrap()],
                updated
            )
        })
        .to_string()
}

/// Preserve <td> content inside <block-grid> tags from being stripped by html5ever.
/// html5ever removes <td> elements that appear outside a <table> context.
/// We wrap the inner content in a placeholder that html5ever won't touch,
/// then unwrap it after the block-grid is transformed into a proper table.
fn preserve_block_grid_tds(html: &str, block_grid_tag: &str) -> String {
    let escaped = regex::escape(block_grid_tag);
    let re = Regex::new(&format!(
        r"(?s)(<{e}(?:\s[^>]*)?>)(.*?)(</{e}>)",
        e = escaped
    ))
    .unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let open = &caps[1];
        let inner = &caps[2];
        let close = &caps[3];
        // Wrap each <td>...</td> in a raw placeholder to protect from html5ever
        let protected = inner
            .replace("<td>", "###BGTD###")
            .replace("</td>", "###/BGTD###");
        format!("{}{}{}", open, protected, close)
    })
    .to_string()
}

/// Restore <td> tags that were protected from html5ever stripping.
fn restore_block_grid_tds(html: &str) -> String {
    html.replace("###BGTD###", "<td>")
        .replace("###/BGTD###", "</td>")
}

/// Pre-process `<image>` tags into their final HTML output.
/// html5ever converts `<image>` to `<img>` per the HTML5 spec,
/// so we handle this before parsing.
fn preprocess_image_tags(html: &str) -> String {
    let re = Regex::new(r#"(?i)<image\s+([^>]*?)(/?\s*)>"#).unwrap();
    let attr_re =
        Regex::new(r#"(\w[\w-]*)(?:\s*=\s*"([^"]*)"|\s*=\s*'([^']*)'|\s*=\s*(\S+))?"#).unwrap();

    re.replace_all(html, |caps: &regex::Captures| {
        let attrs_str = &caps[1];
        let mut src = String::new();
        let mut alt = String::new();
        let mut width: Option<String> = None;
        let mut retina = false;
        let mut classes = Vec::new();

        for attr_cap in attr_re.captures_iter(attrs_str) {
            let name = &attr_cap[1];
            let value = attr_cap
                .get(2)
                .or(attr_cap.get(3))
                .or(attr_cap.get(4))
                .map(|m| m.as_str().to_string());

            match name.to_lowercase().as_str() {
                "src" => src = value.unwrap_or_default(),
                "alt" => alt = value.unwrap_or_default(),
                "width" => width = value,
                "retina" => retina = true,
                "class" => {
                    if let Some(v) = value {
                        classes.push(v);
                    }
                }
                _ => {}
            }
        }

        // For retina, display at half the source width
        let display_width = if retina {
            width
                .as_ref()
                .and_then(|w| w.parse::<u32>().ok())
                .map(|w| (w / 2).to_string())
        } else {
            width.clone()
        };

        let mut parts = Vec::new();
        parts.push(format!(r#"src="{}""#, src));
        parts.push(format!(r#"alt="{}""#, alt));
        if let Some(w) = &display_width {
            parts.push(format!(r#"width="{}""#, w));
            parts.push(format!(r#"style="width: {}px; max-width: 100%;""#, w));
        } else {
            parts.push(r#"style="max-width: 100%;""#.to_string());
        }
        if !classes.is_empty() {
            parts.push(format!(r#"class="{}""#, classes.join(" ")));
        }

        format!("<img {}>", parts.join(" "))
    })
    .to_string()
}

/// Protect template merge tags that look like HTML (ERB/EJS/ASP tags) from html5ever.
/// Tags like `<%= expr %>`, `<% code %>`, and `{% tag %}` get HTML-encoded by the parser.
/// We replace them with placeholders and restore after transformation.
fn protect_merge_tags(html: &str) -> (Vec<String>, String) {
    // Match ERB/EJS/ASP-style tags: <%= ... %>, <% ... %>, <%- ... %>, <%# ... %>
    // Also match Jinja2/Twig/Nunjucks tags: {% ... %}, {%- ... %}
    let re = Regex::new(r"(<%[=#-]?.*?%>|\{%-?.*?-?%\})").unwrap();
    let mut tags = Vec::new();
    let mut result = html.to_string();

    while let Some(m) = re.find(&result) {
        let tag = m.as_str().to_string();
        let placeholder = format!("###MERGE{}###", tags.len());
        result = format!(
            "{}{}{}",
            &result[..m.start()],
            placeholder,
            &result[m.end()..]
        );
        tags.push(tag);
    }

    (tags, result)
}

/// Restore protected merge tags from placeholders.
fn restore_merge_tags(html: &str, tags: &[String]) -> String {
    let mut result = html.to_string();
    for (i, tag) in tags.iter().enumerate() {
        let placeholder = format!("###MERGE{}###", i);
        result = result.replace(&placeholder, tag);
    }
    result
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
        assert!(result.contains("class=\"button\""));
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
    fn test_transform_divider() {
        let input = "<divider></divider>";
        let result = transform(input);
        assert!(result.contains("class=\"divider\""));
        assert!(result.contains("<tbody>"));
    }

    #[test]
    fn test_transform_spacer() {
        let input = "<spacer height=\"10\"></spacer>";
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

    #[test]
    fn test_merge_tags_erb() {
        let input = "<button href=\"<%= url %>\">Click</button>";
        let result = transform(input);
        assert!(result.contains("<%= url %>"));
    }

    #[test]
    fn test_merge_tags_jinja() {
        let input = "<button href=\"{% url 'home' %}\">Click</button>";
        let result = transform(input);
        assert!(result.contains("{% url 'home' %}"));
    }

    // --- protect_merge_tags / restore_merge_tags ---

    #[test]
    fn test_protect_merge_tags_erb() {
        let input = "Hello <%= name %> world";
        let (tags, result) = protect_merge_tags(input);
        assert_eq!(tags, vec!["<%= name %>"]);
        assert_eq!(result, "Hello ###MERGE0### world");
    }

    #[test]
    fn test_protect_merge_tags_multiple() {
        let input = "<%= first %> and <% second %> and {% third %}";
        let (tags, result) = protect_merge_tags(input);
        assert_eq!(tags.len(), 3);
        assert!(result.contains("###MERGE0###"));
        assert!(result.contains("###MERGE1###"));
        assert!(result.contains("###MERGE2###"));
    }

    #[test]
    fn test_restore_merge_tags_roundtrip() {
        let input = "Hello <%= name %>, your order is {% order_id %}.";
        let (tags, protected) = protect_merge_tags(input);
        let restored = restore_merge_tags(&protected, &tags);
        assert_eq!(restored, input);
    }

    #[test]
    fn test_protect_merge_tags_none() {
        let input = "No merge tags here";
        let (tags, result) = protect_merge_tags(input);
        assert!(tags.is_empty());
        assert_eq!(result, input);
    }

    // --- extract_raws / re_inject_raws ---

    #[test]
    fn test_extract_multiple_raws() {
        let input = "a<raw>first</raw>b<raw>second</raw>c";
        let (raws, result) = extract_raws(input);
        assert_eq!(raws, vec!["first", "second"]);
        assert!(result.contains("###RAW0###"));
        assert!(result.contains("###RAW1###"));
        let restored = re_inject_raws(&result, &raws);
        assert!(restored.contains("first"));
        assert!(restored.contains("second"));
    }

    #[test]
    fn test_extract_raws_with_html_content() {
        let input = "<raw><table><tr><td>Keep</td></tr></table></raw>";
        let (raws, result) = extract_raws(input);
        assert_eq!(raws.len(), 1);
        assert!(raws[0].contains("<table>"));
        assert!(!result.contains("<table>"));
        let restored = re_inject_raws(&result, &raws);
        assert!(restored.contains("<table><tr><td>Keep</td></tr></table>"));
    }

    // --- replace_first_tag ---

    #[test]
    fn test_replace_first_tag_simple() {
        let html = "<button>Click</button>";
        let result = replace_first_tag(html, "button", "REPLACED");
        assert_eq!(result, "REPLACED");
    }

    #[test]
    fn test_replace_first_tag_with_surrounding() {
        let html = "before<button>Click</button>after";
        let result = replace_first_tag(html, "button", "REPLACED");
        assert_eq!(result, "beforeREPLACEDafter");
    }

    #[test]
    fn test_replace_first_tag_nested() {
        // Should replace the outermost tag, not be confused by nested same-name tags
        let html = "<div><div>inner</div></div>trailing";
        let result = replace_first_tag(html, "div", "REPLACED");
        assert_eq!(result, "REPLACEDtrailing");
    }

    #[test]
    fn test_replace_first_tag_self_closing() {
        let html = "before<divider />after";
        let result = replace_first_tag(html, "divider", "REPLACED");
        assert_eq!(result, "beforeREPLACEDafter");
    }

    #[test]
    fn test_replace_first_tag_with_attributes() {
        let html = r#"before<button href="http://example.com" class="big">Click</button>after"#;
        let result = replace_first_tag(html, "button", "REPLACED");
        assert_eq!(result, "beforeREPLACEDafter");
    }

    #[test]
    fn test_replace_first_tag_no_match() {
        let html = "<div>content</div>";
        let result = replace_first_tag(html, "button", "REPLACED");
        assert_eq!(result, html);
    }

    #[test]
    fn test_replace_first_tag_only_first() {
        let html = "<spacer></spacer><spacer></spacer>";
        let result = replace_first_tag(html, "spacer", "X");
        assert_eq!(result, "X<spacer></spacer>");
    }

    // --- transform_all_columns ---

    #[test]
    fn test_transform_all_columns_single() {
        let html = "<row><column>Content</column></row>";
        let config = Config::default();
        let result = transform_all_columns(html, &config, "column");
        // Should transform the column into table markup
        assert!(result.contains("class=\""));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_transform_all_columns_multiple_adjacent() {
        let html = "<row><column>First</column><column>Second</column></row>";
        let config = Config::default();
        let result = transform_all_columns(html, &config, "column");
        assert!(result.contains("First"));
        assert!(result.contains("Second"));
        // Both columns should be transformed (no raw <column> tags left)
        assert!(!result.contains("<column>"));
    }

    #[test]
    fn test_transform_all_columns_no_columns() {
        let html = "<row><div>No columns</div></row>";
        let config = Config::default();
        let result = transform_all_columns(html, &config, "column");
        assert_eq!(result, html);
    }

    // --- preserve_block_grid_tds / restore_block_grid_tds ---

    #[test]
    fn test_preserve_block_grid_tds() {
        let html = "<block-grid><td>Item 1</td><td>Item 2</td></block-grid>";
        let result = preserve_block_grid_tds(html, "block-grid");
        assert!(result.contains("###BGTD###"));
        assert!(result.contains("###/BGTD###"));
        assert!(!result.contains("<td>"));
    }

    #[test]
    fn test_restore_block_grid_tds() {
        let html = "###BGTD###Item###/BGTD###";
        let result = restore_block_grid_tds(html);
        assert_eq!(result, "<td>Item</td>");
    }

    #[test]
    fn test_block_grid_td_roundtrip() {
        let html = "<block-grid><td>A</td><td>B</td></block-grid>";
        let preserved = preserve_block_grid_tds(html, "block-grid");
        let restored = restore_block_grid_tds(&preserved);
        assert_eq!(restored, html);
    }

    // --- preprocess_image_tags ---

    #[test]
    fn test_preprocess_image_basic() {
        let html = r#"<image src="photo.jpg" alt="A photo" width="600">"#;
        let result = preprocess_image_tags(html);
        assert!(result.contains("<img "));
        assert!(result.contains(r#"src="photo.jpg""#));
        assert!(result.contains(r#"alt="A photo""#));
        assert!(result.contains(r#"width="600""#));
    }

    #[test]
    fn test_preprocess_image_retina() {
        let html = r#"<image src="photo.jpg" alt="A photo" width="600" retina>"#;
        let result = preprocess_image_tags(html);
        assert!(result.contains(r#"width="300""#));
    }

    #[test]
    fn test_preprocess_image_with_class() {
        let html = r#"<image src="photo.jpg" alt="" class="hero-img">"#;
        let result = preprocess_image_tags(html);
        assert!(result.contains(r#"class="hero-img""#));
    }

    #[test]
    fn test_preprocess_image_no_width() {
        let html = r#"<image src="photo.jpg" alt="test">"#;
        let result = preprocess_image_tags(html);
        assert!(result.contains(r#"style="max-width: 100%;""#));
        assert!(!result.contains("width="));
    }

    // --- add_float_center_to_centered_menu_items ---

    #[test]
    fn test_float_center_added_to_menu_item_in_center() {
        let html = r#"<center><th class="menu-item">Item</th></center>"#;
        let result = add_float_center_to_centered_menu_items(html);
        assert!(result.contains("menu-item float-center"));
    }

    #[test]
    fn test_float_center_not_added_outside_center() {
        let html = r#"<th class="menu-item">Item</th>"#;
        let result = add_float_center_to_centered_menu_items(html);
        assert!(!result.contains("float-center"));
    }

    // --- Full pipeline integration for columns ---

    #[test]
    fn test_full_transform_two_equal_columns() {
        let input = "<row><column>Left</column><column>Right</column></row>";
        let result = transform(input);
        assert!(result.contains("Left"));
        assert!(result.contains("Right"));
        assert!(result.contains("small-12"));
        assert!(result.contains("large-6"));
    }

    #[test]
    fn test_full_transform_three_columns() {
        let input = "<row><column>A</column><column>B</column><column>C</column></row>";
        let result = transform(input);
        assert!(result.contains("large-4"));
        assert!(result.contains("A"));
        assert!(result.contains("B"));
        assert!(result.contains("C"));
    }

    #[test]
    fn test_full_transform_column_with_sizes() {
        let input = r#"<row><column sm="6" lg="8">Wide</column><column sm="6" lg="4">Narrow</column></row>"#;
        let result = transform(input);
        assert!(result.contains("small-6"));
        assert!(result.contains("large-8"));
        assert!(result.contains("large-4"));
    }
}

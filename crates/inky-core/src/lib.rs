pub mod attrs;
pub mod color;
pub mod components;
pub mod config;
pub mod include;
#[cfg(feature = "css-inlining")]
pub mod inline;
pub mod migrate;
pub mod plaintext;
mod render;
#[cfg(feature = "templating")]
pub mod templating;
pub mod validate;

use std::sync::LazyLock;

use regex::Regex;

static RE_MERGE_TAGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(<%[=#-]?.*?%>|\{%-?.*?-?%\})").unwrap());
static RE_RAW_BLOCKS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)(?:\n *)?< *raw *>(.*?)</ *raw *>(?: *\n)?").unwrap());

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
        // Protect template merge tags from html5ever mangling
        let (merge_tags, html) = protect_merge_tags(html);

        // Extract <raw> blocks BEFORE any other preprocessing so raw
        // content (including <image> tags) is truly untouched
        let (raws, html) = extract_raws(&html);

        // Pre-process <image> tags (html5ever converts <image> to <img>)
        let html = preprocess_image_tags(&html);

        // Preserve <td> content inside <block-grid> from html5ever stripping
        let html = preserve_block_grid_tds(&html, &self.config.components.block_grid);

        // Expand `<tag/>` to `<tag></tag>`: HTML parsing ignores the
        // self-closing slash on non-void elements, which would make the
        // component swallow everything after it
        let html = expand_self_closing_components(&html, &self.config);

        // Single parse + bottom-up render
        let current = render::render(&html, &self.config);

        // Restore protected content
        let current = restore_block_grid_tds(&current);
        let current = re_inject_raws(&current, &raws);
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

/// Expand self-closing component tags (`<spacer/>` → `<spacer></spacer>`).
/// The HTML5 parser ignores `/>` on non-void elements, so without this the
/// element would stay open and swallow all following content.
fn expand_self_closing_components(html: &str, config: &Config) -> String {
    let mut tags = config.components.all_tags();
    // Longest first so "columns" wins over "column" in the alternation
    tags.sort_by_key(|t| std::cmp::Reverse(t.len()));
    let alternation = tags
        .iter()
        .map(|t| regex::escape(t))
        .collect::<Vec<_>>()
        .join("|");
    let re = Regex::new(&format!(r"(?i)<({})((?:\s[^>]*?)?)\s*/>", alternation)).unwrap();
    re.replace_all(html, "<$1$2></$1>").to_string()
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
            .replace("<td>", "###bgtd###")
            .replace("</td>", "###/bgtd###");
        format!("{}{}{}", open, protected, close)
    })
    .to_string()
}

/// Restore <td> tags that were protected from html5ever stripping.
fn restore_block_grid_tds(html: &str) -> String {
    html.replace("###bgtd###", "<td>")
        .replace("###/bgtd###", "</td>")
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

/// Build the placeholder token for index `i`. When `comment` is true the token
/// is wrapped in an HTML comment (`<!--###raw0###-->`) so it survives contexts
/// where bare text is illegal — inside `<table>`/`<tbody>`/`<tr>` or `<head>`,
/// HTML5 parsing foster-parents stray text out, which would relocate restored
/// content. Comments are legal in those positions and round-trip verbatim.
fn make_placeholder(prefix: &str, i: usize, comment: bool) -> String {
    if comment {
        format!("<!--###{}{}###-->", prefix, i)
    } else {
        format!("###{}{}###", prefix, i)
    }
}

/// Extract matches from HTML, replacing them with numbered placeholders.
/// `capture_group` selects which regex group to save (0 = whole match, 1+ = sub-group).
/// When `comment` is true the placeholder is wrapped as an HTML comment.
/// Returns the saved content and the modified HTML.
fn extract_with_placeholders(
    html: &str,
    re: &Regex,
    prefix: &str,
    capture_group: usize,
    comment: bool,
) -> (Vec<String>, String) {
    let mut saved = Vec::new();
    let mut result = html.to_string();

    while let Some(caps) = re.captures(&result) {
        let full = caps.get(0).unwrap();
        let content = caps.get(capture_group).unwrap_or(full).as_str().to_string();
        let placeholder = make_placeholder(prefix, saved.len(), comment);
        result = format!(
            "{}{}{}",
            &result[..full.start()],
            placeholder,
            &result[full.end()..]
        );
        saved.push(content);
    }

    (saved, result)
}

/// Restore placeholders with saved content. `comment` must match the value used
/// when the placeholders were created so the full token (including any `<!--`
/// `-->` delimiters) is replaced.
fn restore_placeholders(html: &str, saved: &[String], prefix: &str, comment: bool) -> String {
    let mut result = html.to_string();
    for (i, content) in saved.iter().enumerate() {
        let placeholder = make_placeholder(prefix, i, comment);
        result = result.replace(&placeholder, content);
    }
    result
}

/// Protect template merge tags that look like HTML (ERB/EJS/ASP tags) from html5ever.
fn protect_merge_tags(html: &str) -> (Vec<String>, String) {
    // Merge tags can appear in attribute-value position, where a comment form
    // would be invalid, so they stay as bare-text placeholders.
    extract_with_placeholders(html, &RE_MERGE_TAGS, "merge", 0, false)
}

/// Restore protected merge tags from placeholders.
fn restore_merge_tags(html: &str, tags: &[String]) -> String {
    restore_placeholders(html, tags, "merge", false)
}

/// Extract `<raw>` blocks from HTML, replacing them with placeholders.
/// Raw placeholders use the comment form so they survive table/head contexts.
fn extract_raws(html: &str) -> (Vec<String>, String) {
    extract_with_placeholders(html, &RE_RAW_BLOCKS, "raw", 1, true)
}

/// Re-inject raw block content back into placeholders.
fn re_inject_raws(html: &str, raws: &[String]) -> String {
    restore_placeholders(html, raws, "raw", true)
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
        assert_eq!(result, "before<!--###raw0###-->after");
    }

    #[test]
    fn test_re_inject_raws() {
        let html = "before<!--###raw0###-->after";
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
        assert!(!result.contains("###raw"));
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
        assert_eq!(result, "Hello ###merge0### world");
    }

    #[test]
    fn test_protect_merge_tags_multiple() {
        let input = "<%= first %> and <% second %> and {% third %}";
        let (tags, result) = protect_merge_tags(input);
        assert_eq!(tags.len(), 3);
        assert!(result.contains("###merge0###"));
        assert!(result.contains("###merge1###"));
        assert!(result.contains("###merge2###"));
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
        assert!(result.contains("<!--###raw0###-->"));
        assert!(result.contains("<!--###raw1###-->"));
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

    // --- preserve_block_grid_tds / restore_block_grid_tds ---

    #[test]
    fn test_preserve_block_grid_tds() {
        let html = "<block-grid><td>Item 1</td><td>Item 2</td></block-grid>";
        let result = preserve_block_grid_tds(html, "block-grid");
        assert!(result.contains("###bgtd###"));
        assert!(result.contains("###/bgtd###"));
        assert!(!result.contains("<td>"));
    }

    #[test]
    fn test_restore_block_grid_tds() {
        let html = "###bgtd###Item###/bgtd###";
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

    // --- Phase 2 regression tests: bugs in the old string-replacement engine ---

    #[test]
    fn self_closing_component_does_not_swallow_content() {
        let result = transform(r#"a<spacer height="10"/>b"#);
        assert!(result.contains("a"));
        assert!(result.contains("font-size:10px"));
        assert!(result.ends_with('b'), "content after self-closing tag lost: {result}");
    }

    #[test]
    fn capitalized_tag_does_not_halt_transformation() {
        let result = transform(r#"<Button href="https://x.dev">Go</Button><row>r</row>"#);
        assert!(result.contains(r#"class="button""#));
        assert!(result.contains(r#"class="row""#));
    }

    #[test]
    fn component_tag_in_attribute_value_untouched() {
        let result = transform(r#"<p title="see <button>">x</p>"#);
        assert_eq!(result, r#"<p title="see <button>">x</p>"#);
    }

    #[test]
    fn comment_between_columns_keeps_grid_math() {
        let result = transform("<row><column>A</column><!-- note --><column>B</column></row>");
        assert_eq!(result.matches("large-6").count(), 2);
    }

    #[test]
    fn components_inside_outlook_are_transformed() {
        let result = transform(r##"<outlook><button href="#">B</button></outlook>"##);
        assert!(result.contains("<!--[if mso]>"));
        assert!(result.contains(r#"class="button""#));
    }

    #[test]
    fn raw_protects_image_tags() {
        let result = transform(r#"<raw><image src="x.png"></raw>"#);
        assert!(result.contains(r#"<image src="x.png">"#));
        assert!(!result.contains("<img"));
    }

    #[test]
    fn data_parsed_like_attributes_survive() {
        let result = transform(r#"<p data-parsed-mode="strict" data-parsed="x">y</p>"#);
        assert!(result.contains(r#"data-parsed-mode="strict""#));
        assert!(result.contains(r#"data-parsed="x""#));
    }

    #[test]
    fn full_document_with_doctype_transforms_body() {
        let input = r#"<!DOCTYPE html><html><head><title>T</title></head><body><button href="https://x.dev">Go</button></body></html>"#;
        let result = transform(input);
        assert!(result.starts_with("<!DOCTYPE html>"));
        assert!(result.contains("<head><title>T</title></head>"));
        assert!(result.contains(r#"class="button""#));
        assert!(!result.contains("<button "));
    }

    #[test]
    fn merge_tag_as_attribute_survives() {
        let result = transform("<row <%= extra %>>c</row>");
        assert!(result.contains("<%= extra %>"), "merge tag lost: {result}");
    }

    #[test]
    fn raw_preserves_table_rows_in_place() {
        let result = transform("<table><tbody><raw><tr><td><%= x %></td></tr></raw></tbody></table>");
        assert!(result.contains("<table><tbody><tr><td><%= x %></td></tr></tbody></table>"), "rows relocated: {result}");
    }

    #[test]
    fn raw_stays_inside_head() {
        let input = "<!DOCTYPE html><html><head><raw><style>.a{color:red}</style></raw></head><body><p>x</p></body></html>";
        let result = transform(input);
        let head_end = result.find("</head>").unwrap();
        let style_pos = result.find("<style>").unwrap();
        assert!(style_pos < head_end, "raw content moved out of head: {result}");
    }

    #[test]
    fn deeply_nested_input_does_not_overflow_stack() {
        let mut input = String::new();
        for _ in 0..10_000 { input.push_str("<div>"); }
        input.push('x');
        for _ in 0..10_000 { input.push_str("</div>"); }
        let result = transform(&input);
        assert!(result.contains('x'));
    }
}

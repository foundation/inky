use scraper::{Html, Node};

/// Convert HTML email output to plain text suitable for multipart email.
pub fn html_to_plain_text(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut ctx = Context::default();
    process_node(document.root_element().id(), &document, &mut ctx);
    let raw = ctx.output;
    let wrapped = word_wrap(&raw, 78);
    collapse_blank_lines(&wrapped)
}

#[derive(Default)]
struct Context {
    output: String,
    ol_counter: Vec<usize>,
    in_pre: bool,
}

impl Context {
    fn push(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn ensure_blank_line(&mut self) {
        let trimmed_end = self.output.trim_end_matches(' ');
        if trimmed_end.is_empty() {
            return;
        }
        if !trimmed_end.ends_with("\n\n") {
            if trimmed_end.ends_with('\n') {
                self.output.push('\n');
            } else {
                self.output.push_str("\n\n");
            }
        }
    }

    fn ensure_newline(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }
}

fn process_node(node_id: ego_tree::NodeId, doc: &Html, ctx: &mut Context) {
    let node = doc.tree.get(node_id).unwrap();
    match node.value() {
        Node::Text(text) => {
            if ctx.in_pre {
                ctx.push(text);
            } else {
                // Collapse whitespace
                let collapsed = collapse_whitespace(text);
                if !collapsed.is_empty() {
                    ctx.push(&collapsed);
                }
            }
        }
        Node::Element(el) => {
            let tag = el.name();
            // Skip invisible elements entirely
            if matches!(tag, "style" | "script" | "head") {
                return;
            }

            match tag {
                "br" => {
                    ctx.push("\n");
                }
                "hr" => {
                    ctx.ensure_blank_line();
                    ctx.push("----------------------------------------\n\n");
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    ctx.ensure_blank_line();
                    let text = extract_text(node_id, doc);
                    let upper = text.trim().to_uppercase();
                    ctx.push(&upper);
                    ctx.push("\n");
                    if tag == "h1" {
                        let underline = "=".repeat(upper.len());
                        ctx.push(&underline);
                        ctx.push("\n");
                    } else if tag == "h2" {
                        let underline = "-".repeat(upper.len());
                        ctx.push(&underline);
                        ctx.push("\n");
                    }
                    ctx.push("\n");
                }
                "p" => {
                    ctx.ensure_blank_line();
                    process_children(node_id, doc, ctx);
                    ctx.ensure_blank_line();
                }
                "a" => {
                    let href = el.attr("href").unwrap_or("");
                    let text = extract_text(node_id, doc);
                    let text = text.trim();
                    if text.is_empty() {
                        if !href.is_empty() {
                            ctx.push(href);
                        }
                    } else if href.is_empty() || href == text {
                        ctx.push(text);
                    } else {
                        ctx.push(&format!("{} ({})", text, href));
                    }
                }
                "img" => {
                    let alt = el.attr("alt").unwrap_or("");
                    if !alt.is_empty() {
                        ctx.push(&format!("[{}]", alt));
                    }
                }
                "ul" => {
                    ctx.ensure_blank_line();
                    process_children(node_id, doc, ctx);
                    ctx.ensure_newline();
                }
                "ol" => {
                    ctx.ensure_blank_line();
                    ctx.ol_counter.push(0);
                    process_children(node_id, doc, ctx);
                    ctx.ol_counter.pop();
                    ctx.ensure_newline();
                }
                "li" => {
                    ctx.ensure_newline();
                    if let Some(counter) = ctx.ol_counter.last_mut() {
                        *counter += 1;
                        let num = *counter;
                        ctx.push(&format!("  {}. ", num));
                    } else {
                        ctx.push("  * ");
                    }
                    process_children(node_id, doc, ctx);
                }
                "blockquote" => {
                    ctx.ensure_blank_line();
                    let inner = {
                        let mut inner_ctx = Context {
                            ol_counter: Vec::new(),
                            ..Default::default()
                        };
                        process_children(node_id, doc, &mut inner_ctx);
                        inner_ctx.output
                    };
                    for line in inner.trim().lines() {
                        ctx.push(&format!("> {}\n", line));
                    }
                    ctx.push("\n");
                }
                "pre" | "code" => {
                    let was_pre = ctx.in_pre;
                    ctx.in_pre = true;
                    process_children(node_id, doc, ctx);
                    ctx.in_pre = was_pre;
                }
                "table" => {
                    // For Inky structural tables, just extract content
                    process_children(node_id, doc, ctx);
                }
                "div" | "td" | "th" | "tr" | "tbody" | "thead" | "tfoot" | "span" | "center"
                | "section" | "article" | "main" | "header" | "footer" | "nav" | "aside"
                | "figure" | "figcaption" | "details" | "summary" | "html" | "body" => {
                    process_children(node_id, doc, ctx);
                }
                _ => {
                    // For any unknown tag, recurse into children
                    process_children(node_id, doc, ctx);
                }
            }
        }
        Node::Document => {
            process_children(node_id, doc, ctx);
        }
        _ => {}
    }
}

fn process_children(node_id: ego_tree::NodeId, doc: &Html, ctx: &mut Context) {
    let node = doc.tree.get(node_id).unwrap();
    for child in node.children() {
        process_node(child.id(), doc, ctx);
    }
}

/// Extract all text content from a node and its children (ignoring tags).
fn extract_text(node_id: ego_tree::NodeId, doc: &Html) -> String {
    let mut result = String::new();
    extract_text_inner(node_id, doc, &mut result);
    result
}

fn extract_text_inner(node_id: ego_tree::NodeId, doc: &Html, out: &mut String) {
    let node = doc.tree.get(node_id).unwrap();
    match node.value() {
        Node::Text(text) => {
            out.push_str(&collapse_whitespace(text));
        }
        Node::Element(_) | Node::Document => {
            for child in node.children() {
                extract_text_inner(child.id(), doc, out);
            }
        }
        _ => {}
    }
}

fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_ws = false;
    for ch in s.chars() {
        if ch.is_ascii_whitespace() {
            if !last_was_ws {
                result.push(' ');
                last_was_ws = true;
            }
        } else {
            result.push(ch);
            last_was_ws = false;
        }
    }
    result
}

/// Word-wrap text at the given width, preserving existing newlines.
/// Does not break URLs.
fn word_wrap(text: &str, width: usize) -> String {
    let mut result = String::with_capacity(text.len());
    for line in text.split('\n') {
        if line.len() <= width {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        // Don't wrap lines that contain a URL that would be broken
        if is_url_line(line) && !line.contains(' ') {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        wrap_line(line, width, &mut result);
        result.push('\n');
    }
    // Remove the trailing newline we added if original didn't have one
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

fn is_url_line(s: &str) -> bool {
    s.contains("http://") || s.contains("https://")
}

fn wrap_line(line: &str, width: usize, out: &mut String) {
    let words: Vec<&str> = line.split(' ').collect();
    let mut current_len = 0;
    let mut first = true;

    for word in words {
        if word.is_empty() {
            if first {
                first = false;
            } else if current_len < width {
                out.push(' ');
                current_len += 1;
            }
            continue;
        }

        let word_len = word.len();
        if first {
            out.push_str(word);
            current_len = word_len;
            first = false;
        } else if current_len + 1 + word_len <= width {
            out.push(' ');
            out.push_str(word);
            current_len += 1 + word_len;
        } else if is_url(word) {
            // Don't break URLs - put on new line if needed
            if current_len > 0 {
                out.push('\n');
            }
            out.push_str(word);
            current_len = word_len;
        } else {
            out.push('\n');
            out.push_str(word);
            current_len = word_len;
        }
    }
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || (s.starts_with('(') && (s.contains("http://") || s.contains("https://")))
}

/// Collapse runs of more than 2 consecutive newlines down to 2.
fn collapse_blank_lines(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut consecutive_newlines = 0;

    for ch in text.chars() {
        if ch == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                result.push(ch);
            }
        } else {
            consecutive_newlines = 0;
            result.push(ch);
        }
    }

    // Trim trailing whitespace
    let trimmed = result.trim_end();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{}\n", trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_paragraph() {
        let html = "<p>Hello, world!</p>";
        let result = html_to_plain_text(html);
        assert!(result.contains("Hello, world!"));
    }

    #[test]
    fn test_multiple_paragraphs() {
        let html = "<p>First paragraph.</p><p>Second paragraph.</p>";
        let result = html_to_plain_text(html);
        assert!(result.contains("First paragraph."));
        assert!(result.contains("Second paragraph."));
        // Should have blank line between paragraphs
        assert!(result.contains("\n\n"));
    }

    #[test]
    fn test_heading_h1() {
        let html = "<h1>Main Title</h1>";
        let result = html_to_plain_text(html);
        assert!(result.contains("MAIN TITLE"));
        assert!(result.contains("=========="));
    }

    #[test]
    fn test_heading_h2() {
        let html = "<h2>Subtitle</h2>";
        let result = html_to_plain_text(html);
        assert!(result.contains("SUBTITLE"));
        assert!(result.contains("--------"));
    }

    #[test]
    fn test_heading_h3() {
        let html = "<h3>Section</h3>";
        let result = html_to_plain_text(html);
        assert!(result.contains("SECTION"));
        // h3 should not have underline
        assert!(!result.contains("==="));
        assert!(!result.contains("---"));
    }

    #[test]
    fn test_link_with_different_text() {
        let html = r#"<a href="https://example.com">Click here</a>"#;
        let result = html_to_plain_text(html);
        assert!(result.contains("Click here (https://example.com)"));
    }

    #[test]
    fn test_link_text_matches_href() {
        let html = r#"<a href="https://example.com">https://example.com</a>"#;
        let result = html_to_plain_text(html);
        assert!(result.contains("https://example.com"));
        // Should NOT duplicate the URL
        assert!(!result.contains("(https://example.com)"));
    }

    #[test]
    fn test_unordered_list() {
        let html = "<ul><li>Apple</li><li>Banana</li><li>Cherry</li></ul>";
        let result = html_to_plain_text(html);
        assert!(result.contains("  * Apple"));
        assert!(result.contains("  * Banana"));
        assert!(result.contains("  * Cherry"));
    }

    #[test]
    fn test_ordered_list() {
        let html = "<ol><li>First</li><li>Second</li><li>Third</li></ol>";
        let result = html_to_plain_text(html);
        assert!(result.contains("  1. First"));
        assert!(result.contains("  2. Second"));
        assert!(result.contains("  3. Third"));
    }

    #[test]
    fn test_image_alt_text() {
        let html = r#"<img alt="A cute cat" src="cat.jpg">"#;
        let result = html_to_plain_text(html);
        assert!(result.contains("[A cute cat]"));
    }

    #[test]
    fn test_image_no_alt() {
        let html = r#"<img src="cat.jpg">"#;
        let result = html_to_plain_text(html);
        // Should not produce [] for empty alt
        assert!(!result.contains("[]"));
    }

    #[test]
    fn test_script_style_stripping() {
        let html = r#"<style>body { color: red; }</style><script>alert('hi');</script><p>Visible text</p>"#;
        let result = html_to_plain_text(html);
        assert!(!result.contains("color: red"));
        assert!(!result.contains("alert"));
        assert!(result.contains("Visible text"));
    }

    #[test]
    fn test_word_wrapping() {
        let long_text = format!(
            "<p>{}</p>",
            "word ".repeat(30) // ~150 chars
        );
        let result = html_to_plain_text(&long_text);
        for line in result.lines() {
            // Allow some tolerance for URLs and edge cases
            if !line.contains("http") {
                assert!(line.len() <= 80, "Line too long ({}): {}", line.len(), line);
            }
        }
    }

    #[test]
    fn test_horizontal_rule() {
        let html = "<p>Above</p><hr><p>Below</p>";
        let result = html_to_plain_text(html);
        assert!(result.contains("----------------------------------------"));
        assert!(result.contains("Above"));
        assert!(result.contains("Below"));
    }

    #[test]
    fn test_blockquote() {
        let html = "<blockquote><p>Quoted text here.</p></blockquote>";
        let result = html_to_plain_text(html);
        assert!(result.contains("> Quoted text here."));
    }

    #[test]
    fn test_nested_inky_tables() {
        let html = r#"
        <table class="container">
          <tbody>
            <tr>
              <td>
                <table class="row">
                  <tbody>
                    <tr>
                      <th class="columns">
                        <p>Hello from Inky!</p>
                      </th>
                    </tr>
                  </tbody>
                </table>
              </td>
            </tr>
          </tbody>
        </table>
        "#;
        let result = html_to_plain_text(html);
        assert!(result.contains("Hello from Inky!"));
        // Should not contain any HTML table artifacts
        assert!(!result.contains("<table"));
        assert!(!result.contains("<tr"));
        assert!(!result.contains("<td"));
    }

    #[test]
    fn test_br_tag() {
        let html = "<p>Line one<br>Line two</p>";
        let result = html_to_plain_text(html);
        assert!(result.contains("Line one\nLine two"));
    }

    #[test]
    fn test_collapse_blank_lines() {
        let input = "a\n\n\n\n\nb";
        let result = collapse_blank_lines(input);
        assert_eq!(result, "a\n\nb\n");
    }

    #[test]
    fn test_pre_preserves_whitespace() {
        let html = "<pre>  indented\n    more indent</pre>";
        let result = html_to_plain_text(html);
        assert!(result.contains("  indented"));
        assert!(result.contains("    more indent"));
    }

    #[test]
    fn test_empty_input() {
        let result = html_to_plain_text("");
        assert!(result.trim().is_empty());
    }

    #[test]
    fn test_head_stripped() {
        let html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>";
        let result = html_to_plain_text(html);
        assert!(!result.contains("Test"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_url_not_broken_by_wrap() {
        let html =
            "<p>Visit this link: <a href=\"https://example.com/very/long/path/that/should/not/break\">https://example.com/very/long/path/that/should/not/break</a></p>";
        let result = html_to_plain_text(html);
        // The URL should appear intact on a single line
        assert!(result.contains("https://example.com/very/long/path/that/should/not/break"));
    }
}

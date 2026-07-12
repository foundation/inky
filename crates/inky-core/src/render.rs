// `render()` is not yet called from `transform()` — that wiring lands in
// Task 5 — so this whole module is only reachable via its own unit tests,
// which trips clippy's dead-code lint on an ordinary (non-test) build.
// Remove this once Task 5 wires `render()` into `transform()`.
#![allow(dead_code)]

use std::sync::LazyLock;

use ego_tree::NodeRef;
use regex::Regex;
use scraper::{ElementRef, Html, Node};

use crate::components::{self, El, RenderCtx};
use crate::config::Config;

static RE_FULL_DOCUMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<!doctype\s|<html[\s>]").unwrap());

/// Elements with no closing tag.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "param", "source", "track", "wbr",
];

/// Elements whose text content is emitted verbatim (no entity escaping).
const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style", "xmp", "iframe", "noembed", "noframes"];

/// Render-walk state. `Copy` so recursion sites can tweak flags per child.
#[derive(Clone, Copy)]
struct Walk<'a> {
    config: &'a Config,
    /// True anywhere inside a <center> component.
    inside_center: bool,
    /// True only for direct element children of a <center> component.
    center_child: bool,
}

/// Parse `html` once and serialize it back, transforming component tags.
/// Fragment vs. full-document mode is auto-detected so doctype/head/body
/// round-trip for full emails while snippets stay unwrapped.
pub(crate) fn render(html: &str, config: &Config) -> String {
    let walk = Walk {
        config,
        inside_center: false,
        center_child: false,
    };
    let mut out = String::with_capacity(html.len() * 2);
    if RE_FULL_DOCUMENT.is_match(html) {
        let doc = Html::parse_document(html);
        for child in doc.tree.root().children() {
            render_node(child, walk, &mut out);
        }
    } else {
        let doc = Html::parse_fragment(html);
        // parse_fragment wraps content in an auto-created <html> element
        if let Some(root) = doc
            .tree
            .root()
            .children()
            .find(|n| n.value().is_element())
        {
            for child in root.children() {
                render_node(child, walk, &mut out);
            }
        }
    }
    out
}

fn render_node(node: NodeRef<Node>, walk: Walk, out: &mut String) {
    match node.value() {
        Node::Doctype(doctype) => {
            out.push_str("<!DOCTYPE ");
            out.push_str(&doctype.name);
            if !doctype.public_id.is_empty() {
                out.push_str(" PUBLIC \"");
                out.push_str(&doctype.public_id);
                out.push('"');
                if !doctype.system_id.is_empty() {
                    out.push_str(" \"");
                    out.push_str(&doctype.system_id);
                    out.push('"');
                }
            } else if !doctype.system_id.is_empty() {
                out.push_str(" SYSTEM \"");
                out.push_str(&doctype.system_id);
                out.push('"');
            }
            out.push('>');
        }
        Node::Comment(comment) => {
            out.push_str("<!--");
            out.push_str(comment);
            out.push_str("-->");
        }
        Node::Text(text) => out.push_str(&escape_text(text)),
        Node::Element(_) => render_element(node, walk, out),
        // Document/Fragment roots are handled by render(); PIs are dropped
        // (html5ever already parses them as comments in HTML documents).
        _ => {}
    }
}

fn render_element(node: NodeRef<Node>, walk: Walk, out: &mut String) {
    let element = ElementRef::wrap(node).expect("render_element called on non-element");
    let name = element.value().name();

    // Component dispatch: render children first (bottom-up), then let the
    // component wrap the finished inner HTML. Output is emitted verbatim
    // and never re-scanned.
    if is_component_tag(name, walk.config) {
        let inner = render_children_to_string(node, walk);
        let mut el = El::new(element, inner);
        if walk.center_child {
            el.extra_classes.push("float-center".to_string());
        }
        let ctx = RenderCtx {
            config: walk.config,
            inside_center: walk.inside_center,
        };
        if let Some(output) = components::transform_component(&el, &ctx) {
            out.push_str(&output);
            return;
        }
    }

    out.push('<');
    out.push_str(name);
    for (key, value) in element.value().attrs() {
        out.push(' ');
        out.push_str(key);
        if !value.is_empty() {
            out.push_str("=\"");
            out.push_str(&escape_attr(value));
            out.push('"');
        }
    }
    out.push('>');

    if VOID_ELEMENTS.contains(&name) {
        return;
    }

    if RAW_TEXT_ELEMENTS.contains(&name) {
        for child in node.children() {
            if let Some(text) = child.value().as_text() {
                out.push_str(text);
            }
        }
    } else {
        let child_walk = Walk {
            center_child: false,
            ..walk
        };
        for child in node.children() {
            render_node(child, child_walk, out);
        }
    }

    out.push_str("</");
    out.push_str(name);
    out.push('>');
}

/// Render all children of `node` into a fresh string (component inner HTML).
fn render_children_to_string(node: NodeRef<Node>, walk: Walk) -> String {
    let child_walk = Walk {
        center_child: false,
        ..walk
    };
    let mut out = String::new();
    for child in node.children() {
        render_node(child, child_walk, &mut out);
    }
    out
}

/// Cheap tag-name membership test to avoid building El for plain elements.
fn is_component_tag(name: &str, config: &Config) -> bool {
    config.components.all_tags().contains(&name)
}

/// Escape text-node content the same way html5ever's serializer does.
fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\u{a0}' => out.push_str("&nbsp;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Escape attribute values the same way html5ever's serializer does.
fn escape_attr(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\u{a0}' => out.push_str("&nbsp;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(html: &str) -> String {
        render(html, &Config::default())
    }

    #[test]
    fn plain_fragment_roundtrip() {
        assert_eq!(r(r#"<p class="a">hi</p>"#), r#"<p class="a">hi</p>"#);
    }

    #[test]
    fn text_entities_reescaped() {
        assert_eq!(r("a &amp; b &lt;c&gt;"), "a &amp; b &lt;c&gt;");
    }

    #[test]
    fn nbsp_reescaped() {
        assert_eq!(r("a&nbsp;b"), "a&nbsp;b");
    }

    #[test]
    fn comments_preserved_verbatim() {
        let html = "<!--[if mso]><table><tr><td>x</td></tr></table><![endif]-->";
        assert_eq!(r(html), html);
    }

    #[test]
    fn style_content_not_escaped() {
        let html = "<style>a > b { color: red; }</style>";
        assert_eq!(r(html), html);
    }

    #[test]
    fn void_elements_no_close_tag() {
        assert_eq!(r(r#"<img src="x.png"><br>"#), r#"<img src="x.png"><br>"#);
    }

    #[test]
    fn boolean_attribute_stays_bare() {
        // Note: use a void element here. A bare <td> in a body-context
        // fragment gets foster-parented away by the HTML5 parser, and
        // <video>/<button> are component tags (transformed from Task 3 on).
        assert_eq!(r(r#"<input type="text" disabled>"#), r#"<input type="text" disabled>"#);
    }

    #[test]
    fn attribute_value_escaping() {
        assert_eq!(
            r(r#"<p title="a &quot;b&quot; &amp; c">x</p>"#),
            r#"<p title="a &quot;b&quot; &amp; c">x</p>"#
        );
    }

    #[test]
    fn full_document_preserved() {
        let html = "<!DOCTYPE html><html><head><title>T</title></head><body><p>x</p></body></html>";
        assert_eq!(r(html), html);
    }

    #[test]
    fn legacy_xhtml_doctype_preserved() {
        let html = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd"><html><head></head><body><p>x</p></body></html>"#;
        assert_eq!(r(html), html);
    }

    #[test]
    fn nested_structure_roundtrip() {
        let html = r#"<table class="outer"><tbody><tr><td><a href="https://x.dev/?a=1&amp;b=2">go</a></td></tr></tbody></table>"#;
        assert_eq!(r(html), html);
    }

    #[test]
    fn component_transformed() {
        let out = r(r#"<button href="https://x.dev">Go</button>"#);
        assert!(out.contains(r#"class="button""#));
        assert!(out.contains("https://x.dev"));
        assert!(!out.contains("<button"));
    }

    #[test]
    fn nested_components_transform_bottom_up() {
        let out = r(r##"<container><row><column><button href="#">Go</button></column></row></container>"##);
        assert!(out.contains(r#"class="container""#));
        assert!(out.contains(r#"class="row""#));
        assert!(out.contains("small-12"));
        assert!(out.contains(r#"class="button""#));
        assert!(!out.contains("<container"));
        assert!(!out.contains("<row"));
        assert!(!out.contains("<column"));
        assert!(!out.contains("<button "));
    }

    #[test]
    fn capitalized_component_tag_transforms() {
        // html5ever lowercases tag names; the old regex replacement was
        // case-sensitive and silently halted the whole document here.
        let out = r(r#"<Button HREF="https://x.dev">Go</Button><row>r</row>"#);
        assert!(out.contains(r#"class="button""#));
        assert!(out.contains(r#"class="row""#));
        assert!(!out.contains("<Button"));
    }

    #[test]
    fn component_tag_inside_attribute_value_untouched() {
        let out = r(r#"<p title="see <button>">x</p>"#);
        assert_eq!(out, r#"<p title="see <button>">x</p>"#);
    }

    #[test]
    fn components_inside_outlook_transform() {
        let out = r(r##"<outlook><button href="#">B</button></outlook>"##);
        assert!(out.contains("<!--[if mso]>"));
        assert!(out.contains(r#"class="button""#));
        assert!(!out.contains("<button "));
    }

    #[test]
    fn two_columns_split_evenly() {
        let out = r("<row><column>A</column><column>B</column></row>");
        assert_eq!(out.matches("large-6").count(), 2);
        assert!(out.contains("first"));
        assert!(out.contains("last"));
    }

    #[test]
    fn comment_between_columns_does_not_break_grid() {
        let out = r("<row><column>A</column><!-- note --><column>B</column></row>");
        assert_eq!(out.matches("large-6").count(), 2);
        assert!(out.contains("<!-- note -->"));
    }

    #[test]
    fn three_columns_split() {
        let out = r("<row><column>A</column><column>B</column><column>C</column></row>");
        assert_eq!(out.matches("large-4").count(), 3);
    }

    #[test]
    fn unknown_tags_pass_through() {
        assert_eq!(r("<widget x=\"1\">hi</widget>"), "<widget x=\"1\">hi</widget>");
    }
}

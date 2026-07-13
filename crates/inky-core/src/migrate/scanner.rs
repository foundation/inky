//! A tolerant, quote-aware HTML tag scanner for source-to-source migration.
//!
//! Tokenizes source into verbatim `Text` spans and parsed tags. Anything
//! that cannot be parsed as a tag — unterminated tags, lone `<`, comments,
//! ERB `<%...%>` tags, doctypes, and the entire content of raw-text
//! elements — stays inside `Text` and is emitted byte-for-byte. Migration
//! must never guess: when in doubt, it is text.

// Wired into migrate() in the next task; remove this allow then.
#![allow(dead_code)]

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Quote {
    Double,
    Single,
    Unquoted,
    /// Attribute with no value at all (`<button expand>`).
    Bare,
}

#[derive(Debug, Clone)]
pub(crate) struct Attr {
    pub name: String,
    pub name_out: String,
    pub value: Option<String>,
    pub quote: Quote,
}

impl Attr {
    pub fn new_double(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            name_out: name.to_string(),
            value: Some(value.to_string()),
            quote: Quote::Double,
        }
    }

    pub fn new_bare(name: &str) -> Self {
        Self {
            name: name.to_string(),
            name_out: name.to_string(),
            value: None,
            quote: Quote::Bare,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Tag {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub self_closing: bool,
    pub span: Range<usize>,
    pub dirty: bool,
    pub deleted: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CloseTag {
    pub name: String,
    pub span: Range<usize>,
    pub renamed: Option<String>,
    pub deleted: bool,
}

#[derive(Debug)]
pub(crate) enum Token {
    Text(Range<usize>),
    Open(Tag),
    Close(CloseTag),
}

pub(crate) struct Doc<'a> {
    pub src: &'a str,
    pub tokens: Vec<Token>,
}

/// Elements whose content is never scanned for tags.
const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style", "textarea", "title", "raw"];

pub(crate) fn scan(src: &str) -> Doc<'_> {
    let bytes = src.as_bytes();
    let mut tokens = Vec::new();
    let mut text_start = 0; // start of the current pending Text run
    let mut i = 0;

    macro_rules! flush_text {
        ($upto:expr) => {
            if text_start < $upto {
                tokens.push(Token::Text(text_start..$upto));
            }
        };
    }

    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        let rest = &src[i..];

        // Comments, ERB tags, doctypes/PIs: skip within the text run.
        if rest.starts_with("<!--") {
            i = rest.find("-->").map(|e| i + e + 3).unwrap_or(src.len());
            continue;
        }
        if rest.starts_with("<%") {
            i = rest.find("%>").map(|e| i + e + 2).unwrap_or(src.len());
            continue;
        }
        if rest.starts_with("<!") || rest.starts_with("<?") {
            i = rest.find('>').map(|e| i + e + 1).unwrap_or(src.len());
            continue;
        }

        // Closing tag
        if let Some(after_slash) = rest.strip_prefix("</") {
            if after_slash.starts_with(|c: char| c.is_ascii_alphabetic()) {
                if let Some(gt) = rest.find('>') {
                    let end = i + gt + 1;
                    let name: String = after_slash
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                        .collect::<String>()
                        .to_ascii_lowercase();
                    flush_text!(i);
                    tokens.push(Token::Close(CloseTag {
                        name,
                        span: i..end,
                        renamed: None,
                        deleted: false,
                    }));
                    text_start = end;
                    i = end;
                    continue;
                }
            }
            // "</" not followed by a parseable close: literal text
            i += 1;
            continue;
        }

        // Opening tag
        match parse_open_tag(src, i) {
            Some((tag, end)) => {
                flush_text!(i);
                let name = tag.name.clone();
                if RAW_TEXT_ELEMENTS.contains(&name.as_str()) {
                    // Opaque: open tag + content + close tag all stay Text.
                    let after = find_raw_close(src, end, &name).unwrap_or(src.len());
                    tokens.push(Token::Text(i..after));
                    text_start = after;
                    i = after;
                } else {
                    tokens.push(Token::Open(tag));
                    text_start = end;
                    i = end;
                }
            }
            None => {
                // Not parseable as a tag: the '<' is literal text.
                i += 1;
            }
        }
    }
    flush_text!(src.len());

    Doc { src, tokens }
}

/// Parse an opening tag starting at `start` (src[start] == '<').
/// Returns the parsed Tag and the index just past the closing '>'.
/// Returns None for anything malformed — the caller treats it as text.
fn parse_open_tag(src: &str, start: usize) -> Option<(Tag, usize)> {
    let bytes = src.as_bytes();
    let mut i = start + 1;

    // Tag name: [a-zA-Z][a-zA-Z0-9-]*
    if i >= bytes.len() || !bytes[i].is_ascii_alphabetic() {
        return None;
    }
    let name_start = i;
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
        i += 1;
    }
    let name = src[name_start..i].to_ascii_lowercase();

    let mut attrs = Vec::new();
    let mut self_closing = false;

    loop {
        // Skip whitespace
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return None; // unterminated tag
        }
        match bytes[i] {
            b'>' => {
                i += 1;
                break;
            }
            b'/' => {
                // Must be "/>" (possibly with whitespace before '>')
                let mut j = i + 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'>' {
                    self_closing = true;
                    i = j + 1;
                    break;
                }
                return None; // stray '/' — malformed
            }
            _ => {
                // Attribute name: up to whitespace, '=', '/', '>'
                let attr_start = i;
                while i < bytes.len()
                    && !bytes[i].is_ascii_whitespace()
                    && bytes[i] != b'='
                    && bytes[i] != b'/'
                    && bytes[i] != b'>'
                {
                    i += 1;
                }
                if i == attr_start {
                    return None; // no progress — malformed
                }
                let name_out = src[attr_start..i].to_string();
                let attr_name = name_out.to_ascii_lowercase();

                // Skip whitespace before a possible '='
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'=' {
                    j += 1;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if j >= bytes.len() {
                        return None;
                    }
                    let (value, quote, after) = match bytes[j] {
                        b'"' => {
                            let end = src[j + 1..].find('"').map(|e| j + 1 + e)?;
                            (src[j + 1..end].to_string(), Quote::Double, end + 1)
                        }
                        b'\'' => {
                            let end = src[j + 1..].find('\'').map(|e| j + 1 + e)?;
                            (src[j + 1..end].to_string(), Quote::Single, end + 1)
                        }
                        _ => {
                            let mut k = j;
                            while k < bytes.len()
                                && !bytes[k].is_ascii_whitespace()
                                && bytes[k] != b'>'
                            {
                                k += 1;
                            }
                            if k == j {
                                return None;
                            }
                            (src[j..k].to_string(), Quote::Unquoted, k)
                        }
                    };
                    attrs.push(Attr {
                        name: attr_name,
                        name_out,
                        value: Some(value),
                        quote,
                    });
                    i = after;
                } else {
                    attrs.push(Attr {
                        name: attr_name,
                        name_out,
                        value: None,
                        quote: Quote::Bare,
                    });
                    // i stays at the first non-name char (whitespace/'>'/'/')
                }
            }
        }
    }

    Some((
        Tag {
            name,
            attrs,
            self_closing,
            span: start..i,
            dirty: false,
            deleted: false,
        },
        i,
    ))
}

/// Find the position just past the matching `</name ... >` (case-insensitive,
/// name boundary enforced) starting the search at `from`.
fn find_raw_close(src: &str, from: usize, name: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let needle = format!("</{}", name);
    let nb = needle.as_bytes();
    let mut i = from;
    while i + nb.len() <= bytes.len() {
        if bytes[i..i + nb.len()].eq_ignore_ascii_case(nb) {
            // Boundary: next char must be '>' or whitespace
            let after = i + nb.len();
            if after < bytes.len() && (bytes[after] == b'>' || bytes[after].is_ascii_whitespace())
            {
                return src[after..].find('>').map(|e| after + e + 1);
            }
        }
        i += 1;
    }
    None
}

impl Doc<'_> {
    pub(crate) fn emit(&self) -> String {
        let mut out = String::with_capacity(self.src.len());
        for token in &self.tokens {
            match token {
                Token::Text(range) => out.push_str(&self.src[range.clone()]),
                Token::Open(tag) => {
                    if tag.deleted {
                        continue;
                    }
                    if !tag.dirty {
                        out.push_str(&self.src[tag.span.clone()]);
                        continue;
                    }
                    out.push('<');
                    out.push_str(&tag.name);
                    for attr in &tag.attrs {
                        out.push(' ');
                        out.push_str(&attr.name_out);
                        if let Some(value) = &attr.value {
                            match attr.quote {
                                Quote::Double | Quote::Bare => {
                                    out.push_str("=\"");
                                    out.push_str(value);
                                    out.push('"');
                                }
                                Quote::Single => {
                                    out.push_str("='");
                                    out.push_str(value);
                                    out.push('\'');
                                }
                                Quote::Unquoted => {
                                    out.push('=');
                                    out.push_str(value);
                                }
                            }
                        }
                    }
                    if tag.self_closing {
                        out.push_str(" />");
                    } else {
                        out.push('>');
                    }
                }
                Token::Close(close) => {
                    if close.deleted {
                        continue;
                    }
                    match &close.renamed {
                        Some(new_name) => {
                            out.push_str("</");
                            out.push_str(new_name);
                            out.push('>');
                        }
                        None => out.push_str(&self.src[close.span.clone()]),
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// scan + emit with no mutations must be byte-identical for ANY input.
    fn roundtrip(src: &str) {
        let doc = scan(src);
        assert_eq!(doc.emit(), src, "roundtrip changed bytes for: {src}");
    }

    #[test]
    fn roundtrip_plain_text() {
        roundtrip("hello & <b>world</b> &amp; more");
    }

    #[test]
    fn roundtrip_exotic_formatting() {
        roundtrip("<div   class = \"a\"\n    id='b'\n>x</div  >\n<!-- note --><br/>");
    }

    #[test]
    fn roundtrip_unterminated_tag_is_text() {
        roundtrip("text with a lone < bracket and <notclosed attr=\"x\"");
    }

    #[test]
    fn parses_tag_with_gt_in_quoted_value() {
        let doc = scan(r#"<column large="6" title="a > b">x</column>"#);
        let Token::Open(tag) = &doc.tokens[0] else {
            panic!("expected open tag")
        };
        assert_eq!(tag.name, "column");
        assert_eq!(tag.attrs.len(), 2);
        assert_eq!(tag.attrs[1].name, "title");
        assert_eq!(tag.attrs[1].value.as_deref(), Some("a > b"));
    }

    #[test]
    fn parses_quote_styles() {
        let doc = scan(r#"<t a="1" b='2' c=3 d>"#);
        let Token::Open(tag) = &doc.tokens[0] else {
            panic!()
        };
        assert!(matches!(tag.attrs[0].quote, Quote::Double));
        assert!(matches!(tag.attrs[1].quote, Quote::Single));
        assert!(matches!(tag.attrs[2].quote, Quote::Unquoted));
        assert!(matches!(tag.attrs[3].quote, Quote::Bare));
        assert_eq!(tag.attrs[2].value.as_deref(), Some("3"));
        assert_eq!(tag.attrs[3].value, None);
    }

    #[test]
    fn attr_names_lowercased_with_original_kept() {
        let doc = scan(r#"<COLUMNS LARGE="6">"#);
        let Token::Open(tag) = &doc.tokens[0] else {
            panic!()
        };
        assert_eq!(tag.name, "columns");
        assert_eq!(tag.attrs[0].name, "large");
        assert_eq!(tag.attrs[0].name_out, "LARGE");
    }

    #[test]
    fn self_closing_detected() {
        let doc = scan(r#"<spacer size="16"/>"#);
        let Token::Open(tag) = &doc.tokens[0] else {
            panic!()
        };
        assert!(tag.self_closing);
    }

    #[test]
    fn comments_stay_inside_text_runs() {
        let doc = scan("<!-- <columns large=\"6\"> --><p>x</p>");
        // First token must be Text covering the whole comment; the <columns>
        // inside it must NOT be parsed as a tag.
        assert!(matches!(&doc.tokens[0], Token::Text(_)));
        let opens: Vec<&Tag> = doc
            .tokens
            .iter()
            .filter_map(|t| match t {
                Token::Open(tag) => Some(tag),
                _ => None,
            })
            .collect();
        assert_eq!(opens.len(), 1);
        assert_eq!(opens[0].name, "p");
    }

    #[test]
    fn erb_tags_stay_inside_text_runs() {
        let doc = scan(r#"<%= link_to "a <columns>" %><p>x</p>"#);
        let opens: Vec<&str> = doc
            .tokens
            .iter()
            .filter_map(|t| match t {
                Token::Open(tag) => Some(tag.name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(opens, vec!["p"]);
    }

    #[test]
    fn raw_text_elements_are_opaque() {
        for el in ["script", "style", "textarea", "title", "raw"] {
            let src = format!("<{el}>var x = \"<columns large='6'>\";</{el}><p>x</p>");
            let doc = scan(&src);
            let opens: Vec<&str> = doc
                .tokens
                .iter()
                .filter_map(|t| match t {
                    Token::Open(tag) => Some(tag.name.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(opens, vec!["p"], "raw-text element <{el}> leaked tags");
            assert_eq!(doc.emit(), src);
        }
    }

    #[test]
    fn raw_text_close_is_case_insensitive_and_boundary_checked() {
        // </SCRIPT> closes; </scripts> does not.
        let src = "<script>a</scripts>b</SCRIPT><p>x</p>";
        let doc = scan(src);
        let opens: Vec<&str> = doc
            .tokens
            .iter()
            .filter_map(|t| match t {
                Token::Open(tag) => Some(tag.name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(opens, vec!["p"]);
        assert_eq!(doc.emit(), src);
    }

    #[test]
    fn dirty_tag_reserializes_preserving_quotes() {
        let mut doc = scan(r#"<column large="6" note='n' flag>x</column>"#);
        if let Token::Open(tag) = &mut doc.tokens[0] {
            tag.attrs[0].name_out = "lg".to_string();
            tag.dirty = true;
        }
        assert_eq!(doc.emit(), r#"<column lg="6" note='n' flag>x</column>"#);
    }

    #[test]
    fn dirty_self_closing_emits_slash() {
        let mut doc = scan(r#"<spacer size="16"/>"#);
        if let Token::Open(tag) = &mut doc.tokens[0] {
            tag.attrs[0].name_out = "height".to_string();
            tag.dirty = true;
        }
        assert_eq!(doc.emit(), r#"<spacer height="16" />"#);
    }

    #[test]
    fn deleted_tokens_emit_nothing() {
        let mut doc = scan("<center><menu>x</menu></center>");
        if let Token::Open(tag) = &mut doc.tokens[0] {
            tag.deleted = true;
        }
        let last = doc.tokens.len() - 1;
        if let Token::Close(c) = &mut doc.tokens[last] {
            c.deleted = true;
        }
        assert_eq!(doc.emit(), "<menu>x</menu>");
    }

    #[test]
    fn close_tag_rename() {
        let mut doc = scan("</columns>");
        if let Token::Close(c) = &mut doc.tokens[0] {
            c.renamed = Some("column".to_string());
        }
        assert_eq!(doc.emit(), "</column>");
    }

    #[test]
    fn doctype_and_pi_stay_text() {
        roundtrip("<!DOCTYPE html><?xml version=\"1.0\"?><p>x</p>");
        let doc = scan("<!DOCTYPE html><p>x</p>");
        assert!(matches!(&doc.tokens[0], Token::Text(_)));
    }
}

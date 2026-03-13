use scraper::ElementRef;

use crate::attrs::{get_attr, get_attrs, get_classes};
use super::helpers::inner_html;

/// `<blockquote cite="Author Name" color="#999999">Quoted text here</blockquote>`
///
/// Renders a styled blockquote with a left border and optional citation.
/// Note: This transforms the custom `<blockquote>` element (registered as a
/// component tag) into email-safe table markup, not a standard HTML blockquote.
pub fn make_blockquote(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let cite = get_attr(element, "cite");
    let color = get_attr(element, "color").unwrap_or_else(|| "#cccccc".to_string());
    let inner = inner_html(element);

    let mut classes = vec!["blockquote".to_string()];
    classes.extend(get_classes(element));
    let class_str = classes.join(" ");

    let mut content = format!(
        r#"<tr><td style="padding: 0 0 0 16px; font-style: italic;">{}</td></tr>"#,
        inner
    );

    if let Some(author) = cite {
        content.push_str(&format!(
            r#"<tr><td style="padding: 8px 0 0 16px; font-size: 14px; color: #999999;">&mdash; {}</td></tr>"#,
            author
        ));
    }

    format!(
        r#"<table role="presentation"{} class="{}" width="100%" cellpadding="0" cellspacing="0" style="border-left: 4px solid {};"><tbody>{}</tbody></table>"#,
        attrs, class_str, color, content
    )
}

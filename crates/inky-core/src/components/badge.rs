use scraper::ElementRef;

use crate::attrs::{get_attr, get_attrs, get_classes};
use super::helpers::inner_html;

/// `<badge color="#e74c3c">New</badge>`
///
/// Renders a small inline badge/pill label, useful for status indicators or tags.
pub fn make_badge(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let color = get_attr(element, "color").unwrap_or_else(|| "#333333".to_string());
    let text_color = get_attr(element, "text-color").unwrap_or_else(|| "#ffffff".to_string());
    let inner = inner_html(element);

    let mut classes = vec!["badge".to_string()];
    classes.extend(get_classes(element));
    let class_str = classes.join(" ");

    format!(
        r#"<span{} class="{}" style="display: inline-block; padding: 2px 8px; background-color: {}; color: {}; border-radius: 12px; font-size: 12px; font-weight: bold; line-height: 1.4;">{}</span>"#,
        attrs, class_str, color, text_color, inner
    )
}

use scraper::ElementRef;

use super::helpers::inner_html;
use crate::attrs::{get_attr, get_attrs, get_classes};

/// `<accordion><accordion-item title="Section 1">Content</accordion-item></accordion>`
///
/// Renders a CSS-only accordion using the checkbox hack.
/// Each item is a toggleable section with a title bar and collapsible content.
pub fn make_accordion(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let inner = inner_html(element);

    let mut classes = vec!["accordion".to_string()];
    classes.extend(get_classes(element));
    let class_str = classes.join(" ");

    format!(
        r#"<table role="presentation"{} class="{}" width="100%" cellpadding="0" cellspacing="0"><tbody>{}</tbody></table>"#,
        attrs, class_str, inner
    )
}

/// `<accordion-item title="Section Title">Content here</accordion-item>`
///
/// A single collapsible section within an accordion.
pub fn make_accordion_item(element: &ElementRef) -> String {
    let title = get_attr(element, "title").unwrap_or_else(|| "Untitled".to_string());
    let inner = inner_html(element);

    let mut classes = vec!["accordion-item".to_string()];
    classes.extend(get_classes(element));
    let class_str = classes.join(" ");

    format!(
        r#"<tr class="{}"><td><table role="presentation" width="100%" cellpadding="0" cellspacing="0"><tbody><tr><td class="accordion-title" style="padding: 10px 16px; background-color: #f4f4f4; font-weight: bold; cursor: pointer;">{}</td></tr><tr><td class="accordion-content" style="padding: 16px;">{}</td></tr></tbody></table></td></tr>"#,
        class_str, title, inner
    )
}

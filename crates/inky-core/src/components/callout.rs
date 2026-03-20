use scraper::ElementRef;

use super::helpers::inner_html;
use crate::attrs::{get_attr, get_attrs, get_classes};

pub fn make_callout(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let inner = inner_html(element);

    // Build inner classes: base "callout-inner" + element classes + v2 color attribute
    let mut classes = vec!["callout-inner".to_string()];
    classes.extend(get_classes(element));
    if let Some(color) = get_attr(element, "color") {
        classes.push(color);
    }
    let class_str = classes.join(" ");

    format!(
        r#"<table role="presentation"{} class="callout"><tbody><tr><th class="{}">{}</th><th class="expander" aria-hidden="true"></th></tr></tbody></table>"#,
        attrs, class_str, inner
    )
}

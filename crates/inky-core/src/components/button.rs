use scraper::ElementRef;

use crate::attrs::{get_attr, get_attrs, get_classes, has_class};
use super::helpers::inner_html;

pub fn make_button(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let mut inner = inner_html(element);

    let target = match get_attr(element, "target") {
        Some(t) => format!(" target={}", t),
        None => String::new(),
    };

    if let Some(href) = get_attr(element, "href") {
        if !href.is_empty() {
            inner = format!(
                r#"<a{} href="{}"{}>{}</a>"#,
                attrs, href, target, inner
            );
        }
    }

    let expander;
    if has_class(element, "expand") || has_class(element, "expanded") {
        inner = format!("<center>{}</center>", inner);
        expander = "\n<td class=\"expander\"></td>";
    } else {
        expander = "";
    }

    // Build classes: base "button" + element classes + v2 size/color attributes
    let mut classes = vec!["button".to_string()];
    classes.extend(get_classes(element));
    if let Some(size) = get_attr(element, "size") {
        classes.push(size);
    }
    if let Some(color) = get_attr(element, "color") {
        classes.push(color);
    }
    let class_str = classes.join(" ");

    format!(
        r#"<table role="presentation" class="{}"><tbody><tr><td><table role="presentation"><tbody><tr><td>{}</td></tr></tbody></table></td>{}</tr></tbody></table>"#,
        class_str, inner, expander
    )
}

use scraper::ElementRef;

use super::helpers::{build_classes, inner_html};
use crate::attrs::{get_attr, get_attrs, get_classes};

pub fn make_menu(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let inner = inner_html(element);

    // Build classes: base "menu" + element classes + v2 direction attribute
    let mut classes = vec!["menu".to_string()];
    classes.extend(get_classes(element));
    if let Some(direction) = get_attr(element, "direction") {
        classes.push(direction);
    }
    let class_str = classes.join(" ");

    format!(
        r#"<table role="presentation"{} class="{}"><tbody><tr><td><table role="presentation"><tbody><tr>{}</tr></tbody></table></td></tr></tbody></table>"#,
        attrs, class_str, inner
    )
}

pub fn make_menu_item(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let href = get_attr(element, "href").unwrap_or_default();
    let target = match get_attr(element, "target") {
        Some(t) => format!(" target={}", t),
        None => String::new(),
    };
    let classes = build_classes("menu-item", element);
    let inner = inner_html(element);
    format!(
        r#"<th{} class="{}"><a href="{}"{}>{}</a></th>"#,
        attrs, classes, href, target, inner
    )
}

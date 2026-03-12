use scraper::ElementRef;

use crate::attrs::{get_attr, get_attrs};
use super::helpers::{inner_html, build_classes};

pub fn make_menu(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("menu", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="{}"><tbody><tr><td><table role="presentation"><tbody><tr>{}</tr></tbody></table></td></tr></tbody></table>"#,
        attrs, classes, inner
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

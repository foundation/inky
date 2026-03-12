use scraper::ElementRef;

use crate::attrs::get_attrs;
use super::helpers::{inner_html, build_classes};

pub fn make_wrapper(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("wrapper", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="{}" align="center"><tbody><tr><td class="wrapper-inner">{}</td></tr></tbody></table>"#,
        attrs, classes, inner
    )
}

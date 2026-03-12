use scraper::ElementRef;

use crate::attrs::get_attrs;
use super::helpers::{inner_html, build_classes};

pub fn make_row(element: &ElementRef) -> String {
    let attrs = get_attrs(element);
    let classes = build_classes("row", element);
    let inner = inner_html(element);
    format!(
        r#"<table role="presentation"{} class="{}"><tbody><tr>{}</tr></tbody></table>"#,
        attrs, classes, inner
    )
}
